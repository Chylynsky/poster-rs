use crate::{
    client::{
        fut::SubscribeStream,
        opts::{AuthOpts, ConnectOpts, PublishOpts, SubscribeOpts, UnsubscribeOpts},
        rsp::{AuthRsp, ConnectRsp, PublishRsp, SubscribeRsp, UnsubscribeRsp},
        streams::{RxPacketStream, TxPacketStream},
    },
    codec::*,
    core::{
        base_types::{NonZero, VarSizeInt},
        error::CodecError,
        properties::SubscriptionIdentifier,
        utils::PacketID,
    },
};
use core::{
    fmt, mem,
    sync::atomic::{AtomicU16, AtomicU32, Ordering},
};
use either::{Either, Left, Right};
use futures::{
    channel::{
        mpsc::{self, SendError, TrySendError},
        oneshot::{self, Canceled},
    },
    io::BufReader,
    AsyncRead, AsyncWrite, FutureExt, SinkExt, StreamExt,
};
use std::{collections::HashMap, error::Error, io, sync::Arc};

#[derive(Debug, Clone)]
pub struct SocketClosed;

impl fmt::Display for SocketClosed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "socket closed")
    }
}

impl Error for SocketClosed {}

impl From<io::Error> for SocketClosed {
    fn from(_: io::Error) -> Self {
        Self
    }
}

#[derive(Debug, Clone)]
pub struct HandleClosed;

impl fmt::Display for HandleClosed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "context handle closed")
    }
}

impl Error for HandleClosed {}

impl From<Canceled> for HandleClosed {
    fn from(_: Canceled) -> Self {
        Self
    }
}

impl From<SendError> for HandleClosed {
    fn from(_: SendError) -> Self {
        Self
    }
}

#[derive(Debug, Clone)]
pub struct ContextExited;

impl fmt::Display for ContextExited {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "context exited")
    }
}

impl Error for ContextExited {}

impl From<TrySendError<ContextMessage>> for ContextExited {
    fn from(_: TrySendError<ContextMessage>) -> Self {
        Self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Disconnected {
    pub reason: DisconnectReason,
}

impl fmt::Display for Disconnected {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Disconnected with reason: {} [{:?}]",
            self.reason as u8, self.reason
        )
    }
}

impl Error for Disconnected {}

impl From<Disconnect> for Disconnected {
    fn from(packet: Disconnect) -> Self {
        Self {
            reason: packet.reason,
        }
    }
}

impl From<DisconnectReason> for Disconnected {
    fn from(reason: DisconnectReason) -> Self {
        Self { reason }
    }
}

#[derive(Debug, Clone)]
pub struct InternalError {
    msg: String,
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InternalError: {}", self.msg)
    }
}

impl Error for InternalError {}

impl From<&str> for InternalError {
    fn from(s: &str) -> Self {
        Self {
            msg: String::from(s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MqttError {
    InternalError(InternalError),
    SocketClosed(SocketClosed),
    HandleClosed(HandleClosed),
    ContextExited(ContextExited),
    CodecError(CodecError),
    Disconnected(Disconnected),
}

impl fmt::Display for MqttError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InternalError(err) => write!(f, "MqttError: {}", err),
            Self::SocketClosed(err) => write!(f, "MqttError: {}", err),
            Self::HandleClosed(err) => write!(f, "MqttError: {}", err),
            Self::ContextExited(err) => write!(f, "MqttError: {}", err),
            Self::CodecError(err) => write!(f, "MqttError: {}", err),
            Self::Disconnected(err) => write!(f, "MqttError: {}", err),
        }
    }
}

impl Error for MqttError {}

impl From<InternalError> for MqttError {
    fn from(err: InternalError) -> Self {
        Self::InternalError(err)
    }
}

impl From<SocketClosed> for MqttError {
    fn from(err: SocketClosed) -> Self {
        Self::SocketClosed(err)
    }
}

impl From<io::Error> for MqttError {
    fn from(err: io::Error) -> Self {
        Self::SocketClosed(err.into())
    }
}

impl From<HandleClosed> for MqttError {
    fn from(err: HandleClosed) -> Self {
        Self::HandleClosed(err)
    }
}

impl From<Canceled> for MqttError {
    fn from(err: Canceled) -> Self {
        Self::HandleClosed(err.into())
    }
}

impl From<SendError> for MqttError {
    fn from(err: SendError) -> Self {
        Self::HandleClosed(err.into())
    }
}

impl From<ContextExited> for MqttError {
    fn from(err: ContextExited) -> Self {
        Self::ContextExited(err)
    }
}

impl From<TrySendError<ContextMessage>> for MqttError {
    fn from(err: TrySendError<ContextMessage>) -> Self {
        Self::ContextExited(err.into())
    }
}

impl From<CodecError> for MqttError {
    fn from(err: CodecError) -> Self {
        Self::CodecError(err)
    }
}

impl From<Disconnected> for MqttError {
    fn from(err: Disconnected) -> Self {
        Self::Disconnected(err)
    }
}

impl From<Disconnect> for MqttError {
    fn from(packet: Disconnect) -> Self {
        Self::Disconnected(packet.into())
    }
}

impl From<DisconnectReason> for MqttError {
    fn from(reason: DisconnectReason) -> Self {
        Self::Disconnected(reason.into())
    }
}

fn tx_packet_to_action_id(packet: &TxPacket) -> u32 {
    match packet {
        TxPacket::Connect(_) => 0,
        TxPacket::Auth(_) => 0,
        TxPacket::Subscribe(subscribe) => {
            ((Suback::PACKET_ID as u32) << 24)
                | ((u16::from(subscribe.packet_identifier) as u32) << 8)
        }
        TxPacket::Unsubscribe(unsubscribe) => {
            ((Unsuback::PACKET_ID as u32) << 24)
                | ((u16::from(unsubscribe.packet_identifier) as u32) << 8)
        }
        TxPacket::Pingreq(_) => (Pingresp::PACKET_ID as u32) << 24,
        TxPacket::Publish(publish) => {
            (Puback::PACKET_ID as u32) << 24
                | (publish
                    .packet_identifier
                    .map(|val| -> u32 { u16::from(val) as u32 })
                    .unwrap_or(0u32)
                    << 8)
        }
        TxPacket::Puback(puback) => {
            (Puback::PACKET_ID as u32) << 24 | ((u16::from(puback.packet_identifier) as u32) << 8)
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
            ((Suback::PACKET_ID as u32) << 24) | ((u16::from(suback.packet_identifier) as u32) << 8)
        }
        RxPacket::Unsuback(unsuback) => {
            ((Unsuback::PACKET_ID as u32) << 24)
                | ((u16::from(unsuback.packet_identifier) as u32) << 8)
        }
        RxPacket::Pingresp(_) => (Pingresp::PACKET_ID as u32) << 24,

        RxPacket::Puback(puback) => {
            (Puback::PACKET_ID as u32) << 24 | ((u16::from(puback.packet_identifier) as u32) << 8)
        }

        // TxPacket::Pubrec(_) => (Pubrec::PACKET_ID as u32) << 24,
        // TxPacket::Pubrel(_) => (Pubrel::PACKET_ID as u32) << 24,
        // TxPacket::Pubcomp(_) => (Pubcomp::PACKET_ID as u32) << 24,
        _ => unreachable!("Unexpected packet type."),
    }
}

struct FireAndForget {
    packet: TxPacket,
}

struct AwaitAck {
    action_id: u32,
    packet: TxPacket,
    response_channel: oneshot::Sender<RxPacket>,
}

struct AwaitStream {
    action_id: u32,
    packet: TxPacket,
    response_channel: oneshot::Sender<RxPacket>,
    stream: mpsc::UnboundedSender<RxPacket>,
}

enum ContextMessage {
    FireAndForget(FireAndForget),
    AwaitAck(AwaitAck),
    AwaitStream(AwaitStream),
}

/// Client context. It is responsible for socket management and direct communication with the broker.
pub struct Context<RxStreamT, TxStreamT> {
    rx: RxPacketStream<RxStreamT>,
    tx: TxPacketStream<TxStreamT>,

    active_requests: HashMap<u32, oneshot::Sender<RxPacket>>,
    active_subscriptions: HashMap<u32, mpsc::UnboundedSender<RxPacket>>,

    message_queue: mpsc::Receiver<ContextMessage>,
}

impl<RxStreamT, TxStreamT> Context<RxStreamT, TxStreamT> {
    pub const DEFAULT_BUF_SIZE: usize = 1024;
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
        let (sender, receiver) = mpsc::channel(mem::size_of::<ContextMessage>());

        (
            Self {
                rx: RxPacketStream::from(BufReader::with_capacity(Self::DEFAULT_BUF_SIZE, rx)),
                tx: TxPacketStream::with_capacity(Self::DEFAULT_BUF_SIZE, tx),
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
    sender: mpsc::Sender<ContextMessage>,
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
    pub async fn connect(
        &mut self,
        opts: ConnectOpts,
    ) -> Result<Either<ConnectRsp, AuthRsp>, MqttError> {
        let (sender, receiver) = oneshot::channel();

        let packet = opts
            .build()
            .expect("Invalid configuration found in ConnectOpts.");
        let tx_packet = TxPacket::Connect(packet);

        let message = ContextMessage::AwaitAck(AwaitAck {
            action_id: tx_packet_to_action_id(&tx_packet),
            packet: tx_packet,
            response_channel: sender,
        });

        self.sender
            .try_send(message)
            .expect("Error sending connect request to the context.");

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
    pub async fn authorize(
        &mut self,
        opts: AuthOpts,
    ) -> Result<Either<ConnectRsp, AuthRsp>, MqttError> {
        let (sender, receiver) = oneshot::channel();

        let packet = opts
            .build()
            .expect("Invalid configuration found in AuthOpts.");
        let tx_packet = TxPacket::Auth(packet);

        let message = ContextMessage::AwaitAck(AwaitAck {
            action_id: tx_packet_to_action_id(&tx_packet),
            packet: tx_packet,
            response_channel: sender,
        });

        self.sender
            .try_send(message)
            .expect("Error sending connect request to the context.");

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
        let mut builder = DisconnectBuilder::default();
        builder.reason(DisconnectReason::Success);

        let packet = builder.build().unwrap();

        let message = ContextMessage::FireAndForget(FireAndForget {
            packet: TxPacket::Disconnect(packet),
        });

        self.sender.try_send(message)?;
        Ok(())
    }

    /// Sends ping to the broker by sending
    /// [Ping](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901195) packet.
    pub async fn ping(&mut self) -> Result<(), MqttError> {
        let (sender, receiver) = oneshot::channel();

        let builder = PingreqBuilder::default();
        let packet = builder.build().unwrap();
        let tx_packet = TxPacket::Pingreq(packet);

        let message = ContextMessage::AwaitAck(AwaitAck {
            action_id: tx_packet_to_action_id(&tx_packet),
            packet: tx_packet,
            response_channel: sender,
        });

        self.sender.try_send(message)?;

        if let RxPacket::Pingresp(_) = receiver.await? {
            return Ok(());
        }

        unreachable!("Unexpected packet type.");
    }

    /// TODO: Support higher than QoS 0
    pub async fn publish(&mut self, opts: PublishOpts) -> Result<Option<PublishRsp>, MqttError> {
        let packet = opts.build().unwrap();

        let message = ContextMessage::FireAndForget(FireAndForget {
            packet: TxPacket::Publish(packet),
        });

        self.sender
            .try_send(message)
            .map_err(|_| MqttError::from(ContextExited))?;

        Ok(None)
    }

    /// Performs subscription to the topic specified in `opts`. This corresponds to sending the
    /// [Subscribe](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901161) packet.
    ///
    /// # Arguments
    /// `opts` - Subscription options
    pub async fn subscribe(
        &mut self,
        opts: SubscribeOpts,
    ) -> Result<(SubscribeRsp, SubscribeStream), MqttError> {
        let (sender, receiver) = oneshot::channel();
        let (str_sender, str_receiver) = mpsc::unbounded();

        let packet = opts
            .packet_identifier(self.packet_id.fetch_add(1, Ordering::Relaxed))
            .subscription_identifier(self.sub_id.fetch_add(1, Ordering::Relaxed))
            .build()
            .expect("Invalid options found in SubscribeOpts.");
        let tx_packet = TxPacket::Subscribe(packet);

        let message = ContextMessage::AwaitStream(AwaitStream {
            action_id: tx_packet_to_action_id(&tx_packet),
            packet: tx_packet,
            response_channel: sender,
            stream: str_sender,
        });

        self.sender.try_send(message)?;

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
    pub async fn unsubscribe(
        &mut self,
        opts: UnsubscribeOpts,
    ) -> Result<UnsubscribeRsp, MqttError> {
        let (sender, receiver) = oneshot::channel();

        let packet = opts
            .packet_identifier(self.packet_id.fetch_add(1, Ordering::Relaxed))
            .build()
            .expect("Invalid options found in UnsubscribeOpts.");
        let tx_packet = TxPacket::Unsubscribe(packet);

        let message = ContextMessage::AwaitAck(AwaitAck {
            action_id: tx_packet_to_action_id(&tx_packet),
            packet: tx_packet,
            response_channel: sender,
        });

        self.sender.try_send(message)?;

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
    let unwrap_sub_id = |property: SubscriptionIdentifier| -> u32 {
        let tmp: NonZero<VarSizeInt> = property.into();
        let sub_id: VarSizeInt = tmp.into();
        sub_id.into()
    };

    let mut pck_fut = ctx.rx.next().fuse();
    let mut msg_fut = ctx.message_queue.next().fuse();

    loop {
        futures::select! {
            rx_packet_result = pck_fut => {
                if rx_packet_result.is_none() {
                   return Err(SocketClosed.into());
                }

                let rx_packet = rx_packet_result.unwrap()?;
                match rx_packet {
                    RxPacket::Publish(publish) => {
                        if  publish.subscription_identifier.is_some() {
                            let sub_id = unwrap_sub_id(publish.subscription_identifier.clone().unwrap());
                            if let Some(subscription) =  ctx.active_subscriptions.get_mut(&sub_id) {
                                // User may drop the receiving stream,
                                // in that case remove it from the active subscriptions map.
                                if (subscription.send(RxPacket::Publish(publish)).await).is_err() {
                                    ctx.active_subscriptions.remove(&sub_id);
                                }
                            }
                        }
                    },
                    RxPacket::Disconnect(disconnect) => {
                        if disconnect.reason == DisconnectReason::Success  {
                            return Ok(()); // Graceful disconnection.
                        }

                        return Err(disconnect.into());
                    },
                    _ => {
                        let id = rx_packet_to_action_id(&rx_packet);
                        if let Some(sender) = ctx.active_requests.remove(&id) {
                            // Error here indicates internal error, the receiver
                            // end is inside one of the ContextHandle method.
                            sender.send(rx_packet).map_err(|_| HandleClosed)?;
                        }
                    }
                }

                pck_fut = ctx.rx.next().fuse();
                continue;
            },
            message_result = msg_fut => {
                if message_result.is_none() {
                    return Err(HandleClosed.into());
                }

                match message_result.unwrap() {
                    ContextMessage::FireAndForget(msg) => {
                        match msg.packet {
                            TxPacket::Disconnect(_) => {
                                ctx.tx.write(msg.packet).await?;
                                return Ok(()) // Graceful disconnection.
                            }
                            _ => {
                                ctx.tx.write(msg.packet).await?;
                            }
                        }

                    },
                    ContextMessage::AwaitAck(msg) => {
                        ctx.active_requests.insert(msg.action_id, msg.response_channel);
                        ctx.tx.write(msg.packet).await?;
                    },
                    ContextMessage::AwaitStream(msg) => {
                        match msg.packet {
                            TxPacket::Subscribe(sub) => {
                                let sub_id = unwrap_sub_id(sub.properties.subscription_identifier.clone().unwrap());

                                ctx.active_requests.insert(msg.action_id, msg.response_channel);
                                ctx.active_subscriptions.insert(sub_id, msg.stream);
                                ctx.tx.write(TxPacket::Subscribe(sub)).await?;
                            },
                            _ => {
                                return Err(InternalError::from("unexpected packet type").into());
                            }
                        }
                    },
                }

                msg_fut = ctx.message_queue.next().fuse();
                continue;
            }
        }
    }
}
