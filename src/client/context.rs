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
use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, SystemTime},
};

struct Session {
    awaiting_ack: VecDeque<(usize, oneshot::Sender<RxPacket>)>,
    subscriptions: VecDeque<(usize, mpsc::UnboundedSender<RxPacket>)>,
    unackowledged: VecDeque<(usize, Bytes)>,
}

struct Connection {
    disconnection_timestamp: Option<SystemTime>,
    session_expiry_interval: Duration,
}

/// Client context. It is responsible for socket management and direct communication with the broker.
///
pub struct Context<RxStreamT, TxStreamT> {
    rx: RxPacketStream<RxStreamT>,
    tx: TxPacketStream<TxStreamT>,
    message_queue: mpsc::UnboundedReceiver<ContextMessage>,
    session: Session,
    connection: Connection,
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
                message_queue: receiver,
                session: Session {
                    awaiting_ack: VecDeque::new(),
                    subscriptions: VecDeque::new(),
                    unackowledged: VecDeque::new(),
                },
                connection: Connection {
                    disconnection_timestamp: None,
                    session_expiry_interval: Duration::from_secs(0),
                },
            },
            ContextHandle {
                sender,
                packet_id: Arc::new(AtomicU16::from(1)),
                sub_id: Arc::new(AtomicU32::from(1)),
            },
        )
    }

    fn is_reconnect(connection: &Connection) -> bool {
        connection.disconnection_timestamp.is_some()
    }

    fn session_expired(connection: &Connection) -> bool {
        debug_assert!(Self::is_reconnect(connection));

        connection.session_expiry_interval
            >= connection
                .disconnection_timestamp
                .and_then(|timestamp| timestamp.elapsed().ok())
                .unwrap()
    }

    async fn handle_message(
        tx: &mut TxPacketStream<TxStreamT>,
        session: &mut Session,
        msg: ContextMessage,
    ) -> Result<(), MqttError> {
        match msg {
            ContextMessage::Disconnect(packet) => {
                tx.write(packet.freeze().as_ref()).await?;
                // Graceful disconnection.
            }
            ContextMessage::FireAndForget(packet) => {
                tx.write(packet.freeze().as_ref()).await?;
            }
            ContextMessage::AwaitAck(mut msg) => {
                tx.write(msg.packet.as_ref()).await?;
                session
                    .awaiting_ack
                    .push_back((msg.action_id, msg.response_channel));

                let fixed_hdr = msg.packet.get_mut(0).unwrap();
                let packet_id = *fixed_hdr >> 4;

                if packet_id == PublishTx::PACKET_ID {
                    *fixed_hdr |= (1 << 3) as u8; // Set DUP flag in the PUBLISH fixed header
                    session
                        .unackowledged
                        .push_back((msg.action_id, msg.packet.freeze()));
                } else if packet_id == PubrelTx::PACKET_ID {
                    session
                        .unackowledged
                        .push_back((msg.action_id, msg.packet.freeze()));
                }
            }
            ContextMessage::Subscribe(msg) => {
                session
                    .awaiting_ack
                    .push_back((msg.action_id, msg.response_channel));
                session
                    .subscriptions
                    .push_back((msg.subscription_identifier, msg.stream));
                tx.write(msg.packet.freeze().as_ref()).await?;
            }
        }

        Ok(())
    }

    async fn handle_packet(session: &mut Session, packet: RxPacket) -> Result<(), MqttError> {
        match packet {
            RxPacket::Publish(publish) => match publish.subscription_identifier {
                Some(subscription_identifier) => {
                    let sub_id = NonZero::from(subscription_identifier).get().value() as usize;
                    let maybe_pos = linear_search_by_key(&session.subscriptions, sub_id);

                    if let Some((_, subscription)) =
                        maybe_pos.map(|pos| &mut session.subscriptions[pos])
                    {
                        // User may drop the receiving stream,
                        // in that case remove it from the active subscriptions map.
                        if (subscription.send(RxPacket::Publish(publish)).await).is_err() {
                            linear_search_by_key(&session.subscriptions, sub_id)
                                .and_then(|pos| session.subscriptions.remove(pos));
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
                    linear_search_by_key(&session.unackowledged, action_id)
                        .and_then(|pos| session.unackowledged.remove(pos));
                } else if let RxPacket::Pubcomp(_) = &other {
                    linear_search_by_key(&session.unackowledged, action_id)
                        .and_then(|pos| session.unackowledged.remove(pos));
                }

                if let Some((_, sender)) = linear_search_by_key(&session.awaiting_ack, action_id)
                    .and_then(|pos| session.awaiting_ack.remove(pos))
                {
                    // Error here indicates internal error, the receiver
                    // end is inside one of the ContextHandle methods.
                    sender.send(other).map_err(|_| HandleClosed)?;
                }
            }
        }

        Ok(())
    }

    async fn retransmit(
        tx: &mut TxPacketStream<TxStreamT>,
        connection: &mut Connection,
        session: &mut Session,
    ) -> Result<(), MqttError> {
        connection.disconnection_timestamp = None;

        for (_, packet) in session.unackowledged.iter() {
            tx.write(packet.as_ref()).await?;
        }

        Ok(())
    }

    /// Makes [Context] object start processing MQTT traffic, blocking (on .await) the current task until
    /// graceful disconnection or error. Successful disconnection via [disconnect] method or
    /// receiving a [Disconnect](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901205)
    /// packet with reason a code equal to 0 (success) is considered a graceful disconnection.
    ///
    pub async fn run(&mut self) -> Result<(), MqttError>
    where
        RxStreamT: AsyncRead + Unpin,
        TxStreamT: AsyncWrite + Unpin,
    {
        let tx = &mut self.tx;
        let rx = &mut self.rx;
        let message_queue = &mut self.message_queue;
        let session = &mut self.session;
        let connection = &mut self.connection;

        if Self::is_reconnect(connection) {
            if Self::session_expired(connection) {
                session.awaiting_ack.clear();
                session.subscriptions.clear();
                session.unackowledged.clear();
            }

            Self::retransmit(tx, connection, session).await?;
        }

        let mut pck_fut = rx.next().fuse();
        let mut msg_fut = message_queue.next().fuse();

        loop {
            futures::select! {
                maybe_rx_packet = pck_fut => {
                    match maybe_rx_packet {
                        Some(rx_packet) => Self::handle_packet(session, rx_packet?).await?,
                        None => {
                            connection.disconnection_timestamp = Some(SystemTime::now());
                            return Err(SocketClosed.into())
                        },
                    }

                    pck_fut = rx.next().fuse();
                },
                maybe_msg = msg_fut => {
                    match maybe_msg {
                        Some(msg) => Self::handle_message(tx, session, msg).await?,
                        None => return Err(HandleClosed.into()),
                    }

                    msg_fut = message_queue.next().fuse();
                }
            }
        }
    }
}
