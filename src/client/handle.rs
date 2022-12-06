use crate::{
    client::{
        error::MqttError,
        message::*,
        opts::{AuthOpts, ConnectOpts, PublishOpts, SubscribeOpts, UnsubscribeOpts},
        rsp::{AuthRsp, ConnectRsp, PublishError, SubscribeError, UnsubscribeError},
        stream::*,
        utils::*,
    },
    codec::*,
    core::{
        base_types::{NonZero, QoS},
        utils::{Encode, SizedPacket},
    },
    PublishData,
};
use bytes::BytesMut;
use core::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use either::{Either, Left, Right};
use futures::{
    channel::{mpsc, oneshot},
    stream, SinkExt, Stream,
};
use std::sync::Arc;

/// Cloneable handle to the client [Context]. The [ContextHandle] object is used to perform MQTT operations.
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
            action_id: tx_action_id(&TxPacket::Connect(packet)),
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
            action_id: tx_action_id(&TxPacket::Auth(packet)),
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
    pub async fn disconnect(&mut self, reason: DisconnectReason) -> Result<(), MqttError> {
        let mut builder = DisconnectTxBuilder::default();
        builder.reason(reason);

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

    /// Performs subscription to the topic specified in `opts`. This corresponds to sending the
    /// [Subscribe](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901161) packet.
    ///
    /// # Arguments
    /// `opts` - Subscription options
    pub async fn subscribe<'a>(
        &mut self,
        opts: SubscribeOpts<'a>,
    ) -> Result<impl Stream<Item = PublishData>, MqttError> {
        let (sender, receiver) = oneshot::channel();
        let (str_sender, str_receiver) = mpsc::unbounded();

        let packet = opts
            .packet_identifier(self.packet_id.fetch_add(1, Ordering::Relaxed))
            .subscription_identifier(self.sub_id.fetch_add(1, Ordering::Relaxed))
            .build()?;

        let subscription_identifier =
            NonZero::from(packet.subscription_identifier.unwrap())
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
            let reason = suback.payload.first().copied().unwrap();
            if reason as u8 >= 0x80 {
                return Err(SubscribeError { reason }.into());
            }

            return Ok(stream::unfold(
                SubscribeStreamState {
                    receiver: str_receiver,
                    sender: self.sender.clone(),
                },
                |mut state| async {
                    state.impl_next().await.map(move |data| (data, state))
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
    pub async fn unsubscribe<'a>(&mut self, opts: UnsubscribeOpts<'a>) -> Result<(), MqttError> {
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
            let reason = unsuback.payload.first().copied().unwrap();
            if reason as u8 >= 0x80 {
                return Err(UnsubscribeError { reason }.into());
            }

            return Ok(());
        }

        unreachable!("Unexpected packet type.");
    }
}
