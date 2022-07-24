use crate::{
    base_types::*,
    properties::*,
    utils::{
        ByteReader, ByteWriter, PacketID, SizedPacket, SizedProperty, TryFromBytes,
        TryFromIterator, TryToByteBuffer,
    },
};
use std::mem;

pub(crate) struct Publish {
    dup: bool,
    retain: bool,
    qos: QoS,

    topic_name: UTF8String,
    packet_identifier: Option<VarSizeInt>,

    payload_format_indicator: Option<PayloadFormatIndicator>,
    topic_alias: Option<TopicAlias>,
    message_expiry_interval: Option<MessageExpiryInterval>,
    subscription_identifier: Option<SubscriptionIdentifier>,
    correlation_data: Option<CorrelationData>,
    response_topic: Option<ResponseTopic>,
    content_type: Option<ContentType>,
    user_property: Vec<UserProperty>,

    payload: Binary,
}

impl Publish {
    fn fixed_hdr(&self) -> Byte {
        (Self::PACKET_ID << 4)
            | ((self.dup as u8) << 3)
            | ((self.qos as u8) << 1)
            | (self.retain as u8)
    }

    fn property_len(&self) -> VarSizeInt {
        VarSizeInt::from(
            self.payload_format_indicator
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
                + self
                    .topic_alias
                    .as_ref()
                    .map(|val| val.property_len())
                    .unwrap_or(0)
                + self
                    .message_expiry_interval
                    .as_ref()
                    .map(|val| val.property_len())
                    .unwrap_or(0)
                + self
                    .subscription_identifier
                    .as_ref()
                    .map(|val| val.property_len())
                    .unwrap_or(0)
                + self
                    .correlation_data
                    .as_ref()
                    .map(|val| val.property_len())
                    .unwrap_or(0)
                + self
                    .response_topic
                    .as_ref()
                    .map(|val| val.property_len())
                    .unwrap_or(0)
                + self
                    .content_type
                    .as_ref()
                    .map(|val| val.property_len())
                    .unwrap_or(0)
                + self
                    .user_property
                    .iter()
                    .map(|val| val.property_len())
                    .sum::<usize>(),
        )
    }

    fn remaining_len(&self) -> VarSizeInt {
        let property_len = self.property_len();
        VarSizeInt::from(
            self.topic_name.property_len()
                + self
                    .packet_identifier
                    .as_ref()
                    .map(|val| val.property_len())
                    .unwrap_or(0)
                + property_len.len()
                + property_len.value() as usize
                + self.payload.property_len(),
        )
    }
}

impl PacketID for Publish {
    const PACKET_ID: u8 = 3;
}

impl SizedPacket for Publish {
    fn packet_len(&self) -> usize {
        let remaining_len = self.remaining_len();
        mem::size_of::<Byte>() // Fixed header size
            + remaining_len.len()
            + remaining_len.value() as usize
    }
}

impl TryFromBytes for Publish {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut builder = PublishPacketBuilder::default();
        let mut reader = ByteReader::from(bytes);

        let fixed_hdr = reader.try_read::<Byte>()?;
        if fixed_hdr >> 4 != Self::PACKET_ID {
            return None; // Invalid header
        }

        let qos = QoS::try_from(((fixed_hdr >> 1) & 0x03) as u8)?;
        builder
            .dup(fixed_hdr & (1 << 3) != 0)
            .retain(fixed_hdr & 1 != 0)
            .qos(qos);

        let remaining_len = reader.try_read::<VarSizeInt>()?;
        let packet_size =
            mem::size_of_val(&fixed_hdr) + remaining_len.len() + remaining_len.value() as usize;
        if packet_size > bytes.len() {
            return None; // Invalid packet size
        }

        let topic_name = reader.try_read::<UTF8String>()?;
        builder.topic_name(topic_name);

        // Packet identifier inly available if QoS > 0
        if qos == QoS::AtLeastOnce || qos == QoS::ExactlyOnce {
            let packet_id = reader.try_read::<VarSizeInt>()?;
            builder.packet_identifier(packet_id);
        }

        let property_len = reader.try_read::<VarSizeInt>()?;
        if property_len.value() as usize > reader.remaining() {
            return None; // Invalid property length
        }

        for property in PropertyIterator::from(reader.get_buf()) {
            match property {
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
                    return None;
                }
            }
        }

        builder.payload(reader.try_read::<Binary>()?);
        builder.build()

        //

        // let mut iter = bytes.iter().copied();
        // let fixed_hdr = iter.next()?;

        // debug_assert!(fixed_hdr >> 4 == Self::PACKET_ID as u8);
        // let qos = QoS::try_from(((fixed_hdr >> 1) & 0x03) as u8)?;
        // builder
        //     .dup(fixed_hdr & (1 << 3) != 0)
        //     .retain(fixed_hdr & 1 != 0)
        //     .qos(qos);

        // let remaining_len = VarSizeInt::try_from_iter(iter)?;
        // if mem::size_of_val(&fixed_hdr) + remaining_len.len() > bytes.len() {
        //     return None;
        // }

        // let (_, remaining) = bytes.split_at(mem::size_of_val(&fixed_hdr) + remaining_len.len());
        // if remaining_len.value() as usize > remaining.len() {
        //     return None;
        // }

        // let (remaining, _) = remaining.split_at(remaining_len.value() as usize);

        // // Topic name must be the first field in variable header
        // let topic_name = UTF8String::try_from_bytes(remaining)?;
        // let (_, mut remaining) = remaining.split_at(topic_name.property_len());
        // builder.topic_name(topic_name);

        // // Packet identifier inly available if QoS > 0
        // if qos == QoS::AtLeastOnce || qos == QoS::ExactlyOnce {
        //     let packet_id = VarSizeInt::try_from_iter(remaining.iter().copied())?;
        //     remaining = &remaining[packet_id.len()..remaining.len()];
        //     builder.packet_identifier(packet_id);
        // }

        // // Read property length
        // let property_len = VarSizeInt::try_from_iter(remaining.iter().copied())?;
        // if property_len.len() > remaining.len() {
        //     return None;
        // }

        // let (_, remaining) = remaining.split_at(property_len.len());
        // if property_len.value() as usize > remaining.len() {
        //     return None;
        // }

        // let (properties, payload) = remaining.split_at(property_len.value() as usize);

        // for property in PropertyIterator::from(properties) {
        //     match property {
        //         Property::PayloadFormatIndicator(val) => {
        //             builder.payload_format_indicator(val);
        //         }
        //         Property::TopicAlias(val) => {
        //             builder.topic_alias(val);
        //         }
        //         Property::MessageExpiryInterval(val) => {
        //             builder.message_expiry_interval(val);
        //         }
        //         Property::SubscriptionIdentifier(val) => {
        //             builder.subscription_identifier(val);
        //         }
        //         Property::CorrelationData(val) => {
        //             builder.correlation_data(val);
        //         }
        //         Property::ResponseTopic(val) => {
        //             builder.response_topic(val);
        //         }
        //         Property::ContentType(val) => {
        //             builder.content_type(val);
        //         }
        //         Property::UserProperty(val) => {
        //             builder.user_property(val);
        //         }
        //         _ => {
        //             return None;
        //         }
        //     }
        // }

        // builder.payload(Binary::try_from_bytes(payload)?);

        // builder.build()
    }
}

impl TryToByteBuffer for Publish {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let packet_len = self.packet_len();
        if packet_len > buf.len() {
            return None;
        }

        let result = &mut buf[0..packet_len];
        let mut writer = ByteWriter::from(result);

        writer.write(&self.fixed_hdr());
        writer.write(&self.remaining_len());
        writer.write(&self.topic_name);

        if let Some(val) = self.packet_identifier.as_ref() {
            writer.write(val);
        }

        writer.write(&self.property_len());

        if let Some(val) = self.payload_format_indicator.as_ref() {
            writer.write(val);
        }

        if let Some(val) = self.topic_alias.as_ref() {
            writer.write(val);
        }

        if let Some(val) = self.message_expiry_interval.as_ref() {
            writer.write(val);
        }

        if let Some(val) = self.subscription_identifier.as_ref() {
            writer.write(val);
        }

        if let Some(val) = self.correlation_data.as_ref() {
            writer.write(val);
        }

        if let Some(val) = self.response_topic.as_ref() {
            writer.write(val);
        }

        if let Some(val) = self.content_type.as_ref() {
            writer.write(val);
        }

        for val in self.user_property.iter() {
            writer.write(val);
        }

        writer.write(&self.payload);

        Some(result)
    }
}

#[derive(Default)]
pub(crate) struct PublishPacketBuilder {
    dup: bool,
    retain: bool,
    qos: QoS,

    topic_name: Option<UTF8String>,
    packet_identifier: Option<VarSizeInt>,

    payload_format_indicator: Option<PayloadFormatIndicator>,
    topic_alias: Option<TopicAlias>,
    message_expiry_interval: Option<MessageExpiryInterval>,
    subscription_identifier: Option<SubscriptionIdentifier>,
    correlation_data: Option<CorrelationData>,
    response_topic: Option<ResponseTopic>,
    content_type: Option<ContentType>,
    user_property: Vec<UserProperty>,

    payload: Option<Binary>,
}

impl PublishPacketBuilder {
    pub(crate) fn dup(&mut self, val: bool) -> &mut Self {
        self.dup = val;
        self
    }

    pub(crate) fn retain(&mut self, val: bool) -> &mut Self {
        self.retain = val;
        self
    }

    pub(crate) fn qos(&mut self, val: QoS) -> &mut Self {
        self.qos = val;
        self
    }

    pub(crate) fn topic_name(&mut self, val: UTF8String) -> &mut Self {
        self.topic_name = Some(val);
        self
    }

    pub(crate) fn packet_identifier(&mut self, val: VarSizeInt) -> &mut Self {
        self.packet_identifier = Some(val);
        self
    }

    pub(crate) fn payload_format_indicator(&mut self, val: PayloadFormatIndicator) -> &mut Self {
        self.payload_format_indicator = Some(val);
        self
    }

    pub(crate) fn topic_alias(&mut self, val: TopicAlias) -> &mut Self {
        self.topic_alias = Some(val);
        self
    }

    pub(crate) fn message_expiry_interval(&mut self, val: MessageExpiryInterval) -> &mut Self {
        self.message_expiry_interval = Some(val);
        self
    }

    pub(crate) fn subscription_identifier(&mut self, val: SubscriptionIdentifier) -> &mut Self {
        self.subscription_identifier = Some(val);
        self
    }

    pub(crate) fn correlation_data(&mut self, val: CorrelationData) -> &mut Self {
        self.correlation_data = Some(val);
        self
    }

    pub(crate) fn response_topic(&mut self, val: ResponseTopic) -> &mut Self {
        self.response_topic = Some(val);
        self
    }

    pub(crate) fn content_type(&mut self, val: ContentType) -> &mut Self {
        self.content_type = Some(val);
        self
    }

    pub(crate) fn user_property(&mut self, val: UserProperty) -> &mut Self {
        self.user_property.push(val);
        self
    }

    pub(crate) fn payload(&mut self, val: Binary) -> &mut Self {
        self.payload = Some(val);
        self
    }

    pub(crate) fn build(self) -> Option<Publish> {
        match self.qos {
            QoS::AtMostOnce => {
                if self.dup {
                    return None;
                }

                if self.packet_identifier.is_some() {
                    return None;
                }
            }
            QoS::AtLeastOnce => {
                self.packet_identifier?;
            }
            QoS::ExactlyOnce => {
                self.packet_identifier?;
            }
        }

        Some(Publish {
            dup: self.dup,
            retain: self.retain,
            qos: self.qos,
            topic_name: self.topic_name?,
            packet_identifier: self.packet_identifier,
            payload_format_indicator: self.payload_format_indicator,
            topic_alias: self.topic_alias,
            message_expiry_interval: self.message_expiry_interval,
            subscription_identifier: self.subscription_identifier,
            correlation_data: self.correlation_data,
            response_topic: self.response_topic,
            content_type: self.content_type,
            user_property: self.user_property,
            payload: self.payload?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const FIXED_HDR: u8 = (((Publish::PACKET_ID as u8) << 4) | 0x0b) as u8; // DUP: 1, QoS: 1, RETAIN: 1
    const PACKET: [u8; 16] = [
        FIXED_HDR, 14, // Remaining length
        0,  // Topic length
        4, b't', b'e', b's', b't', 13, // Packet ID
        0,  // Property length
        // Payload
        0, 4, b't', b'e', b's', b't',
    ];

    #[test]
    fn from_bytes() {
        let packet = Publish::try_from_bytes(&PACKET).unwrap();

        assert!(packet.dup);
        assert!(packet.retain);
        assert_eq!(packet.qos, QoS::AtLeastOnce);
        assert_eq!(packet.packet_identifier.unwrap().value(), 13);
        assert_eq!(std::str::from_utf8(&packet.payload).unwrap(), "test");
    }

    #[test]
    fn to_bytes() {
        let mut builder = PublishPacketBuilder::default();
        builder.dup(true);
        builder.qos(QoS::AtLeastOnce);
        builder.retain(true);
        builder.packet_identifier(VarSizeInt::from(13u8));
        builder.topic_name(String::from("test"));
        builder.payload(Vec::from([b't', b'e', b's', b't']));

        let packet = builder.build().unwrap();
        let mut buf = [0u8; PACKET.len()];
        let result = packet.try_to_byte_buffer(&mut buf).unwrap();

        assert_eq!(result, PACKET);
    }
}
