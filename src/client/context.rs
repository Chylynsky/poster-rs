use crate::{
    client::{
        error::{HandleClosed, MqttError, SocketClosed},
        handle::ContextHandle,
        message::*,
        utils::*,
    },
    codec::*,
    core::{
        base_types::NonZero,
        error::{CodecError, MandatoryPropertyMissing},
        utils::PacketID,
    },
    io::{RxPacketStream, TxPacketStream},
};
use bytes::Bytes;
use core::sync::atomic::{AtomicU16, AtomicU32};
use futures::{
    channel::{mpsc, oneshot},
    AsyncRead, AsyncWrite, FutureExt, SinkExt, StreamExt,
};
use std::{collections::VecDeque, sync::Arc};

/// Client context. It is responsible for socket management and direct communication with the broker.
///
pub struct Context<RxStreamT, TxStreamT> {
    rx: RxPacketStream<RxStreamT>,
    tx: TxPacketStream<TxStreamT>,

    awaiting_ack: VecDeque<(usize, oneshot::Sender<RxPacket>)>,
    subscriptions: VecDeque<(usize, mpsc::UnboundedSender<RxPacket>)>,
    unackowledged: VecDeque<(usize, Bytes)>,

    message_queue: mpsc::UnboundedReceiver<ContextMessage>,

    resend_flag: bool,
}

impl<RxStreamT, TxStreamT> Context<RxStreamT, TxStreamT>
where
    RxStreamT: AsyncRead + Unpin,
    TxStreamT: AsyncWrite + Unpin,
{
    /// Creates a new [Context] instance.
    ///
    /// # Arguments
    /// * `rx` - Read half of the stream, must be [AsyncRead] + [Unpin].
    /// * `tx` - Write half of the stream, must be [AsyncWrite] + [Unpin].
    ///
    /// # Note
    /// User is responsible for making sure that at the point of calling this method,
    /// both the `rx` and `tx` are connected to the broker and ready for communication.
    ///
    pub fn new(rx: RxStreamT, tx: TxStreamT) -> (Self, ContextHandle) {
        let (sender, receiver) = mpsc::unbounded();

        (
            Self {
                rx: RxPacketStream::with_capacity(256, rx),
                tx: TxPacketStream::from(tx),
                awaiting_ack: VecDeque::new(),
                subscriptions: VecDeque::new(),
                unackowledged: VecDeque::new(),
                message_queue: receiver,
                resend_flag: false,
            },
            ContextHandle {
                sender,
                packet_id: Arc::new(AtomicU16::from(1)),
                sub_id: Arc::new(AtomicU32::from(1)),
            },
        )
    }

    /// Makes [Context] object start processing MQTT traffic, blocking the current task/thread until
    /// graceful disconnection or error. Successful disconnection via [disconnect] method or
    /// receiving a [Disconnect](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901205)
    /// packet with reason a code equal to 0 (success) is considered a graceful disconnection.
    ///
    pub async fn run(&mut self) -> Result<(), MqttError>
    where
        RxStreamT: AsyncRead + Unpin,
        TxStreamT: AsyncWrite + Unpin,
    {
        let mut pck_fut = self.rx.next().fuse();
        let mut msg_fut = self.message_queue.next().fuse();

        // Resend all unacknowledged packets
        if self.resend_flag {
            for (_, packet) in self.unackowledged.iter() {
                self.tx.write(packet.as_ref()).await?;
                self.resend_flag = false;
            }
        }

        loop {
            futures::select! {
                maybe_rx_packet = pck_fut => {
                    match maybe_rx_packet {
                        Some(rx_packet) => match rx_packet? {
                            RxPacket::Publish(publish) => match publish.subscription_identifier {
                                Some(subscription_identifier) => {
                                    let sub_id = NonZero::from(subscription_identifier).get().value() as usize;
                                    let maybe_pos = linear_search_by_key(&self.subscriptions, sub_id);

                                    if let Some((_, subscription)) = maybe_pos.map(|pos| &mut self.subscriptions[pos])  {
                                        // User may drop the receiving stream,
                                        // in that case remove it from the active subscriptions map.
                                        if (subscription.send(RxPacket::Publish(publish)).await).is_err() {
                                            linear_search_by_key(&self.subscriptions, sub_id).and_then(|pos| self.subscriptions.remove(pos));
                                        }
                                    }
                                }
                                None => {
                                    return Err(
                                        CodecError::MandatoryPropertyMissing(MandatoryPropertyMissing).into(),
                                    )
                                }
                            },
                            RxPacket::Disconnect(disconnect) => {
                                if disconnect.reason == DisconnectReason::Success {
                                    return Ok(()); // Graceful disconnection.
                                }

                                return Err(disconnect.into());
                            }
                            other => {
                                let action_id = rx_action_id(&other);

                                if let RxPacket::Puback(_) = &other {
                                    linear_search_by_key(&self.unackowledged, action_id).and_then(|pos| self.unackowledged.remove(pos));
                                } else if let RxPacket::Pubcomp(_) = &other {
                                    linear_search_by_key(&self.unackowledged, action_id).and_then(|pos| self.unackowledged.remove(pos));
                                }

                                if let Some((_, sender)) = linear_search_by_key(&self.awaiting_ack, action_id).and_then(|pos| self.awaiting_ack.remove(pos)) {
                                    // Error here indicates internal error, the receiver
                                    // end is inside one of the ContextHandle methods.
                                    sender.send(other).map_err(|_| HandleClosed)?;
                                }
                            }
                        },
                        None => {
                            self.resend_flag = true;
                            return Err(SocketClosed.into())
                        },
                    }

                    pck_fut = self.rx.next().fuse();
                },
                maybe_msg = msg_fut => {
                    match maybe_msg {
                        Some(msg) => match msg {
                            ContextMessage::Disconnect(packet) => {
                                self.tx.write(packet.freeze().as_ref()).await?;
                                return Ok(()) // Graceful disconnection.
                            },
                            ContextMessage::FireAndForget(packet) => {
                                self.tx.write(packet.freeze().as_ref()).await?;

                            },
                            ContextMessage::AwaitAck(mut msg) => {
                                self.tx.write(msg.packet.as_ref()).await?;
                                self.awaiting_ack.push_back((msg.action_id, msg.response_channel));

                                let packet_id = msg.packet.get_mut(0).unwrap();

                                if *packet_id == PublishTx::PACKET_ID {
                                    *packet_id |= (1 << 3) as u8; // Set DUP flag in the PUBLISH fixed header
                                    self.unackowledged.push_back((msg.action_id, msg.packet.freeze()));
                                } else if *packet_id == PubrelTx::PACKET_ID {
                                    self.unackowledged.push_back((msg.action_id, msg.packet.freeze()));
                                }
                            },
                            ContextMessage::Subscribe(msg) => {
                                self.awaiting_ack.push_back((msg.action_id, msg.response_channel));
                                self.subscriptions.push_back((msg.subscription_identifier, msg.stream));
                                self.tx.write(msg.packet.freeze().as_ref()).await?;
                            },
                        }
                        None => return Err(HandleClosed.into()),
                    }

                    msg_fut = self.message_queue.next().fuse();
                }
            }
        }
    }
}
