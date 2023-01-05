use crate::{
    client::{
        error::MqttError,
        error::PublishError,
        message::*,
        opts::{
            AuthOpts, ConnectOpts, DisconnectOpts, PublishOpts, SubscribeOpts, UnsubscribeOpts,
        },
        rsp::{AuthRsp, ConnectRsp},
        rsp::{SubscribeRsp, UnsubscribeRsp},
        utils::*,
    },
    codec::*,
    core::{
        base_types::{NonZero, QoS},
        utils::{Encode, SizedPacket},
    },
};
use bytes::BytesMut;
use core::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use either::{Either, Left, Right};
use futures::{
    channel::{mpsc, oneshot},
    SinkExt,
};
use std::sync::Arc;

/// Cloneable handle to the client [Context](crate::Context). The [ContextHandle] object is used to perform MQTT operations.
///
#[derive(Clone)]
pub struct ContextHandle {
    pub(crate) sender: mpsc::UnboundedSender<ContextMessage>,
    pub(crate) packet_id: Arc<AtomicU16>,
    pub(crate) sub_id: Arc<AtomicU32>,
}

impl ContextHandle {
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
    pub async fn connect<'a>(
        &mut self,
        opts: ConnectOpts<'a>,
    ) -> Result<Either<ConnectRsp, AuthRsp>, MqttError> {
        let (sender, receiver) = oneshot::channel();

        let packet = opts.build()?;
        let session_expiry_interval = packet.session_expiry_interval.map(u32::from).unwrap_or(0);

        let mut buf = BytesMut::with_capacity(packet.packet_len());
        packet.encode(&mut buf);

        let message = ContextMessage::Connect(Connect {
            action_id: tx_action_id(&TxPacket::Connect(packet)),
            session_expiry_interval,
            packet: buf,
            response_channel: sender,
        });

        self.sender.send(message).await?;

        match receiver.await? {
            RxPacket::Connack(connack) => Ok(Left(ConnectRsp::try_from(connack)?)),
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
    pub async fn authorize<'a>(
        &mut self,
        opts: AuthOpts<'a>,
    ) -> Result<Either<ConnectRsp, AuthRsp>, MqttError> {
        let (sender, receiver) = oneshot::channel();

        let packet = opts.build()?;

        let mut buf = BytesMut::with_capacity(packet.packet_len());
        packet.encode(&mut buf);

        let message = ContextMessage::AwaitAck(AwaitAck {
            action_id: tx_action_id(&TxPacket::Auth(packet)),
            packet: buf,
            response_channel: sender,
        });

        self.sender.send(message).await?;

        match receiver.await? {
            RxPacket::Connack(connack) => Ok(Left(ConnectRsp::try_from(connack)?)),
            RxPacket::Auth(auth) => Ok(Right(AuthRsp::try_from(auth)?)),
            _ => {
                unreachable!("Unexpected packet type.");
            }
        }
    }

    /// Performs graceful disconnection with the broker by sending the
    /// [Disconnect](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901205) packet.
    ///
    pub async fn disconnect<'a>(&mut self, opts: DisconnectOpts<'a>) -> Result<(), MqttError> {
        let packet = opts.build()?;

        let mut buf = BytesMut::with_capacity(packet.packet_len());
        packet.encode(&mut buf);

        let message = ContextMessage::Disconnect(buf);

        self.sender.send(message).await?;
        Ok(())
    }

    /// Sends ping to the broker by sending
    /// [Ping](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901195) packet.
    /// This method MUST be called periodically if `session_expiry_interval` ([ConnectOpts]) was
    /// set during connection request in order to maintain the session.
    ///
    pub async fn ping(&mut self) -> Result<(), MqttError> {
        let (sender, receiver) = oneshot::channel();

        let builder = PingreqTxBuilder::default();
        let packet = builder.build().unwrap();

        let mut buf = BytesMut::with_capacity(packet.packet_len());
        packet.encode(&mut buf);

        let message = ContextMessage::AwaitAck(AwaitAck {
            action_id: tx_action_id(&TxPacket::Pingreq(packet)),
            packet: buf,
            response_channel: sender,
        });

        self.sender.send(message).await?;

        if let RxPacket::Pingresp(_) = receiver.await? {
            return Ok(());
        }

        unreachable!("Unexpected packet type.");
    }

    /// Publish data with the parameters set in [PublishOpts]. Acknowledgement of QoS>0
    /// messages is handled automatically.
    ///
    /// [PublishError](crate::error::PublishError) is returned for:
    /// - [QoS::AtLeastOnce] PUBACK reason greater or equal 0x80
    /// - [QoS::ExactlyOnce] PUBREC or PUBCOMP reason greater or equal 0x80
    ///
    pub async fn publish<'a>(&mut self, opts: PublishOpts<'a>) -> Result<(), MqttError> {
        match opts.qos.unwrap_or_default() {
            QoS::AtMostOnce => {
                let packet = opts.build()?;

                let mut buf = BytesMut::with_capacity(packet.packet_len());
                packet.encode(&mut buf);

                let message = ContextMessage::FireAndForget(buf);
                self.sender.send(message).await?;
                Ok(())
            }
            QoS::AtLeastOnce => {
                let packet = opts
                    .packet_identifier(self.packet_id.fetch_add(1, Ordering::Relaxed))
                    .build()?;

                let mut buf = BytesMut::with_capacity(packet.packet_len());
                packet.encode(&mut buf);

                let (sender, receiver) = oneshot::channel();

                let message = ContextMessage::AwaitAck(AwaitAck {
                    action_id: tx_action_id(&TxPacket::Publish(packet)),
                    packet: buf,
                    response_channel: sender,
                });

                self.sender.send(message).await?;

                if let RxPacket::Puback(puback) = receiver.await? {
                    if puback.reason as u8 >= 0x80 {
                        return Err(PublishError::Puback(puback.reason).into());
                    }

                    return Ok(());
                }

                unreachable!("Unexpected packet type.");
            }
            QoS::ExactlyOnce => {
                let packet = opts
                    .packet_identifier(self.packet_id.fetch_add(1, Ordering::Relaxed))
                    .build()?;

                let mut buf = BytesMut::with_capacity(packet.packet_len());
                packet.encode(&mut buf);

                let (pubrec_sender, pubrec_receiver) = oneshot::channel();

                let pub_msg = ContextMessage::AwaitAck(AwaitAck {
                    action_id: tx_action_id(&TxPacket::Publish(packet)),
                    packet: buf.split(),
                    response_channel: pubrec_sender,
                });

                self.sender.send(pub_msg).await?;

                if let RxPacket::Pubrec(pubrec) = pubrec_receiver.await? {
                    if pubrec.reason as u8 >= 0x80 {
                        return Err(PublishError::Pubrec(pubrec.reason).into());
                    }

                    let (pubrel_sender, pubrel_receiver) = oneshot::channel();

                    let mut builder = PubrelTxBuilder::default();
                    builder.packet_identifier(pubrec.packet_identifier);

                    let pubrel = builder.build().unwrap();

                    buf.reserve(pubrel.packet_len());
                    pubrel.encode(&mut buf);

                    let pubrel_msg = ContextMessage::AwaitAck(AwaitAck {
                        action_id: tx_action_id(&TxPacket::Pubrel(pubrel)),
                        packet: buf,
                        response_channel: pubrel_sender,
                    });

                    self.sender.send(pubrel_msg).await?;

                    if let RxPacket::Pubcomp(pubcomp) = pubrel_receiver.await? {
                        if pubcomp.reason as u8 >= 0x80 {
                            return Err(PublishError::Pubcomp(pubcomp.reason).into());
                        }

                        return Ok(());
                    }
                }

                unreachable!("Unexpected packet type.");
            }
        }
    }

    /// Performs subscription to the topics specified in [`opts`](SubscribeOpts). This corresponds to sending the
    /// [Subscribe](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901161) packet.
    ///
    /// QoS>0 messages are acknowledged automatically.
    ///
    /// On success returns [SubscribeRsp] object containing the acknowledgment data from the broker.
    /// This object can be transformed into the asynchronous stream of messages published to the subscribed
    /// topics by using the [stream](SubscribeRsp::stream) method.
    ///
    /// # Errors
    /// Per-topic [reason codes](SubackReason) are retrieved with the [payload](SubscribeRsp::payload) method.
    ///
    pub async fn subscribe<'a>(
        &mut self,
        opts: SubscribeOpts<'a>,
    ) -> Result<SubscribeRsp, MqttError> {
        let (sender, receiver) = oneshot::channel();
        let (str_sender, str_receiver) = mpsc::unbounded();

        let packet = opts
            .packet_identifier(self.packet_id.fetch_add(1, Ordering::Relaxed))
            .subscription_identifier(self.sub_id.fetch_add(1, Ordering::Relaxed))
            .build()?;

        let subscription_identifier = NonZero::from(packet.subscription_identifier.unwrap())
            .get()
            .value();

        let mut buf = BytesMut::with_capacity(packet.packet_len());
        packet.encode(&mut buf);

        let message = ContextMessage::Subscribe(Subscribe {
            action_id: tx_action_id(&TxPacket::Subscribe(packet)),
            subscription_identifier: subscription_identifier as usize,
            packet: buf,
            response_channel: sender,
            stream: str_sender,
        });

        self.sender.send(message).await?;

        if let RxPacket::Suback(suback) = receiver.await? {
            return Ok(SubscribeRsp {
                packet: suback,
                receiver: str_receiver,
                sender: self.sender.clone(),
            });
        }

        unreachable!("Unexpected packet type.");
    }

    /// Unsubscribes from the topics specified in [`opts`](UnsubscribeOpts). This corresponds to sending the
    /// [Unsubscribe](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901179) packet.
    ///
    /// # Errors
    /// Per-topic [reason codes](UnsubackReason) are retrieved with the [payload](UnsubscribeRsp::payload) method.
    ///
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
            action_id: tx_action_id(&TxPacket::Unsubscribe(packet)),
            packet: buf,
            response_channel: sender,
        });

        self.sender.send(message).await?;

        if let RxPacket::Unsuback(unsuback) = receiver.await? {
            return Ok(UnsubscribeRsp { packet: unsuback });
        }

        unreachable!("Unexpected packet type.");
    }
}
