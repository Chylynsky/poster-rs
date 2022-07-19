use crate::{
    base_types::*,
    properties::*,
    utils::{SizedProperty, TryFromBytes, TryFromIterator},
};
use std::mem;

pub struct Publish {
    // Fixed header
    dup: bool,
    retain: bool,
    qos: QoS,

    // Variable header
    topic_name: UTF8String,
    packet_identifier: Option<VarSizeInt>,

    // Properties
    payload_format_indicator: Option<PayloadFormatIndicator>,

    topic_alias: Option<TopicAlias>,

    message_expiry_interval: Option<MessageExpiryInterval>,

    subscription_identifier: Option<SubscriptionIdentifier>,

    correlation_data: Option<CorrelationData>,

    response_topic: Option<ResponseTopic>,
    content_type: Option<ContentType>,

    user_property: Vec<UserProperty>,

    // Payload
    payload: Binary,
}

#[derive(Default)]
pub struct PublishPacketBuilder {
    // Fixed header
    dup: Option<bool>,
    retain: Option<bool>,
    qos: Option<QoS>,

    // Variable header
    topic_name: Option<UTF8String>,
    packet_identifier: Option<VarSizeInt>,

    // Properties
    payload_format_indicator: Option<PayloadFormatIndicator>,

    topic_alias: Option<TopicAlias>,

    message_expiry_interval: Option<MessageExpiryInterval>,

    subscription_identifier: Option<SubscriptionIdentifier>,

    correlation_data: Option<CorrelationData>,

    response_topic: Option<ResponseTopic>,
    content_type: Option<ContentType>,

    user_property: Vec<UserProperty>,

    // Payload
    payload: Option<Binary>,
}

impl PublishPacketBuilder {
    fn dup(&mut self, val: bool) -> &mut Self {
        self.dup = Some(val);
        self
    }

    fn retain(&mut self, val: bool) -> &mut Self {
        self.retain = Some(val);
        self
    }

    fn qos(&mut self, val: QoS) -> &mut Self {
        self.qos = Some(val);
        self
    }

    fn topic_name(&mut self, val: UTF8String) -> &mut Self {
        self.topic_name = Some(val);
        self
    }

    fn packet_identifier(&mut self, val: VarSizeInt) -> &mut Self {
        self.packet_identifier = Some(val);
        self
    }

    fn payload_format_indicator(&mut self, val: PayloadFormatIndicator) -> &mut Self {
        self.payload_format_indicator = Some(val);
        self
    }

    fn topic_alias(&mut self, val: TopicAlias) -> &mut Self {
        self.topic_alias = Some(val);
        self
    }

    fn message_expiry_interval(&mut self, val: MessageExpiryInterval) -> &mut Self {
        self.message_expiry_interval = Some(val);
        self
    }

    fn subscription_identifier(&mut self, val: SubscriptionIdentifier) -> &mut Self {
        self.subscription_identifier = Some(val);
        self
    }

    fn correlation_data(&mut self, val: CorrelationData) -> &mut Self {
        self.correlation_data = Some(val);
        self
    }

    fn response_topic(&mut self, val: ResponseTopic) -> &mut Self {
        self.response_topic = Some(val);
        self
    }

    fn content_type(&mut self, val: ContentType) -> &mut Self {
        self.content_type = Some(val);
        self
    }

    fn user_property(&mut self, val: UserProperty) -> &mut Self {
        self.user_property.push(val);
        self
    }

    fn payload(&mut self, val: Binary) -> &mut Self {
        self.payload = Some(val);
        self
    }

    fn build(self) -> Option<Publish> {
        match self.qos? {
            QoS::AtMostOnce => {
                if self.dup? {
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
            dup: self.dup?,
            retain: self.retain?,
            qos: self.qos.unwrap(),
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

impl Publish {
    pub const PACKET_ID: isize = 0x03;
}

impl TryFromBytes for Publish {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut packet_builder = PublishPacketBuilder::default();

        let mut iter = bytes.iter().copied();
        let fixed_hdr = iter.next()?;

        debug_assert!(fixed_hdr >> 4 == Self::PACKET_ID as u8);
        let qos = QoS::try_from(((fixed_hdr >> 1) & 0x03) as u8)?;
        packet_builder
            .dup(fixed_hdr & (1 << 3) != 0)
            .retain(fixed_hdr & 1 != 0)
            .qos(qos);

        let remaining_len = VarSizeInt::try_from_iter(iter)?;
        if mem::size_of_val(&fixed_hdr) + remaining_len.len() > bytes.len() {
            return None;
        }

        let (_, remaining) = bytes.split_at(mem::size_of_val(&fixed_hdr) + remaining_len.len());
        if remaining_len.value() as usize > remaining.len() {
            return None;
        }

        let (remaining, _) = remaining.split_at(remaining_len.value() as usize);

        // Topic name must be the first field in variable header
        let topic_name = UTF8String::try_from_bytes(remaining)?;
        let (_, mut remaining) = remaining.split_at(topic_name.property_len());
        packet_builder.topic_name(topic_name);

        // Packet identifier inly available if QoS > 0
        if qos == QoS::AtLeastOnce || qos == QoS::ExactlyOnce {
            let packet_id = VarSizeInt::try_from_iter(remaining.iter().copied())?;
            remaining = &remaining[packet_id.len()..remaining.len()];
            packet_builder.packet_identifier(packet_id);
        }

        // Read property length
        let property_len = VarSizeInt::try_from_iter(remaining.iter().copied())?;
        if property_len.len() > remaining.len() {
            return None;
        }

        let (_, remaining) = remaining.split_at(property_len.len());
        if property_len.value() as usize > remaining.len() {
            return None;
        }

        let (properties, payload) = remaining.split_at(property_len.value() as usize);

        for property in PropertyIterator::from(properties) {
            match property {
                Property::PayloadFormatIndicator(val) => {
                    packet_builder.payload_format_indicator(val);
                }
                Property::TopicAlias(val) => {
                    packet_builder.topic_alias(val);
                }
                Property::MessageExpiryInterval(val) => {
                    packet_builder.message_expiry_interval(val);
                }
                Property::SubscriptionIdentifier(val) => {
                    packet_builder.subscription_identifier(val);
                }
                Property::CorrelationData(val) => {
                    packet_builder.correlation_data(val);
                }
                Property::ResponseTopic(val) => {
                    packet_builder.response_topic(val);
                }
                Property::ContentType(val) => {
                    packet_builder.content_type(val);
                }
                Property::UserProperty(val) => {
                    packet_builder.user_property(val);
                }
                _ => {
                    return None;
                }
            }
        }

        packet_builder.payload(Vec::from(payload));

        packet_builder.build()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes() {
        const FIXED_HDR: u8 = (((Publish::PACKET_ID as u8) << 4) | 0x0b) as u8; // DUP: 1, QoS: 1, RETAIN: 1
        const PACKET: [u8; 14] = [
            FIXED_HDR, 12, // Remaining length
            0,  // Topic length
            4, b't', b'e', b's', b't', 13, // Packet ID
            0,  // Property length
            // Payload
            b't', b'e', b's', b't',
        ];

        let packet = Publish::try_from_bytes(&PACKET).unwrap();

        assert!(packet.dup);
        assert!(packet.retain);
        assert_eq!(packet.qos, QoS::AtLeastOnce);
        assert_eq!(packet.packet_identifier.unwrap().value(), 13);
        assert_eq!(std::str::from_utf8(&packet.payload).unwrap(), "test");
    }
}
