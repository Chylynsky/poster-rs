use crate::{
    client::{
        error::{HandleClosed, MqttError, SocketClosed},
        fut::SubscribeStream,
        opts::{AuthOpts, ConnectOpts, PublishOpts, SubscribeOpts, UnsubscribeOpts},
        rsp::{AuthRsp, ConnectRsp, PublishRsp, SubscribeRsp, UnsubscribeRsp},
        streams::{RxPacketStream, TxPacketStream},
    },
    codec::*,
    core::{
        base_types::{NonZero, VarSizeInt},
        error::{CodecError, MandatoryPropertyMissing},
        properties::SubscriptionIdentifier,
        utils::{Encode, PacketID, SizedPacket},
    },
};
use bytes::{Bytes, BytesMut};
use core::{
    fmt,
    sync::atomic::{AtomicU16, AtomicU32, Ordering},
};
use either::{Either, Left, Right};
use futures::{
    channel::{
        mpsc::{self, SendError, TrySendError},
        oneshot::{self, Canceled},
    },
    AsyncRead, AsyncWrite, FutureExt, SinkExt, StreamExt,
};
use std::{collections::HashMap, error::Error, io, sync::Arc};

fn tx_packet_to_action_id(packet: &TxPacket) -> u32 {
    match packet {
        TxPacket::Connect(_) => 0,
        TxPacket::Auth(_) => 0,
        TxPacket::Subscribe(subscribe) => {
            ((SubackRx::PACKET_ID as u32) << 24) | ((subscribe.packet_identifier.get() as u32) << 8)
        }
        TxPacket::Unsubscribe(unsubscribe) => {
            ((UnsubackRx::PACKET_ID as u32) << 24)
                | ((unsubscribe.packet_identifier.get() as u32) << 8)
        }
        TxPacket::Pingreq(_) => (PingrespRx::PACKET_ID as u32) << 24,
        TxPacket::Publish(publish) => {
            (PubackRx::PACKET_ID as u32) << 24
                | (publish
                    .packet_identifier
                    .map(|val| -> u32 { val.get() as u32 })
                    .unwrap_or(0u32)
                    << 8)
        }
        TxPacket::Puback(puback) => {
            (PubackTx::PACKET_ID as u32) << 24
                | ((u16::from(puback.packet_identifier.get()) as u32) << 8)
        }

        // TxPacket::Pubrec(_) => (Pubrec::PACKET_ID as u32) << 24,
        // TxPacket::Pubrel(_) => (Pubrel::PACKET_ID as u32) << 24,
        // TxPacket::Pubcomp(_) => (Pubcomp::PACKET_ID as u32) << 24,
        _ => unreachable!("Unexpected packet type."),
    }
}

fn rx_packet_to_action_id(packet: &RxPacket) -> u32 {
    // Note that PUBLISH packet is ommited, it is handled as a subscription
    match packet {
        RxPacket::Connack(_) => 0,
        RxPacket::Auth(_) => 0,
        RxPacket::Suback(suback) => {
            ((SubackRx::PACKET_ID as u32) << 24) | ((suback.packet_identifier.get() as u32) << 8)
        }
        RxPacket::Unsuback(unsuback) => {
            ((UnsubackRx::PACKET_ID as u32) << 24)
                | ((unsuback.packet_identifier.get() as u32) << 8)
        }
        RxPacket::Pingresp(_) => (PingrespRx::PACKET_ID as u32) << 24,

        RxPacket::Puback(puback) => {
            (PubackRx::PACKET_ID as u32) << 24 | ((puback.packet_identifier.get() as u32) << 8)
        }

        // TxPacket::Pubrec(_) => (Pubrec::PACKET_ID as u32) << 24,
        // TxPacket::Pubrel(_) => (Pubrel::PACKET_ID as u32) << 24,
        // TxPacket::Pubcomp(_) => (Pubcomp::PACKET_ID as u32) << 24,
        _ => unreachable!("Unexpected packet type."),
    }
}

struct AwaitAck {
    action_id: u32,
    packet: BytesMut,
    response_channel: oneshot::Sender<RxPacket>,
}

struct Subscribe {
    action_id: u32,
    subscription_identifier: u32,
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

    active_requests: HashMap<u32, oneshot::Sender<RxPacket>>,
    active_subscriptions: HashMap<u32, mpsc::UnboundedSender<RxPacket>>,

    message_queue: mpsc::UnboundedReceiver<ContextMessage>,
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
                active_requests: HashMap::new(),
                active_subscriptions: HashMap::new(),
                message_queue: receiver,
            },
            ContextHandle {
                sender,
                packet_id: Arc::new(AtomicU16::from(1)),
                sub_id: Arc::new(AtomicU32::from(1)),
            },
        )
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

    /// TODO: Support higher than QoS 0
    pub async fn publish<'a>(
        &mut self,
        opts: PublishOpts<'a>,
    ) -> Result<Option<PublishRsp>, MqttError> {
        let packet = opts.build()?;

        let mut buf = BytesMut::with_capacity(packet.packet_len());
        packet.encode(&mut buf);

        let message = ContextMessage::FireAndForget(buf);

        self.sender.send(message).await?;

        Ok(None)
    }

    /// Performs subscription to the topic specified in `opts`. This corresponds to sending the
    /// [Subscribe](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901161) packet.
    ///
    /// # Arguments
    /// `opts` - Subscription options
    pub async fn subscribe<'a>(
        &mut self,
        opts: SubscribeOpts<'a>,
    ) -> Result<(SubscribeRsp, SubscribeStream), MqttError> {
        let (sender, receiver) = oneshot::channel();
        let (str_sender, str_receiver) = mpsc::unbounded();

        let packet = opts
            .packet_identifier(self.packet_id.fetch_add(1, Ordering::Relaxed))
            .subscription_identifier(self.sub_id.fetch_add(1, Ordering::Relaxed))
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
            subscription_identifier: subscription_identifier,
            packet: buf,
            response_channel: sender,
            stream: str_sender,
        });

        self.sender.send(message).await?;

        if let RxPacket::Suback(suback) = receiver.await? {
            return Ok((
                SubscribeRsp::from(suback),
                SubscribeStream {
                    receiver: str_receiver,
                },
            ));
        }

        unreachable!("Unexpected packet type.");
    }

    /// Unsubscribes from the topic specified in `opts`. This corresponds to sending the
    /// [Unsubscribe](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901179) packet.
    ///
    /// # Arguments
    /// `opts` - Subscription options
    pub async fn unsubscribe<'a>(
        &mut self,
        opts: UnsubscribeOpts<'a>,
    ) -> Result<UnsubscribeRsp, MqttError> {
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
            return Ok(UnsubscribeRsp::from(unsuback));
        }

        unreachable!("Unexpected packet type.");
    }
}

/// Makes [Context] object start processing MQTT traffic, blocking the current task/thread until
/// graceful disconnection or error. Successful disconnection via [disconnect] method or
/// receiving a [Disconnect](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901205)
/// packet with reason a code equal to 0 (success) is considered a graceful disconnection.
///
/// # Tokio example
/// ```
/// let ctx_task = tokio::spawn(async move {
///     poster::run(ctx).await; // Blocks until disconnection or critical error.
/// });
/// ```
pub async fn run<RxStreamT, TxStreamT>(
    ctx: &mut Context<RxStreamT, TxStreamT>,
) -> Result<(), MqttError>
where
    RxStreamT: AsyncRead + Unpin,
    TxStreamT: AsyncWrite + Unpin,
{
    let mut pck_fut = ctx.rx.next().fuse();
    let mut msg_fut = ctx.message_queue.next().fuse();

    loop {
        futures::select! {
            maybe_rx_packet = pck_fut => {
                match maybe_rx_packet {
                    Some(rx_packet) => match rx_packet? {
                        RxPacket::Publish(publish) => match publish.subscription_identifier {
                            Some(subscription_identifier) => {
                                let sub_id = NonZero::from(subscription_identifier).get().value();
                                if let Some(subscription) =  ctx.active_subscriptions.get_mut(&sub_id) {
                                    // User may drop the receiving stream,
                                    // in that case remove it from the active subscriptions map.
                                    if (subscription.send(RxPacket::Publish(publish)).await).is_err() {
                                        ctx.active_subscriptions.remove(&sub_id);
                                    }
                                }
                            },
                            None => return Err(CodecError::MandatoryPropertyMissing(MandatoryPropertyMissing).into()),
                        },
                        RxPacket::Disconnect(disconnect) => {
                            if disconnect.reason == DisconnectReason::Success  {
                                return Ok(()); // Graceful disconnection.
                            }

                            return Err(disconnect.into());
                        },
                        packet => {
                            let id = rx_packet_to_action_id(&packet);
                            if let Some(sender) = ctx.active_requests.remove(&id) {
                                // Error here indicates internal error, the receiver
                                // end is inside one of the ContextHandle method.
                                sender.send(packet).map_err(|_| HandleClosed)?;
                            }
                        }
                    }
                    None => return Err(SocketClosed.into()),
                }

                pck_fut = ctx.rx.next().fuse();
                continue;
            },
            maybe_msg = msg_fut => {
                match maybe_msg {
                    Some(msg) => match msg {
                        ContextMessage::Disconnect(packet) => {
                            ctx.tx.write(packet.freeze()).await?;
                            return Ok(()) // Graceful disconnection.
                        },
                        ContextMessage::FireAndForget(packet) => {
                            ctx.tx.write(packet.freeze()).await?;

                        },
                        ContextMessage::AwaitAck(msg) => {
                            ctx.active_requests.insert(msg.action_id, msg.response_channel);
                            ctx.tx.write(msg.packet.freeze()).await?;
                        },
                        ContextMessage::Subscribe(msg) => {
                            ctx.active_requests.insert(msg.action_id, msg.response_channel);
                            ctx.active_subscriptions.insert(msg.subscription_identifier, msg.stream);
                            ctx.tx.write(msg.packet.freeze()).await?;
                        },
                    }
                    None => return Err(HandleClosed.into()),
                }

                msg_fut = ctx.message_queue.next().fuse();
                continue;
            }
        }
    }
}
