use crate::{
    client::{
        error::{HandleClosed, MaximumPacketSizeExceeded, MqttError, SocketClosed},
        handle::ContextHandle,
        message::*,
        opts::{AuthOpts, ConnectOpts},
        rsp::{AuthRsp, ConnectRsp},
        utils::*,
    },
    codec::*,
    core::{
        base_types::NonZero,
        error::{CodecError, MandatoryPropertyMissing},
        properties::ReceiveMaximum,
        utils::{Encode, PacketID, SizedPacket},
    },
    io::{RxPacketStream, TxPacketStream},
};
use bytes::{Bytes, BytesMut};
use core::sync::atomic::{AtomicU16, AtomicU32};
use either::{Either, Left, Right};
use futures::{
    channel::{mpsc, oneshot},
    AsyncRead, AsyncWrite, FutureExt, StreamExt,
};
use std::{collections::VecDeque, sync::Arc, time::SystemTime};

use super::error::{InternalError, QuotaExceeded};

struct Session {
    awaiting_ack: VecDeque<(usize, oneshot::Sender<RxPacket>)>,
    subscriptions: VecDeque<(usize, mpsc::UnboundedSender<RxPacket>)>,
    retrasmit_queue: VecDeque<(usize, Bytes)>,
}

struct Connection {
    disconnection_timestamp: Option<SystemTime>,
    session_expiry_interval: u32,
    remote_receive_maximum: u16,
    remote_max_packet_size: Option<u32>,
    send_quota: u16,
}

/// Client context. Responsible for socket management and direct communication with the broker.
///
pub struct Context<RxStreamT, TxStreamT> {
    rx: Option<RxPacketStream<RxStreamT>>,
    tx: Option<TxPacketStream<TxStreamT>>,

    message_queue: mpsc::UnboundedReceiver<ContextMessage>,

    session: Session,
    connection: Connection,
}

impl<RxStreamT, TxStreamT> Context<RxStreamT, TxStreamT>
where
    RxStreamT: AsyncRead + Unpin,
    TxStreamT: AsyncWrite + Unpin,
{
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
                        return Err(QuotaExceeded.into());
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
        connection: &mut Connection,
        session: &mut Session,
        packet: RxPacket,
    ) -> Result<(), MqttError> {
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
                        if (subscription.unbounded_send(RxPacket::Publish(publish))).is_err() {
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
                    connection.send_quota += 1;
                }

                linear_search_by_key(&session.retrasmit_queue, action_id)
                    .and_then(|pos| session.retrasmit_queue.remove(pos));

                if let Some((_, sender)) = linear_search_by_key(&session.awaiting_ack, action_id)
                    .and_then(|pos| session.awaiting_ack.remove(pos))
                {
                    sender
                        .send(rx_packet)
                        .map_err(|_| InternalError::from("Unable to complete async operation."))?;
                }
            }
            other => {
                let action_id = rx_action_id(&other);

                if let Some((_, sender)) = linear_search_by_key(&session.awaiting_ack, action_id)
                    .and_then(|pos| session.awaiting_ack.remove(pos))
                {
                    sender
                        .send(other)
                        .map_err(|_| InternalError::from("Unable to complete async operation."))?;
                }
            }
        }

        Ok(())
    }

    fn handle_connack(connection: &mut Connection, connack: &ConnackRx) {
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

        connection.remote_receive_maximum = u16::from(NonZero::from(connack.receive_maximum));
        connection.send_quota = connection.remote_receive_maximum;
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

    /// Creates a new [Context] instance, paired with [ContextHandle].
    ///
    pub fn new() -> (Self, ContextHandle) {
        let (sender, receiver) = mpsc::unbounded();

        (
            Self {
                rx: None,
                tx: None,
                message_queue: receiver,

                session: Session {
                    awaiting_ack: VecDeque::new(),
                    subscriptions: VecDeque::new(),
                    retrasmit_queue: VecDeque::new(),
                },
                connection: Connection {
                    disconnection_timestamp: None,
                    session_expiry_interval: 0,
                    remote_receive_maximum: u16::from(NonZero::from(ReceiveMaximum::default())),
                    remote_max_packet_size: None,
                    send_quota: u16::from(NonZero::from(ReceiveMaximum::default())),
                },
            },
            ContextHandle {
                sender,
                packet_id: Arc::new(AtomicU16::from(1)),
                sub_id: Arc::new(AtomicU32::from(1)),
            },
        )
    }

    /// Sets up communication primitives for the context. This is the first method
    /// to call when starting the connection with the broker.
    ///
    /// # Arguments
    /// * `rx` - Read half of the stream, must be [AsyncRead] + [Unpin].
    /// * `tx` - Write half of the stream, must be [AsyncWrite] + [Unpin].
    ///
    /// # Note
    /// Calling any other member function before prior call to [set_up](Context::set_up) will panic.
    ///
    pub fn set_up(&mut self, (rx, tx): (RxStreamT, TxStreamT)) -> &mut Self {
        self.rx = Some(RxPacketStream::from(rx));
        self.tx = Some(TxPacketStream::from(tx));
        self
    }

    /// Performs connection with the broker on the protocol level. Calling this method corresponds to sending the
    /// [Connect](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901033) packet.
    ///
    /// If [authentication_method](ConnectOpts::authentication_method) and [authentication_data](ConnectOpts::authentication_data) are
    /// set in [`opts`](ConnectOpts), the extended authorization is performed, the result of calling this method
    /// is then [AuthRsp]. Otherwise, the return type is [ConnectRsp].
    ///
    /// When the [reason](crate::reason::ConnectReason) in the CONNACK packet is greater or equal 0x80, the
    /// [ConnectError](crate::error::ConnectError) is returned.
    ///
    /// When in extended authorization mode, the authorize method is used for subsequent
    /// authorization requests.
    ///
    /// # Panics
    /// When invoked without prior call to [set_up](Context::set_up).
    ///
    pub async fn connect<'a>(
        &mut self,
        opts: ConnectOpts<'a>,
    ) -> Result<Either<ConnectRsp, AuthRsp>, MqttError> {
        assert!(
            self.rx.is_some() && self.tx.is_some(),
            "Context must be set up before connecting."
        );

        let packet = opts.build()?;
        self.connection.session_expiry_interval =
            packet.session_expiry_interval.map(u32::from).unwrap_or(0);

        let mut buf = BytesMut::with_capacity(packet.packet_len());
        packet.encode(&mut buf);

        let tx = self.tx.as_mut().unwrap();
        let rx = self.rx.as_mut().unwrap();

        tx.write(buf.as_ref()).await?;

        match rx
            .next()
            .await
            .transpose()
            .map_err(MqttError::from)
            .and_then(|maybe_next| maybe_next.ok_or(SocketClosed.into()))?
        {
            RxPacket::Connack(connack) => {
                Self::handle_connack(&mut self.connection, &connack);
                Ok(Left(ConnectRsp::try_from(connack)?))
            }
            RxPacket::Auth(auth) => Ok(Right(AuthRsp::try_from(auth)?)),
            _ => {
                unreachable!("Unexpected packet type.");
            }
        }
    }

    /// Performs extended authorization between the client and the broker. It corresponds to sending the
    /// [Auth](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901217) packet.
    /// User may perform multiple calls of this method, as needed, until the ConnectRsp is returned,
    /// meaning the authorization is successful.
    ///
    /// When the [reason](crate::reason::AuthReason) in the AUTH packet is greater or equal 0x80, the
    /// [AuthError](crate::error::AuthError) is returned.
    ///
    /// # Panics
    /// When invoked without prior call to [set_up](Context::set_up).
    ///
    pub async fn authorize<'a>(
        &mut self,
        opts: AuthOpts<'a>,
    ) -> Result<Either<ConnectRsp, AuthRsp>, MqttError> {
        assert!(
            self.rx.is_some() && self.tx.is_some(),
            "Context must be set up before authorizing."
        );

        let packet = opts.build()?;

        let mut buf = BytesMut::with_capacity(packet.packet_len());
        packet.encode(&mut buf);

        let tx = self.tx.as_mut().unwrap();
        let rx = self.rx.as_mut().unwrap();

        tx.write(buf.as_ref()).await?;

        match rx
            .next()
            .await
            .transpose()
            .map_err(MqttError::from)
            .and_then(|maybe_next| maybe_next.ok_or(SocketClosed.into()))?
        {
            RxPacket::Connack(connack) => {
                Self::handle_connack(&mut self.connection, &connack);
                Ok(Left(ConnectRsp::try_from(connack)?))
            }
            RxPacket::Auth(auth) => Ok(Right(AuthRsp::try_from(auth)?)),
            _ => {
                unreachable!("Unexpected packet type.");
            }
        }
    }

    /// Starts processing MQTT traffic, blocking (on .await) the current task until
    /// graceful disconnection or error. Successful disconnection via [disconnect](ContextHandle::disconnect) method or
    /// receiving a [Disconnect](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901205)
    /// packet with reason a code equal to 0 (success) is considered a graceful disconnection.
    ///
    /// # Panics
    /// When invoked without prior call to [set_up](Context::set_up).
    ///
    pub async fn run(&mut self) -> Result<(), MqttError>
    where
        RxStreamT: AsyncRead + Unpin,
        TxStreamT: AsyncWrite + Unpin,
    {
        assert!(
            self.rx.is_some() && self.tx.is_some(),
            "Context must be set up before running."
        );

        let rx = self.rx.as_mut().unwrap();
        let tx = self.tx.as_mut().unwrap();
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
                    Self::handle_packet(connection, session, rx_packet?).await?;
                    pck_fut = rx.next().fuse();
                },
                maybe_msg = msg_fut => {
                    Self::handle_message(tx, connection, session, maybe_msg.ok_or(HandleClosed)?).await?;
                    msg_fut = message_queue.next().fuse();
                }
            }
        }
    }
}
