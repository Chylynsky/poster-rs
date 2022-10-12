use crate::core::{
    base_types::*,
    error::{
        CodecError, InsufficientBufferSize, InvalidPacketHeader, InvalidPacketSize,
        InvalidPropertyLength, MandatoryPropertyMissing, UnexpectedProperty,
    },
    properties::*,
    utils::{
        ByteReader, ByteWriter, PacketID, SizedPacket, SizedProperty, TryFromBytes, TryToByteBuffer,
    },
};
use core::mem;

pub(crate) struct Publish {
    pub(crate) dup: bool,
    pub(crate) retain: bool,
    pub(crate) qos: QoS,

    pub(crate) topic_name: String,
    pub(crate) packet_identifier: Option<NonZero<u16>>,

    pub(crate) payload_format_indicator: Option<PayloadFormatIndicator>,
    pub(crate) topic_alias: Option<TopicAlias>,
    pub(crate) message_expiry_interval: Option<MessageExpiryInterval>,
    pub(crate) subscription_identifier: Option<SubscriptionIdentifier>,
    pub(crate) correlation_data: Option<CorrelationData>,
    pub(crate) response_topic: Option<ResponseTopic>,
    pub(crate) content_type: Option<ContentType>,
    pub(crate) user_property: Vec<UserProperty>,

    pub(crate) payload: Binary,
}

impl Publish {
    fn fixed_hdr(&self) -> u8 {
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
        mem::size_of::<u8>() // Fixed header size
            + remaining_len.len()
            + remaining_len.value() as usize
    }
}

impl TryFromBytes for Publish {
    type Error = CodecError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut builder = PublishBuilder::default();
        let mut reader = ByteReader::from(bytes);

        let fixed_hdr = reader.try_read::<u8>()?;
        if fixed_hdr >> 4 != Self::PACKET_ID {
            return Err(InvalidPacketHeader.into());
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
            return Err(InvalidPacketSize.into());
        }

        let topic_name = reader.try_read::<String>()?;
        builder.topic_name(topic_name);

        // Packet identifier only available if QoS > 0
        if qos == QoS::AtLeastOnce || qos == QoS::ExactlyOnce {
            let packet_id = reader.try_read::<NonZero<u16>>()?;
            builder.packet_identifier(packet_id);
        }

        let property_len = reader.try_read::<VarSizeInt>().map(usize::from)?;
        if property_len as usize > reader.remaining() {
            return Err(InvalidPropertyLength.into());
        }

        let property_buf = reader.get_buf().get(..property_len).unwrap();
        for property in PropertyIterator::from(property_buf) {
            if property.is_err() {
                return Err(property.unwrap_err().into());
            }

            match property.unwrap() {
                Property::PayloadFormatIndicator(val) => {
                    builder.payload_format_indicator(val.into());
                }
                Property::TopicAlias(val) => {
                    builder.topic_alias(val.into());
                }
                Property::MessageExpiryInterval(val) => {
                    builder.message_expiry_interval(val.into());
                }
                Property::SubscriptionIdentifier(val) => {
                    builder.subscription_identifier(val.into());
                }
                Property::CorrelationData(val) => {
                    builder.correlation_data(val.into());
                }
                Property::ResponseTopic(val) => {
                    builder.response_topic(val.into());
                }
                Property::ContentType(val) => {
                    builder.content_type(val.into());
                }
                Property::UserProperty(val) => {
                    builder.user_property(val.into());
                }
                _ => {
                    return Err(UnexpectedProperty.into());
                }
            }
        }

        reader.advance_by(property_len);

        builder.payload(reader.try_read::<Binary>()?);
        builder.build()
    }
}

impl TryToByteBuffer for Publish {
    type Error = CodecError;

    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Result<&'a [u8], Self::Error> {
        let result = buf
            .get_mut(0..self.packet_len())
            .ok_or(InsufficientBufferSize)?;
        let mut writer = ByteWriter::from(result);

        writer.write(&self.fixed_hdr());

        let remaining_len = self.remaining_len();
        debug_assert!(remaining_len.value() as usize <= writer.remaining());
        writer.write(&remaining_len);

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

        Ok(result)
    }
}

#[derive(Default)]
pub(crate) struct PublishBuilder {
    dup: bool,
    retain: bool,
    qos: QoS,

    topic_name: Option<String>,
    packet_identifier: Option<NonZero<u16>>,

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

impl PublishBuilder {
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

    pub(crate) fn topic_name(&mut self, val: String) -> &mut Self {
        self.topic_name = Some(val);
        self
    }

    pub(crate) fn packet_identifier(&mut self, val: NonZero<u16>) -> &mut Self {
        self.packet_identifier = Some(val);
        self
    }

    pub(crate) fn payload_format_indicator(&mut self, val: bool) -> &mut Self {
        self.payload_format_indicator = Some(val.into());
        self
    }

    pub(crate) fn topic_alias(&mut self, val: NonZero<u16>) -> &mut Self {
        self.topic_alias = Some(val.into());
        self
    }

    pub(crate) fn message_expiry_interval(&mut self, val: u32) -> &mut Self {
        self.message_expiry_interval = Some(val.into());
        self
    }

    pub(crate) fn subscription_identifier(&mut self, val: NonZero<VarSizeInt>) -> &mut Self {
        self.subscription_identifier = Some(val.into());
        self
    }

    pub(crate) fn correlation_data(&mut self, val: Binary) -> &mut Self {
        self.correlation_data = Some(val.into());
        self
    }

    pub(crate) fn response_topic(&mut self, val: String) -> &mut Self {
        self.response_topic = Some(val.into());
        self
    }

    pub(crate) fn content_type(&mut self, val: String) -> &mut Self {
        self.content_type = Some(val.into());
        self
    }

    pub(crate) fn user_property(&mut self, val: StringPair) -> &mut Self {
        self.user_property.push(val.into());
        self
    }

    pub(crate) fn payload(&mut self, val: Binary) -> &mut Self {
        self.payload = Some(val);
        self
    }

    pub(crate) fn build(self) -> Result<Publish, CodecError> {
        match self.qos {
            QoS::AtMostOnce => {
                if self.dup {
                    return Err(UnexpectedProperty.into());
                }

                if self.packet_identifier.is_some() {
                    return Err(UnexpectedProperty.into());
                }
            }
            QoS::AtLeastOnce => {
                self.packet_identifier.ok_or(MandatoryPropertyMissing)?;
            }
            QoS::ExactlyOnce => {
                self.packet_identifier.ok_or(MandatoryPropertyMissing)?;
            }
        }

        Ok(Publish {
            dup: self.dup,
            retain: self.retain,
            qos: self.qos,
            topic_name: self.topic_name.ok_or(MandatoryPropertyMissing)?,
            packet_identifier: self.packet_identifier,
            payload_format_indicator: self.payload_format_indicator,
            topic_alias: self.topic_alias,
            message_expiry_interval: self.message_expiry_interval,
            subscription_identifier: self.subscription_identifier,
            correlation_data: self.correlation_data,
            response_topic: self.response_topic,
            content_type: self.content_type,
            user_property: self.user_property,
            payload: self.payload.ok_or(MandatoryPropertyMissing)?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const FIXED_HDR: u8 = (((Publish::PACKET_ID as u8) << 4) | 0x0b) as u8; // DUP: 1, QoS: 1, RETAIN: 1
    const PACKET: [u8; 17] = [
        FIXED_HDR, 15, // Remaining length
        0,  // Topic length MSB
        4,  // Topic length LSB
        b't', b'e', b's', b't', 0, 13, // Packet ID
        0,  // Property length
        // Payload
        0, 4, b't', b'e', b's', b't',
    ];

    #[test]
    fn from_bytes_0() {
        let packet = Publish::try_from_bytes(&PACKET).unwrap();

        assert!(packet.dup);
        assert!(packet.retain);
        assert_eq!(packet.qos, QoS::AtLeastOnce);
        assert_eq!(packet.packet_identifier.unwrap(), 13.into());
        assert_eq!(std::str::from_utf8(&packet.payload).unwrap(), "test");
    }

    #[test]
    fn to_bytes_0() {
        let mut builder = PublishBuilder::default();
        builder.dup(true);
        builder.qos(QoS::AtLeastOnce);
        builder.retain(true);
        builder.packet_identifier(NonZero::from(13));
        builder.topic_name(String::from("test"));
        builder.payload(Vec::from([b't', b'e', b's', b't']));

        let packet = builder.build().unwrap();
        let mut buf = [0u8; PACKET.len()];
        let result = packet.try_to_byte_buffer(&mut buf).unwrap();

        assert_eq!(result, PACKET);
    }
}
