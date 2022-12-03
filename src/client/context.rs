use crate::{
    client::{
        error::{HandleClosed, MqttError, SocketClosed},
        fut::SubscribeStream,
        opts::{AuthOpts, ConnectOpts, PublishOpts, SubscribeOpts, UnsubscribeOpts},
        rsp::{AuthRsp, ConnectRsp, PublishError, SubscribeError, UnsubscribeError},
        streams::{RxPacketStream, TxPacketStream},
    },
    codec::*,
    core::{
        base_types::{NonZero, QoS},
        error::{CodecError, MandatoryPropertyMissing},
        utils::{Encode, PacketID, SizedPacket},
    },
};
use bytes::{Bytes, BytesMut};
use core::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use either::{Either, Left, Right};
use futures::{
    channel::{mpsc, oneshot},
    AsyncRead, AsyncWrite, FutureExt, SinkExt, StreamExt,
};
use std::{collections::VecDeque, sync::Arc};

// TODO:
// - ACK subscriptions
// - Outbound quota
// - Accessing user properties
// - Automatic pings
// - Handle packet or subscription IDs in use
// - Authorization?
// - Context states?

fn tx_packet_to_action_id(packet: &TxPacket) -> usize {
    match packet {
        TxPacket::Connect(_) => 0,
        TxPacket::Auth(_) => 0,
        TxPacket::Subscribe(subscribe) => {
            ((SubackRx::PACKET_ID as usize) << 24)
                | ((subscribe.packet_identifier.get() as usize) << 8)
        }
        TxPacket::Unsubscribe(unsubscribe) => {
            ((UnsubackRx::PACKET_ID as usize) << 24)
                | ((unsubscribe.packet_identifier.get() as usize) << 8)
        }
        TxPacket::Pingreq(_) => (PingrespRx::PACKET_ID as usize) << 24,
        TxPacket::Publish(publish) => match publish.qos {
            QoS::AtLeastOnce => {
                (PubackRx::PACKET_ID as usize) << 24
                    | (publish
                        .packet_identifier
                        .map(|val| -> usize { val.get() as usize })
                        .unwrap()
                        << 8)
            }
            QoS::ExactlyOnce => {
                (PubrecRx::PACKET_ID as usize) << 24
                    | (publish
                        .packet_identifier
                        .map(|val| -> usize { val.get() as usize })
                        .unwrap()
                        << 8)
            }
            _ => unreachable!("Method cannot be called for QoS 0."),
        },
        TxPacket::Pubrel(pubrel) => {
            (PubcompRx::PACKET_ID as usize) << 24 | ((pubrel.packet_identifier.get() as usize) << 8)
        }
        TxPacket::Pubrec(pubrec) => {
            (PubrelRx::PACKET_ID as usize) << 24 | ((pubrec.packet_identifier.get() as usize) << 8)
        }
        _ => unreachable!("Unexpected packet type."),
    }
}

fn rx_packet_to_action_id(packet: &RxPacket) -> usize {
    match packet {
        RxPacket::Connack(_) => 0,
        RxPacket::Auth(_) => 0,
        RxPacket::Suback(suback) => {
            ((SubackRx::PACKET_ID as usize) << 24)
                | ((suback.packet_identifier.get() as usize) << 8)
        }
        RxPacket::Unsuback(unsuback) => {
            ((UnsubackRx::PACKET_ID as usize) << 24)
                | ((unsuback.packet_identifier.get() as usize) << 8)
        }
        RxPacket::Pingresp(_) => (PingrespRx::PACKET_ID as usize) << 24,
        RxPacket::Puback(puback) => {
            (PubackRx::PACKET_ID as usize) << 24 | ((puback.packet_identifier.get() as usize) << 8)
        }
        RxPacket::Pubrec(pubrec) => {
            (PubrecRx::PACKET_ID as usize) << 24 | ((pubrec.packet_identifier.get() as usize) << 8)
        }
        RxPacket::Pubrel(pubrel) => {
            (PubrelRx::PACKET_ID as usize) << 24 | ((pubrel.packet_identifier.get() as usize) << 8)
        }
        RxPacket::Pubcomp(pubcomp) => {
            (PubcompRx::PACKET_ID as usize) << 24
                | ((pubcomp.packet_identifier.get() as usize) << 8)
        }
        _ => unreachable!("Unexpected packet type."),
    }
}

fn linear_search_by_key<K, V>(deque: &VecDeque<(K, V)>, key: K) -> Option<usize>
where
    K: Copy + PartialEq,
{
    deque.iter().position(|(k, _)| *k == key)
}

struct AwaitAck {
    action_id: usize,
    packet: BytesMut,
    response_channel: oneshot::Sender<RxPacket>,
}

struct Subscribe {
    action_id: usize,
    subscription_identifier: usize,
    packet: BytesMut,
    response_channel: oneshot::Sender<RxPacket>,
    stream: mpsc::UnboundedSender<RxPacket>,
}

enum ContextMessage {
    FireAndForget(BytesMut),
    AwaitAck(AwaitAck),
    Disconnect(BytesMut),
    Subscribe(Subscribe),
}

/// Client context. It is responsible for socket management and direct communication with the broker.
pub struct Context<RxStreamT, TxStreamT> {
    rx: RxPacketStream<RxStreamT>,
    tx: TxPacketStream<TxStreamT>,

    awaiting_ack: VecDeque<(usize, oneshot::Sender<RxPacket>)>,
    subscriptions: VecDeque<(usize, mpsc::UnboundedSender<RxPacket>)>,
    unackowledged: VecDeque<(usize, Bytes)>,

    message_queue: mpsc::UnboundedReceiver<ContextMessage>,

    resend_flag: bool,
}

impl<RxStreamT, TxStreamT> Context<RxStreamT, TxStreamT> {
    pub const DEFAULT_BUF_SIZE: usize = 2048;
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
    pub fn new(rx: RxStreamT, tx: TxStreamT) -> (Self, ContextHandle) {
        let (sender, receiver) = mpsc::unbounded();

        (
            Self {
                rx: RxPacketStream::with_capacity(Self::DEFAULT_BUF_SIZE, rx),
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
                                let action_id = rx_packet_to_action_id(&other);

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

/// Cloneable handle to the client [Context]. This handle is needed to perform MQTT operations.
#[derive(Clone)]
pub struct ContextHandle {
    sender: mpsc::UnboundedSender<ContextMessage>,
    packet_id: Arc<AtomicU16>,
    sub_id: Arc<AtomicU32>,
}

impl ContextHandle {
    /// Creates a unique packet identifier.
    fn make_packet_id(&mut self) -> u16 {
        self.packet_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Creates a unique subscription identifier.
    fn make_subscription_id(&mut self) -> u32 {
        self.sub_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Performs connection with the broker on the protocol level. Calling this method corresponds to sending the
    /// [Connect](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901033) packet.
    ///
    /// If `authentication_method` and `authentication_data` are
    /// set in `opts`, the extended authorization is performed, the result of calling this method
    /// is then [AuthRsp]. Otherwise, the return type is [ConnectRsp].
    ///
    /// When in extended authorization mode, the authorize method is used for subsequent
    /// authorization requests.
    ///
    /// # Arguments
    /// `opts` - Connection options.
    pub async fn connect<'a>(
        &mut self,
        opts: ConnectOpts<'a>,
    ) -> Result<Either<ConnectRsp, AuthRsp>, MqttError> {
        let (sender, receiver) = oneshot::channel();

        let packet = opts.build()?;

        let mut buf = BytesMut::with_capacity(packet.packet_len());
        packet.encode(&mut buf);

        let message = ContextMessage::AwaitAck(AwaitAck {
            action_id: tx_packet_to_action_id(&TxPacket::Connect(packet)),
            packet: buf,
            response_channel: sender,
        });

        self.sender.send(message).await?;

        match receiver.await? {
            RxPacket::Connack(connack) => Ok(Left(ConnectRsp::from(connack))),
            RxPacket::Auth(auth) => Ok(Right(AuthRsp::from(auth))),
            _ => {
                unreachable!("Unexpected packet type.");
            }
        }
    }

    /// Method used for performing the extended authorization between the client and the broker. It corresponds to sending the
    /// [Auth](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901217) packet.
    /// User may perform multiple calls of this method, as needed, until the ConnectRsp is returned,
    /// meaning the authorization is successful.
    ///
    /// # Arguments
    /// `opts` - Authorization options
    pub async fn authorize<'a>(
        &mut self,
        opts: AuthOpts<'a>,
    ) -> Result<Either<ConnectRsp, AuthRsp>, MqttError> {
        let (sender, receiver) = oneshot::channel();

        let packet = opts.build()?;

        let mut buf = BytesMut::with_capacity(packet.packet_len());
        packet.encode(&mut buf);

        let message = ContextMessage::AwaitAck(AwaitAck {
            action_id: tx_packet_to_action_id(&TxPacket::Auth(packet)),
            packet: buf,
            response_channel: sender,
        });

        self.sender.send(message).await?;

        match receiver.await? {
            RxPacket::Connack(connack) => Ok(Left(ConnectRsp::from(connack))),
            RxPacket::Auth(auth) => Ok(Right(AuthRsp::from(auth))),
            _ => {
                unreachable!("Unexpected packet type.");
            }
        }
    }

    /// Performs graceful disconnection with the broker by sending the
    /// [Disconnect](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901205) packet.
    pub async fn disconnect(&mut self) -> Result<(), MqttError> {
        let mut builder = DisconnectTxBuilder::default();
        builder.reason(DisconnectReason::Success);

        let packet = builder.build().unwrap();

        let mut buf = BytesMut::with_capacity(packet.packet_len());
        packet.encode(&mut buf);

        let message = ContextMessage::Disconnect(buf);

        self.sender.send(message).await?;
        Ok(())
    }

    /// Sends ping to the broker by sending
    /// [Ping](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901195) packet.
    pub async fn ping(&mut self) -> Result<(), MqttError> {
        let (sender, receiver) = oneshot::channel();

        let builder = PingreqTxBuilder::default();
        let packet = builder.build().unwrap();

        let mut buf = BytesMut::with_capacity(packet.packet_len());
        packet.encode(&mut buf);

        let message = ContextMessage::AwaitAck(AwaitAck {
            action_id: tx_packet_to_action_id(&TxPacket::Pingreq(packet)),
            packet: buf,
            response_channel: sender,
        });

        self.sender.send(message).await?;

        if let RxPacket::Pingresp(_) = receiver.await? {
            return Ok(());
        }

        unreachable!("Unexpected packet type.");
    }

    pub async fn publish<'a>(&mut self, opts: PublishOpts<'a>) -> Result<(), MqttError> {
        match opts.qos.unwrap_or_default() {
            QoS::AtMostOnce => {
                let packet = opts.build()?;

                let mut buf = BytesMut::with_capacity(packet.packet_len());
                packet.encode(&mut buf);

                let message = ContextMessage::FireAndForget(buf);
                self.sender.send(message).await?;
                return Ok(());
            }
            QoS::AtLeastOnce => {
                let packet = opts.packet_identifier(self.make_packet_id()).build()?;

                let mut buf = BytesMut::with_capacity(packet.packet_len());
                packet.encode(&mut buf);

                let (sender, receiver) = oneshot::channel();

                let message = ContextMessage::AwaitAck(AwaitAck {
                    action_id: tx_packet_to_action_id(&TxPacket::Publish(packet)),
                    packet: buf,
                    response_channel: sender,
                });

                self.sender.send(message).await?;

                if let RxPacket::Puback(puback) = receiver.await? {
                    if puback.reason as u8 >= 0x80 {
                        return Err(PublishError::Puback(puback.reason.into()).into());
                    }

                    return Ok(());
                }

                unreachable!("Unexpected packet type.");
            }
            QoS::ExactlyOnce => {
                let packet = opts.packet_identifier(self.make_packet_id()).build()?;

                let mut buf = BytesMut::with_capacity(packet.packet_len());
                packet.encode(&mut buf);

                let (pub_sender, pub_receiver) = oneshot::channel();

                let pub_msg = ContextMessage::AwaitAck(AwaitAck {
                    action_id: tx_packet_to_action_id(&TxPacket::Publish(packet)),
                    packet: buf.split(),
                    response_channel: pub_sender,
                });

                self.sender.send(pub_msg).await?;

                if let RxPacket::Pubrec(pubrec) = pub_receiver.await? {
                    if pubrec.reason as u8 >= 0x80 {
                        return Err(PublishError::Pubrec(pubrec.reason.into()).into());
                    }

                    let (pubrel_sender, pubrel_receiver) = oneshot::channel();

                    let mut builder = PubrelTxBuilder::default();
                    builder.packet_identifier(pubrec.packet_identifier);

                    let pubrel = builder.build().unwrap();

                    buf.reserve(pubrel.packet_len());
                    pubrel.encode(&mut buf);

                    let pubrel_msg = ContextMessage::AwaitAck(AwaitAck {
                        action_id: tx_packet_to_action_id(&TxPacket::Pubrel(pubrel)),
                        packet: buf,
                        response_channel: pubrel_sender,
                    });

                    self.sender.send(pubrel_msg).await?;

                    if let RxPacket::Pubcomp(pubcomp) = pubrel_receiver.await? {
                        if pubcomp.reason as u8 >= 0x80 {
                            return Err(PublishError::Pubcomp(pubcomp.reason.into()).into());
                        }

                        return Ok(());
                    }
                }

                unreachable!("Unexpected packet type.");
            }
        }
    }

    /// Performs subscription to the topic specified in `opts`. This corresponds to sending the
    /// [Subscribe](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901161) packet.
    ///
    /// # Arguments
    /// `opts` - Subscription options
    pub async fn subscribe<'a>(
        &mut self,
        opts: SubscribeOpts<'a>,
    ) -> Result<SubscribeStream, MqttError> {
        let (sender, receiver) = oneshot::channel();
        let (str_sender, str_receiver) = mpsc::unbounded();

        let packet = opts
            .packet_identifier(self.make_packet_id())
            .subscription_identifier(self.make_subscription_id())
            .build()?;

        let subscription_identifier = NonZero::from(
            packet
                .subscription_identifier
                .clone()
                .expect("Subscription identifier missing."),
        )
        .get()
        .value();

        let mut buf = BytesMut::with_capacity(packet.packet_len());
        packet.encode(&mut buf);

        let message = ContextMessage::Subscribe(Subscribe {
            action_id: tx_packet_to_action_id(&TxPacket::Subscribe(packet)),
            subscription_identifier: subscription_identifier as usize,
            packet: buf,
            response_channel: sender,
            stream: str_sender,
        });

        self.sender.send(message).await?;

        if let RxPacket::Suback(suback) = receiver.await? {
            let reason = suback.payload.first().copied().unwrap();
            if reason as u8 >= 0x80 {
                return Err(SubscribeError { reason }.into());
            }

            return Ok(SubscribeStream {
                receiver: str_receiver,
            });
        }

        unreachable!("Unexpected packet type.");
    }

    /// Unsubscribes from the topic specified in `opts`. This corresponds to sending the
    /// [Unsubscribe](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901179) packet.
    ///
    /// # Arguments
    /// `opts` - Subscription options
    pub async fn unsubscribe<'a>(&mut self, opts: UnsubscribeOpts<'a>) -> Result<(), MqttError> {
        let (sender, receiver) = oneshot::channel();

        let packet = opts
            .packet_identifier(self.packet_id.fetch_add(1, Ordering::Relaxed))
            .build()?;

        let mut buf = BytesMut::with_capacity(packet.packet_len());
        packet.encode(&mut buf);

        let message = ContextMessage::AwaitAck(AwaitAck {
            action_id: tx_packet_to_action_id(&TxPacket::Unsubscribe(packet)),
            packet: buf,
            response_channel: sender,
        });

        self.sender.send(message).await?;

        if let RxPacket::Unsuback(unsuback) = receiver.await? {
            let reason = unsuback.payload.first().copied().unwrap();
            if reason as u8 >= 0x80 {
                return Err(UnsubscribeError { reason }.into());
            }

            return Ok(());
        }

        unreachable!("Unexpected packet type.");
    }
}
