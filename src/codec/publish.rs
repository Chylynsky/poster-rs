use crate::core::{
    base_types::*,
    collections::UserProperties,
    error::{
        CodecError, InvalidPacketHeader, InvalidPacketSize, InvalidPropertyLength,
        MandatoryPropertyMissing, UnexpectedProperty,
    },
    properties::*,
    utils::{ByteLen, Decoder, Encode, Encoder, PacketID, SizedPacket, TryDecode},
};
use bytes::{Bytes, BytesMut};
use core::mem;
use derive_builder::Builder;

#[derive(Builder)]
#[builder(build_fn(error = "CodecError", validate = "Self::validate"))]
pub(crate) struct PublishRx {
    #[builder(default)]
    pub(crate) dup: bool,
    #[builder(default)]
    pub(crate) retain: bool,
    #[builder(default)]
    pub(crate) qos: QoS,

    pub(crate) topic_name: UTF8String,
    #[builder(setter(strip_option), default)]
    pub(crate) packet_identifier: Option<NonZero<u16>>,

    #[builder(setter(strip_option), default)]
    pub(crate) payload_format_indicator: Option<PayloadFormatIndicator>,
    #[builder(setter(strip_option), default)]
    pub(crate) topic_alias: Option<TopicAlias>,
    #[builder(setter(strip_option), default)]
    pub(crate) message_expiry_interval: Option<MessageExpiryInterval>,
    #[builder(setter(strip_option), default)]
    pub(crate) subscription_identifier: Option<SubscriptionIdentifier>,
    #[builder(setter(strip_option), default)]
    pub(crate) correlation_data: Option<CorrelationData>,
    #[builder(setter(strip_option), default)]
    pub(crate) response_topic: Option<ResponseTopic>,
    #[builder(setter(strip_option), default)]
    pub(crate) content_type: Option<ContentType>,
    #[builder(setter(custom), default)]
    pub(crate) user_property: UserProperties,

    #[builder(default)]
    pub(crate) payload: Payload,
}

impl PublishRxBuilder {
    fn validate(&self) -> Result<(), CodecError> {
        match self.qos.unwrap_or_default() {
            QoS::AtMostOnce => Ok(()),
            _ => match self.packet_identifier {
                Some(_) => Ok(()),
                None => Err(MandatoryPropertyMissing.into()),
            },
        }
    }

    fn user_property(&mut self, value: UserProperty) {
        match self.user_property.as_mut() {
            Some(user_property) => {
                user_property.push(value);
            }
            None => {
                self.user_property = Some(UserProperties::new());
                self.user_property.as_mut().unwrap().push(value);
            }
        }
    }
}

impl PacketID for PublishRx {
    const PACKET_ID: u8 = 3;
}

impl TryDecode for PublishRx {
    type Error = CodecError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        let mut builder = PublishRxBuilder::default();
        let mut decoder = Decoder::from(bytes);

        let fixed_hdr = decoder.try_decode::<u8>()?;
        if fixed_hdr >> 4 != Self::PACKET_ID {
            return Err(InvalidPacketHeader.into());
        }

        let qos = QoS::try_from(((fixed_hdr >> 1) & 0x03) as u8)?;
        builder
            .dup(fixed_hdr & (1 << 3) != 0)
            .retain(fixed_hdr & 1 != 0)
            .qos(qos);

        let remaining_len = decoder.try_decode::<VarSizeInt>()?;
        if remaining_len > decoder.remaining() {
            return Err(InvalidPacketSize.into());
        }

        let topic_name = decoder.try_decode::<UTF8String>()?;
        builder.topic_name(topic_name);

        // Packet identifier only available if QoS > 0
        if qos == QoS::AtLeastOnce || qos == QoS::ExactlyOnce {
            let packet_id = decoder.try_decode::<NonZero<u16>>()?;
            builder.packet_identifier(packet_id);
        }

        let property_len = decoder.try_decode::<VarSizeInt>()?;
        if property_len > decoder.remaining() {
            return Err(InvalidPropertyLength.into());
        }

        let property_iterator =
            Decoder::from(decoder.get_buf().split_to(property_len.value() as usize))
                .iter::<Property>();
        for property in property_iterator {
            if let Err(err) = property {
                return Err(err.into());
            }

            match property.unwrap() {
                Property::PayloadFormatIndicator(val) => {
                    builder.payload_format_indicator(val);
                }
                Property::TopicAlias(val) => {
                    builder.topic_alias(val);
                }
                Property::MessageExpiryInterval(val) => {
                    builder.message_expiry_interval(val);
                }
                Property::SubscriptionIdentifier(val) => {
                    builder.subscription_identifier(val);
                }
                Property::CorrelationData(val) => {
                    builder.correlation_data(val);
                }
                Property::ResponseTopic(val) => {
                    builder.response_topic(val);
                }
                Property::ContentType(val) => {
                    builder.content_type(val);
                }
                Property::UserProperty(val) => {
                    builder.user_property(val);
                }
                _ => {
                    return Err(UnexpectedProperty.into());
                }
            }
        }

        decoder.advance_by(usize::from(property_len));
        builder.payload(decoder.try_decode::<Payload>()?);
        builder.build()
    }
}

#[derive(Builder)]
#[builder(build_fn(error = "CodecError", validate = "Self::validate"))]
pub(crate) struct PublishTx<'a> {
    #[builder(default)]
    pub(crate) dup: bool,
    #[builder(default)]
    pub(crate) retain: bool,
    #[builder(default)]
    pub(crate) qos: QoS,

    pub(crate) topic_name: UTF8StringRef<'a>,
    #[builder(setter(strip_option), default)]
    pub(crate) packet_identifier: Option<NonZero<u16>>,

    #[builder(setter(strip_option), default)]
    pub(crate) payload_format_indicator: Option<PayloadFormatIndicator>,
    #[builder(setter(strip_option), default)]
    pub(crate) topic_alias: Option<TopicAlias>,
    #[builder(setter(strip_option), default)]
    pub(crate) message_expiry_interval: Option<MessageExpiryInterval>,
    // #[builder(setter(strip_option), default)]
    // pub(crate) subscription_identifier: Option<SubscriptionIdentifier>,
    #[builder(setter(strip_option), default)]
    pub(crate) correlation_data: Option<CorrelationDataRef<'a>>,
    #[builder(setter(strip_option), default)]
    pub(crate) response_topic: Option<ResponseTopicRef<'a>>,
    #[builder(setter(strip_option), default)]
    pub(crate) content_type: Option<ContentTypeRef<'a>>,
    #[builder(setter(custom), default)]
    pub(crate) user_property: Vec<UserPropertyRef<'a>>,
    #[builder(setter(strip_option), default)]
    pub(crate) payload: Option<PayloadRef<'a>>,
}

impl<'a> PublishTxBuilder<'a> {
    fn validate(&self) -> Result<(), CodecError> {
        match self.qos.unwrap_or_default() {
            QoS::AtMostOnce => Ok(()),
            _ => match self.packet_identifier {
                Some(_) => Ok(()),
                None => Err(MandatoryPropertyMissing.into()),
            },
        }
    }

    pub(crate) fn user_property(&mut self, value: UserPropertyRef<'a>) {
        match self.user_property.as_mut() {
            Some(user_property) => {
                user_property.push(value);
            }
            None => {
                self.user_property = Some(Vec::new());
                self.user_property.as_mut().unwrap().push(value);
            }
        }
    }
}

impl<'a> PublishTx<'a> {
    fn fixed_hdr(&self) -> u8 {
        (Self::PACKET_ID << 4)
            | ((self.dup as u8) << 3)
            | ((self.qos as u8) << 1)
            | (self.retain as u8)
    }

    fn property_len(&self) -> VarSizeInt {
        VarSizeInt::try_from(
            self.payload_format_indicator
                .as_ref()
                .map(|val| val.byte_len())
                .unwrap_or(0)
                + self
                    .topic_alias
                    .as_ref()
                    .map(|val| val.byte_len())
                    .unwrap_or(0)
                + self
                    .message_expiry_interval
                    .as_ref()
                    .map(|val| val.byte_len())
                    .unwrap_or(0)
                // + self
                //     .subscription_identifier
                //     .as_ref()
                //     .map(|val| val.byte_len())
                //     .unwrap_or(0)
                + self
                    .correlation_data
                    .as_ref()
                    .map(|val| val.byte_len())
                    .unwrap_or(0)
                + self
                    .response_topic
                    .as_ref()
                    .map(|val| val.byte_len())
                    .unwrap_or(0)
                + self
                    .content_type
                    .as_ref()
                    .map(|val| val.byte_len())
                    .unwrap_or(0)
                + self
                    .user_property
                    .iter()
                    .map(|val| val.byte_len())
                    .sum::<usize>(),
        )
        .unwrap()
    }

    fn remaining_len(&self) -> VarSizeInt {
        let property_len = self.property_len();
        VarSizeInt::try_from(
            self.topic_name.byte_len()
                + self
                    .packet_identifier
                    .as_ref()
                    .map(|val| val.byte_len())
                    .unwrap_or(0)
                + property_len.len()
                + property_len.value() as usize
                + self.payload.as_ref().map(|val| val.byte_len()).unwrap_or(0),
        )
        .unwrap()
    }
}

impl<'a> PacketID for PublishTx<'a> {
    const PACKET_ID: u8 = 3;
}

impl<'a> SizedPacket for PublishTx<'a> {
    fn packet_len(&self) -> usize {
        let remaining_len = self.remaining_len();
        mem::size_of::<u8>() // Fixed header size
            + remaining_len.len()
            + remaining_len.value() as usize
    }
}

impl<'a> Encode for PublishTx<'a> {
    fn encode(&self, buf: &mut BytesMut) {
        let mut encoder = Encoder::from(buf);

        encoder.encode(self.fixed_hdr());

        let remaining_len = self.remaining_len();
        encoder.encode(remaining_len);

        encoder.encode(self.topic_name);

        if let Some(val) = self.packet_identifier {
            encoder.encode(val);
        }

        encoder.encode(self.property_len());

        if let Some(val) = self.payload_format_indicator {
            encoder.encode(val);
        }

        if let Some(val) = self.topic_alias {
            encoder.encode(val);
        }

        if let Some(val) = self.message_expiry_interval {
            encoder.encode(val);
        }

        // if let Some(val) = self.subscription_identifier {
        //     encoder.encode(val);
        // }

        if let Some(val) = self.correlation_data {
            encoder.encode(val);
        }

        if let Some(val) = self.response_topic {
            encoder.encode(val);
        }

        if let Some(val) = self.content_type {
            encoder.encode(val);
        }

        for val in self.user_property.iter().copied() {
            encoder.encode(val);
        }

        if let Some(payload) = self.payload {
            encoder.encode(payload);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const FIXED_HDR: u8 = (((PublishRx::PACKET_ID as u8) << 4) | 0x0b) as u8; // DUP: 1, QoS: 1, RETAIN: 1
    const PACKET: [u8; 15] = [
        FIXED_HDR, 13, // Remaining length
        0,  // Topic length MSB
        4,  // Topic length LSB
        b't', b'e', b's', b't', 0, 13, // Packet ID
        0,  // Property length
        // Payload
        b't', b'e', b's', b't',
    ];

    #[test]
    fn from_bytes_0() {
        let packet = PublishRx::try_decode(Bytes::from_static(&PACKET)).unwrap();

        assert!(packet.dup);
        assert!(packet.retain);
        assert_eq!(packet.qos, QoS::AtLeastOnce);
        assert_eq!(packet.packet_identifier.unwrap(), 13);
        assert_eq!(
            packet.payload,
            Payload(Bytes::from_static("test".as_bytes()))
        );
    }

    #[test]
    fn to_bytes_0() {
        let mut builder = PublishTxBuilder::default();
        builder.dup(true);
        builder.qos(QoS::AtLeastOnce);
        builder.retain(true);
        builder.packet_identifier(NonZero::try_from(13).unwrap());
        builder.topic_name(UTF8StringRef("test"));
        builder.payload(PayloadRef(&[b't', b'e', b's', b't']));

        let packet = builder.build().unwrap();
        let mut buf = BytesMut::new();
        packet.encode(&mut buf);

        assert_eq!(&buf.split().freeze()[..], &PACKET);
    }
}
