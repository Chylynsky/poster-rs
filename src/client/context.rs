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
        utils::PacketID,
    },
};
use either::{Either, Left, Right};
use futures::{
    channel::{mpsc, oneshot},
    io::BufReader,
    AsyncRead, AsyncWrite, FutureExt, StreamExt,
};
use std::{
    collections::HashMap,
    mem,
    sync::{
        atomic::{AtomicU16, AtomicU32, Ordering},
        Arc,
    },
};

// TODO:
// Subscription streams
// Revisit error handling

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
        _ => panic!("Unexpected packet type."),
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
        _ => panic!("Unexpected packet type."),
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
    active_subscriptions: HashMap<u32, mpsc::Sender<RxPacket>>,

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
        let (sender, receiver) = mpsc::channel(16 * mem::size_of::<ContextMessage>());

        (
            Self {
                rx: RxPacketStream::from(BufReader::with_capacity(Self::DEFAULT_BUF_SIZE, rx)),
                tx: TxPacketStream::with_capacity(Self::DEFAULT_BUF_SIZE, tx),
                active_requests: HashMap::new(),
                active_subscriptions: HashMap::new(),
                message_queue: receiver,
            },
            ContextHandle {
                sender: sender,
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
    pub async fn connect(&mut self, opts: ConnectOpts) -> Option<Either<ConnectRsp, AuthRsp>> {
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

        receiver.await.ok().map(|packet| match packet {
            RxPacket::Connack(connack) => Left(ConnectRsp::from(connack)),
            RxPacket::Auth(auth) => Right(AuthRsp::from(auth)),
            _ => {
                panic!("Unexpected packet type.");
            }
        })
    }

    /// Method used for performing the extended authorization between the client and the broker. It corresponds to sending the
    /// [Auth](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901217) packet.
    /// User may perform multiple calls of this method, as needed, until the ConnectRsp is returned,
    /// meaning the authorization is successful.
    ///
    /// # Arguments
    /// `opts` - Authorization options
    pub async fn authorize(&mut self, opts: AuthOpts) -> Option<Either<ConnectRsp, AuthRsp>> {
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

        receiver.await.ok().map(|packet| match packet {
            RxPacket::Connack(connack) => Left(ConnectRsp::from(connack)),
            RxPacket::Auth(auth) => Right(AuthRsp::from(auth)),
            _ => {
                panic!("Unexpected packet type.");
            }
        })
    }

    /// Performs graceful disconnection with the broker by sending the
    /// [Disconnect](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901205) packet.
    pub async fn disconnect(&mut self) {
        let mut builder = DisconnectBuilder::default();
        builder.reason(DisconnectReason::Success);

        let packet = builder.build().unwrap();

        let message = ContextMessage::FireAndForget(FireAndForget {
            packet: TxPacket::Disconnect(packet),
        });

        self.sender
            .try_send(message)
            .expect("Error sending disconnect message to the context.");
    }

    /// Sends ping to the broker by sending
    /// [Ping](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901195) packet.
    pub async fn ping(&mut self) {
        let (sender, receiver) = oneshot::channel();

        let builder = PingreqBuilder::default();
        let packet = builder.build().unwrap();
        let tx_packet = TxPacket::Pingreq(packet);

        let message = ContextMessage::AwaitAck(AwaitAck {
            action_id: tx_packet_to_action_id(&tx_packet),
            packet: tx_packet,
            response_channel: sender,
        });

        self.sender
            .try_send(message)
            .expect("Error sending ping request to the context.");

        receiver.await.ok().map(|packet| match packet {
            RxPacket::Pingresp(_) => return,
            _ => {
                panic!("Unexpected packet type.");
            }
        });
    }

    /// TODO: Support higher than QoS 0
    pub async fn publish(&mut self, opts: PublishOpts) -> Option<PublishRsp> {
        let packet = opts.build().unwrap();

        let message = ContextMessage::FireAndForget(FireAndForget {
            packet: TxPacket::Publish(packet),
        });

        self.sender
            .try_send(message)
            .expect("Error sending publish message to the context.");

        None
    }

    /// Performs subscription to the topic specified in `opts`. This corresponds to sending the
    /// [Subscribe](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901161) packet.
    ///
    /// # Arguments
    /// `opts` - Subscription options
    pub async fn subscribe(
        &mut self,
        opts: SubscribeOpts,
    ) -> Option<(SubscribeRsp, SubscribeStream)> {
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

        self.sender
            .try_send(message)
            .expect("Error sending subscribe request to the context.");

        receiver.await.ok().map(|packet| match packet {
            RxPacket::Suback(suback) => {
                return (
                    SubscribeRsp::from(suback),
                    SubscribeStream {
                        receiver: str_receiver,
                    },
                )
            }
            _ => {
                panic!("Unexpected packet type.");
            }
        })
    }

    /// Unsubscribes from the topic specified in `opts`. This corresponds to sending the
    /// [Unsubscribe](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901179) packet.
    ///
    /// # Arguments
    /// `opts` - Subscription options
    pub async fn unsubscribe(&mut self, opts: UnsubscribeOpts) -> Option<UnsubscribeRsp> {
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

        self.sender
            .try_send(message)
            .expect("Error sending unsubscribe request to the context.");

        receiver.await.ok().map(|packet| match packet {
            RxPacket::Unsuback(unsuback) => UnsubscribeRsp::from(unsuback),
            _ => {
                panic!("Unexpected packet type.");
            }
        })
    }
}

/// Makes [Context] object start processing MQTT traffic, blocking the current task/thread until
/// graceful disconnection or critical error.
///
/// # Tokio example
/// ```
/// let ctx_task = tokio::spawn(async move {
///     poster::run(ctx).await; // Blocks until disconnection or critical error.
/// });
/// ```
pub async fn run<RxStreamT, TxStreamT>(mut ctx: Context<RxStreamT, TxStreamT>)
where
    RxStreamT: AsyncRead + Unpin,
    TxStreamT: AsyncWrite + Unpin,
{
    let mut pck_fut = ctx.rx.next().fuse();
    let mut msg_fut = ctx.message_queue.next().fuse();

    loop {
        futures::select! {
            rx_packet_result = pck_fut => {
                if rx_packet_result.is_none() {
                    // Socket closed
                    eprintln!("Socket closed.");
                    return;
                }

                let rx_packet = rx_packet_result.unwrap();

                if let RxPacket::Publish(publish) = &rx_packet {
                    if  publish.subscription_identifier.is_none() {
                        continue;
                    }

                    let sub_id: NonZero<VarSizeInt> = publish.subscription_identifier.clone().unwrap().into();
                    let subscription = ctx.active_subscriptions.get_mut(&sub_id.value().into()).expect("Subscription identifier not found.");

                    subscription.try_send(rx_packet).expect("Failed to pass data to suscription.");
                    continue;
                }

                let id = rx_packet_to_action_id(&rx_packet);

                if let Some(sender) = ctx.active_requests.remove(&id) {
                    sender.send(rx_packet).ok().expect("Failed to pass response to the context handle.");
                }

                pck_fut = ctx.rx.next().fuse();
                continue;
            },
            message_result = msg_fut => {
                if let Some(msg_type) = message_result {
                    match msg_type {
                        ContextMessage::FireAndForget(msg) => {
                            ctx.tx.write(msg.packet).await.expect("Failed to publish packet.");
                        },
                        ContextMessage::AwaitAck(msg) => {
                            ctx.active_requests.insert(msg.action_id, msg.response_channel);
                            ctx.tx.write(msg.packet).await.expect("Failed to publish packet.");
                        },
                        ContextMessage::AwaitStream(msg) => {
                            ctx.active_requests.insert(msg.action_id, msg.response_channel);
                            ctx.tx.write(msg.packet).await.expect("Failed to publish packet.");
                        },
                    }

                    msg_fut = ctx.message_queue.next().fuse();
                    continue;
                }

                eprintln!("MESSAGE QUEUE CLOSED");
                return; // CRITICAL ERROR - message queue closed
            }
        }
    }
}
