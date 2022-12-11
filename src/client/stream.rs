use crate::{
    client::{
        message::{AwaitAck, ContextMessage},
        rsp::PublishData,
        utils::*,
    },
    codec::{PubackTxBuilder, PubcompTxBuilder, PubrecTxBuilder, RxPacket, TxPacket},
    core::utils::{Encode, SizedPacket},
    QoS,
};
use bytes::BytesMut;
use futures::{
    channel::{
        mpsc::{self},
        oneshot,
    },
    SinkExt, StreamExt,
};

pub(crate) struct SubscribeStreamState {
    pub(crate) receiver: mpsc::UnboundedReceiver<RxPacket>,
    pub(crate) sender: mpsc::UnboundedSender<ContextMessage>,
}

impl SubscribeStreamState {
    pub(crate) async fn impl_next(&mut self) -> Option<PublishData> {
        match self.receiver.next().await {
            Some(RxPacket::Publish(publish)) => match publish.qos {
                QoS::AtMostOnce => Some(PublishData::from(publish)),
                QoS::AtLeastOnce => {
                    let mut builder = PubackTxBuilder::default();
                    builder.packet_identifier(publish.packet_identifier.unwrap());
                    let puback = builder.build().unwrap();

                    let mut buf = BytesMut::with_capacity(puback.packet_len());
                    puback.encode(&mut buf);

                    self.sender
                        .send(ContextMessage::FireAndForget(buf))
                        .await
                        .ok()?; // Err
                    Some(PublishData::from(publish))
                }
                QoS::ExactlyOnce => {
                    let mut pubrec_builder = PubrecTxBuilder::default();
                    pubrec_builder.packet_identifier(publish.packet_identifier.unwrap());
                    let pubrec = pubrec_builder.build().unwrap();

                    let mut buf = BytesMut::with_capacity(pubrec.packet_len());
                    pubrec.encode(&mut buf);

                    let (pubrel_sender, pubrel_receiver) = oneshot::channel();

                    self.sender
                        .send(ContextMessage::AwaitAck(AwaitAck {
                            action_id: tx_action_id(&TxPacket::Pubrec(pubrec)),
                            packet: buf.split(),
                            response_channel: pubrel_sender,
                        }))
                        .await
                        .ok()?; // Err

                    if let RxPacket::Pubrel(pubrel) = pubrel_receiver.await.ok()? {
                        if pubrel.reason as u8 >= 0x80 {
                            return None; // Err
                        }

                        let mut pubcomp_builder = PubcompTxBuilder::default();
                        pubcomp_builder.packet_identifier(pubrel.packet_identifier);

                        let pubcomp = pubcomp_builder.build().ok()?;
                        pubcomp.encode(&mut buf);

                        self.sender
                            .send(ContextMessage::FireAndForget(buf))
                            .await
                            .ok()?; // Err

                        return Some(PublishData::from(publish));
                    }

                    unreachable!("Unexpected packet type.");
                }
            },
            _ => None,
        }
    }
}
