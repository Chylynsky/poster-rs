use crate::{
    client::{
        error::{HandleClosed, MaximumPacketSizeExceeded, MqttError, SocketClosed},
        handle::ContextHandle,
        message::*,
        utils::*,
    },
    codec::*,
    core::{
        base_types::NonZero,
        error::{CodecError, MandatoryPropertyMissing},
        properties::ReceiveMaximum,
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
use std::{collections::VecDeque, sync::Arc, time::SystemTime};

struct Session {
    awaiting_ack: VecDeque<(usize, oneshot::Sender<RxPacket>)>,
    subscriptions: VecDeque<(usize, mpsc::UnboundedSender<RxPacket>)>,
    retrasmit_queue: VecDeque<(usize, Bytes)>,
    unsent: VecDeque<ContextMessage>,
}

struct Connection {
    disconnection_timestamp: Option<SystemTime>,
    session_expiry_interval: u32,

    remote_receive_maximum: u16,
    send_quota: u16,

    remote_max_packet_size: Option<u32>,
}

/// Client context. Responsible for socket management and direct communication with the broker.
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
    /// Creates a new [Context] instance, paired with [ContextHandle].
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
                rx: RxPacketStream::from(rx),
                tx: TxPacketStream::from(tx),
                message_queue: receiver,
                session: Session {
                    awaiting_ack: VecDeque::new(),
                    subscriptions: VecDeque::new(),
                    retrasmit_queue: VecDeque::new(),
                    unsent: VecDeque::new(),
                },
                connection: Connection {
                    disconnection_timestamp: None,
                    session_expiry_interval: 0,
                    remote_receive_maximum: u16::from(NonZero::from(ReceiveMaximum::default())),
                    send_quota: u16::from(NonZero::from(ReceiveMaximum::default())),
                    remote_max_packet_size: None,
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

        if connection.session_expiry_interval == 0 {
            return true;
        }

        if connection.session_expiry_interval == u32::MAX {
            return false;
        }

        let elapsed = connection
            .disconnection_timestamp
            .map(|timestamp| timestamp.elapsed().unwrap())
            .map(|elapsed| elapsed.as_secs())
            .map(|elapsed| {
                if elapsed > u32::MAX as u64 {
                    u32::MAX
                } else {
                    elapsed as u32
                }
            })
            .unwrap();

        connection.session_expiry_interval >= elapsed
    }

    fn reset_session(session: &mut Session) {
        session.awaiting_ack.clear();
        session.subscriptions.clear();
        session.retrasmit_queue.clear();
    }

    fn validate_packet_size(connection: &Connection, packet: &[u8]) -> Result<(), MqttError> {
        if connection.remote_max_packet_size.is_none()
            || packet.len() <= connection.remote_max_packet_size.unwrap() as usize
        {
            Ok(())
        } else {
            Err(MaximumPacketSizeExceeded.into())
        }
    }

    async fn handle_message(
        tx: &mut TxPacketStream<TxStreamT>,
        connection: &mut Connection,
        session: &mut Session,
        msg: ContextMessage,
    ) -> Result<(), MqttError> {
        match msg {
            ContextMessage::Connect(msg) => {
                Self::validate_packet_size(connection, msg.packet.as_ref())?;
                tx.write(msg.packet.as_ref()).await?;
                connection.session_expiry_interval = msg.session_expiry_interval;
                session
                    .awaiting_ack
                    .push_back((msg.action_id, msg.response_channel));
            }
            ContextMessage::Disconnect(packet) => {
                Self::validate_packet_size(connection, packet.as_ref())?;
                tx.write(packet.freeze().as_ref()).await?;
                // Graceful disconnection.
            }
            ContextMessage::FireAndForget(packet) => {
                Self::validate_packet_size(connection, packet.as_ref())?;
                tx.write(packet.freeze().as_ref()).await?;
            }
            ContextMessage::AwaitAck(mut msg) => {
                Self::validate_packet_size(connection, msg.packet.as_ref())?;

                let packet_id = msg.packet.get(0).unwrap() >> 4; // Extract packet id, being the four MSB bits

                if packet_id == PublishTx::PACKET_ID {
                    if connection.send_quota == 0 {
                        session.unsent.push_back(ContextMessage::AwaitAck(msg));
                        return Ok(());
                    }

                    connection.send_quota -= 1;

                    tx.write(msg.packet.as_ref()).await?;

                    let fixed_hdr = msg.packet.get_mut(0).unwrap();
                    *fixed_hdr |= (1 << 3) as u8; // Set DUP flag in the PUBLISH fixed header

                    session
                        .awaiting_ack
                        .push_back((msg.action_id, msg.response_channel));

                    session
                        .retrasmit_queue
                        .push_back((msg.action_id, msg.packet.freeze()));
                } else if packet_id == PubrelTx::PACKET_ID {
                    tx.write(msg.packet.as_ref()).await?;
                    session
                        .awaiting_ack
                        .push_back((msg.action_id, msg.response_channel));

                    session
                        .retrasmit_queue
                        .push_back((msg.action_id, msg.packet.freeze()));
                } else {
                    tx.write(msg.packet.as_ref()).await?;
                    session
                        .awaiting_ack
                        .push_back((msg.action_id, msg.response_channel));
                }
            }
            ContextMessage::Subscribe(msg) => {
                Self::validate_packet_size(connection, msg.packet.as_ref())?;
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

    async fn handle_packet(
        tx: &mut TxPacketStream<TxStreamT>,
        connection: &mut Connection,
        session: &mut Session,
        packet: RxPacket,
    ) -> Result<(), MqttError> {
        match packet {
            RxPacket::Connack(connack) => {
                if connack.session_expiry_interval.is_some() {
                    connection.session_expiry_interval =
                        connack.session_expiry_interval.map(u32::from).unwrap();
                }

                if connack.maximum_packet_size.is_some() {
                    connection.remote_max_packet_size = connack
                        .maximum_packet_size
                        .map(NonZero::from)
                        .map(u32::from);
                }

                connection.remote_receive_maximum =
                    u16::from(NonZero::from(connack.receive_maximum));
                connection.send_quota = connection.remote_receive_maximum;
            }
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
            RxPacket::Puback(puback) => {
                let rx_packet = RxPacket::Puback(puback);
                let action_id = rx_action_id(&rx_packet);

                if connection.send_quota != connection.remote_receive_maximum {
                    let maybe_unsent = session.unsent.pop_front();
                    if maybe_unsent.is_some() {
                        Self::handle_message(tx, connection, session, maybe_unsent.unwrap())
                            .await?;
                    }

                    connection.send_quota += 1;
                }

                linear_search_by_key(&session.retrasmit_queue, action_id)
                    .and_then(|pos| session.retrasmit_queue.remove(pos));

                if let Some((_, sender)) = linear_search_by_key(&session.awaiting_ack, action_id)
                    .and_then(|pos| session.awaiting_ack.remove(pos))
                {
                    // Error here indicates internal error, the receiver
                    // end is inside one of the ContextHandle methods.
                    sender.send(rx_packet).map_err(|_| HandleClosed)?;
                }
            }
            RxPacket::Pubcomp(pubcomp) => {
                let rx_packet = RxPacket::Pubcomp(pubcomp);
                let action_id = rx_action_id(&rx_packet);

                if connection.send_quota != connection.remote_receive_maximum {
                    let maybe_unsent = session.unsent.pop_front();
                    if maybe_unsent.is_some() {
                        Self::handle_message(tx, connection, session, maybe_unsent.unwrap())
                            .await?;
                    }

                    connection.send_quota += 1;
                }

                linear_search_by_key(&session.retrasmit_queue, action_id)
                    .and_then(|pos| session.retrasmit_queue.remove(pos));

                if let Some((_, sender)) = linear_search_by_key(&session.awaiting_ack, action_id)
                    .and_then(|pos| session.awaiting_ack.remove(pos))
                {
                    // Error here indicates internal error, the receiver
                    // end is inside one of the ContextHandle methods.
                    sender.send(rx_packet).map_err(|_| HandleClosed)?;
                }
            }
            other => {
                let action_id = rx_action_id(&other);

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

        for (_, packet) in session.retrasmit_queue.iter() {
            tx.write(packet.as_ref()).await?;
        }

        Ok(())
    }

    /// Starts processing MQTT traffic, blocking (on .await) the current task until
    /// graceful disconnection or error. Successful disconnection via [disconnect](ContextHandle::disconnect) method or
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
                Self::reset_session(session);
            }

            Self::retransmit(tx, connection, session).await?;
        }

        let mut pck_fut = rx.next().fuse();
        let mut msg_fut = message_queue.next().fuse();

        loop {
            futures::select! {
                maybe_rx_packet = pck_fut => {
                    let rx_packet = maybe_rx_packet.ok_or(SocketClosed)?;
                    Self::handle_packet(tx, connection, session, rx_packet?).await?;
                    pck_fut = rx.next().fuse();
                },
                maybe_msg = msg_fut => {
                    let msg = maybe_msg.ok_or(HandleClosed)?;
                    Self::handle_message(tx, connection, session, msg).await?;
                    msg_fut = message_queue.next().fuse();
                }
            }
        }
    }
}
