use crate::core::{
    base_types::*,
    properties::*,
    utils::{ByteWriter, PacketID, SizedPacket, SizedProperty, ToByteBuffer, TryToByteBuffer},
};
use std::mem;

#[derive(Default)]
pub(crate) struct ConnectWillProperties {
    will_delay_interval: Option<WillDelayInterval>,
    payload_format_indicator: Option<PayloadFormatIndicator>,
    message_expiry_interval: Option<MessageExpiryInterval>,
    content_type: Option<ContentType>,
    reponse_topic: Option<ResponseTopic>,
    correlation_data: Option<CorrelationData>,
    user_property: Vec<UserProperty>,
}

impl SizedProperty for ConnectWillProperties {
    fn property_len(&self) -> usize {
        self.will_delay_interval
            .as_ref()
            .map(|val| val.property_len())
            .unwrap_or(0)
            + self
                .payload_format_indicator
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
            + self
                .message_expiry_interval
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
            + self
                .content_type
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
            + self
                .reponse_topic
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
            + self
                .correlation_data
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
            + self
                .user_property
                .iter()
                .map(|val| val.property_len())
                .sum::<usize>()
    }
}

impl ToByteBuffer for ConnectWillProperties {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let property_len = VarSizeInt::from(self.property_len());
        let len = property_len.len() + property_len.value() as usize;

        debug_assert!(buf.len() >= len);

        let result = &mut buf[0..len];
        let mut writer = ByteWriter::from(result);

        writer.write(&property_len);

        if let Some(val) = self.will_delay_interval.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.payload_format_indicator.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.message_expiry_interval.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.content_type.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.reponse_topic.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.correlation_data.as_ref() {
            writer.write(val)
        }

        for val in self.user_property.iter() {
            writer.write(val)
        }

        result
    }
}

#[derive(Default)]
pub(crate) struct ConnectWillPropertiesBuilder {
    will_delay_interval: Option<WillDelayInterval>,
    payload_format_indicator: Option<PayloadFormatIndicator>,
    message_expiry_interval: Option<MessageExpiryInterval>,
    content_type: Option<ContentType>,
    reponse_topic: Option<ResponseTopic>,
    correlation_data: Option<CorrelationData>,
    user_property: Vec<UserProperty>,
}

impl ConnectWillPropertiesBuilder {
    pub(crate) fn will_delay_interval(&mut self, val: FourByteInteger) -> &mut Self {
        self.will_delay_interval = Some(WillDelayInterval(val));
        self
    }

    pub(crate) fn payload_format_indicator(&mut self, val: Boolean) -> &mut Self {
        self.payload_format_indicator = Some(PayloadFormatIndicator(val));
        self
    }

    pub(crate) fn message_expiry_interval(&mut self, val: FourByteInteger) -> &mut Self {
        self.message_expiry_interval = Some(MessageExpiryInterval(val));
        self
    }

    pub(crate) fn content_type(&mut self, val: UTF8String) -> &mut Self {
        self.content_type = Some(ContentType(val));
        self
    }

    pub(crate) fn reponse_topic(&mut self, val: UTF8String) -> &mut Self {
        self.reponse_topic = Some(ResponseTopic(val));
        self
    }

    pub(crate) fn correlation_data(&mut self, val: Binary) -> &mut Self {
        self.correlation_data = Some(CorrelationData(val));
        self
    }

    pub(crate) fn user_property(&mut self, val: UTF8StringPair) -> &mut Self {
        self.user_property.push(UserProperty(val));
        self
    }

    pub(crate) fn build(self) -> Option<ConnectWillProperties> {
        Some(ConnectWillProperties {
            will_delay_interval: self.will_delay_interval,
            payload_format_indicator: self.payload_format_indicator,
            message_expiry_interval: self.message_expiry_interval,
            content_type: self.content_type,
            reponse_topic: self.reponse_topic,
            correlation_data: self.correlation_data,
            user_property: self.user_property,
        })
    }
}

#[derive(Default)]
pub(crate) struct ConnectProperties {
    session_expiry_interval: Option<SessionExpiryInterval>,
    receive_maximum: Option<ReceiveMaximum>,
    maximum_packet_size: Option<MaximumPacketSize>,
    topic_alias_maximum: Option<TopicAliasMaximum>,
    request_response_information: Option<RequestResponseInformation>,
    request_problem_information: Option<RequestProblemInformation>,
    authentication_method: Option<AuthenticationMethod>,
    authentication_data: Option<AuthenticationData>,
    user_property: Vec<UserProperty>,
}

impl SizedProperty for ConnectProperties {
    fn property_len(&self) -> usize {
        self.session_expiry_interval
            .as_ref()
            .map(|val| val.property_len())
            .unwrap_or(0)
            + self
                .receive_maximum
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
            + self
                .maximum_packet_size
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
            + self
                .topic_alias_maximum
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
            + self
                .request_response_information
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
            + self
                .request_problem_information
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
            + self
                .authentication_method
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
            + self
                .authentication_data
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
            + self
                .user_property
                .iter()
                .map(|val| val.property_len())
                .sum::<usize>()
    }
}

impl ToByteBuffer for ConnectProperties {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let property_len = VarSizeInt::from(self.property_len());
        let len = property_len.len() + property_len.value() as usize;

        debug_assert!(buf.len() >= len);

        let result = &mut buf[0..len];
        let mut writer = ByteWriter::from(result);

        writer.write(&property_len);

        if let Some(val) = self.session_expiry_interval.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.receive_maximum.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.maximum_packet_size.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.topic_alias_maximum.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.request_response_information.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.request_problem_information.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.authentication_method.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.authentication_data.as_ref() {
            writer.write(val)
        }

        for val in self.user_property.iter() {
            writer.write(val)
        }

        result
    }
}

#[derive(Default)]
pub(crate) struct ConnectPropertiesBuilder {
    session_expiry_interval: Option<SessionExpiryInterval>,
    receive_maximum: Option<ReceiveMaximum>,
    maximum_packet_size: Option<MaximumPacketSize>,
    topic_alias_maximum: Option<TopicAliasMaximum>,
    request_response_information: Option<RequestResponseInformation>,
    request_problem_information: Option<RequestProblemInformation>,
    authentication_method: Option<AuthenticationMethod>,
    authentication_data: Option<AuthenticationData>,
    user_property: Vec<UserProperty>,
}

impl ConnectPropertiesBuilder {
    pub(crate) fn session_expiry_interval(&mut self, val: FourByteInteger) -> &mut Self {
        self.session_expiry_interval = Some(SessionExpiryInterval(val));
        self
    }

    pub(crate) fn receive_maximum(&mut self, val: TwoByteInteger) -> &mut Self {
        self.receive_maximum = Some(ReceiveMaximum(val));
        self
    }

    pub(crate) fn maximum_packet_size(&mut self, val: FourByteInteger) -> &mut Self {
        self.maximum_packet_size = Some(MaximumPacketSize(val));
        self
    }

    pub(crate) fn topic_alias_maximum(&mut self, val: TwoByteInteger) -> &mut Self {
        self.topic_alias_maximum = Some(TopicAliasMaximum(val));
        self
    }

    pub(crate) fn request_response_information(&mut self, val: Byte) -> &mut Self {
        self.request_response_information = Some(RequestResponseInformation(val));
        self
    }

    pub(crate) fn request_problem_information(&mut self, val: Byte) -> &mut Self {
        self.request_problem_information = Some(RequestProblemInformation(val));
        self
    }

    pub(crate) fn authentication_method(&mut self, val: UTF8String) -> &mut Self {
        self.authentication_method = Some(AuthenticationMethod(val));
        self
    }

    pub(crate) fn user_property(&mut self, val: UTF8StringPair) -> &mut Self {
        self.user_property.push(UserProperty(val));
        self
    }

    pub(crate) fn build(self) -> Option<ConnectProperties> {
        Some(ConnectProperties {
            session_expiry_interval: self.session_expiry_interval,
            receive_maximum: self.receive_maximum,
            maximum_packet_size: self.maximum_packet_size,
            topic_alias_maximum: self.topic_alias_maximum,
            request_response_information: self.request_response_information,
            request_problem_information: self.request_problem_information,
            authentication_method: self.authentication_method,
            authentication_data: self.authentication_data,
            user_property: self.user_property,
        })
    }
}

#[derive(Default)]
pub(crate) struct ConnectMetadata {
    will_qos: QoS,
    will_retain: Boolean,
    clean_start: Boolean,
}

#[derive(Default)]
pub(crate) struct ConnectPayload {
    meta: ConnectMetadata,

    client_identifier: Option<UTF8String>,
    will_properties: Option<ConnectWillProperties>,
    will_topic: Option<UTF8String>,
    will_payload: Option<Binary>,
    username: Option<UTF8String>,
    password: Option<Binary>,
}

impl ConnectPayload {
    fn to_flags(&self) -> u8 {
        let will_flag = self
            .will_properties
            .as_ref()
            .and(self.will_topic.as_ref())
            .and(self.will_payload.as_ref())
            .map(|_| 1)
            .unwrap_or(0);

        (self.username.as_ref().map(|_| 1).unwrap_or(0) << 7)
            | (self.password.as_ref().map(|_| 1).unwrap_or(0) << 6)
            | ((self.meta.will_retain as u8) << 5)
            | ((self.meta.will_qos as u8) << 3)
            | (will_flag << 2)
            | ((self.meta.clean_start as u8) << 1)
    }
}

impl SizedProperty for ConnectPayload {
    fn property_len(&self) -> usize {
        let will_properties_len = self
            .will_properties
            .as_ref()
            .map(|val| {
                let len = VarSizeInt::from(val.property_len());
                // If present, the length of Variable Byte Integer must be added
                len.len() + len.value() as usize
            })
            .unwrap_or(0);

        self.client_identifier
            .as_ref()
            .map(|val| val.property_len())
            .unwrap_or(0)
            + will_properties_len
            + self
                .will_topic
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
            + self
                .will_payload
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
            + self
                .username
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
            + self
                .password
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
    }
}

impl ToByteBuffer for ConnectPayload {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let len = self.property_len();

        debug_assert!(buf.len() >= len);

        let result = &mut buf[0..len];
        let mut writer = ByteWriter::from(result);

        if let Some(val) = self.client_identifier.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.will_properties.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.will_topic.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.will_payload.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.username.as_ref() {
            writer.write(val)
        }

        if let Some(val) = self.password.as_ref() {
            writer.write(val)
        }

        result
    }
}

#[derive(Default)]
pub(crate) struct ConnectPayloadBuilder {
    will_qos: Option<QoS>,
    will_retain: Option<Boolean>,
    clean_start: Option<Boolean>,
    client_identifier: Option<UTF8String>,
    will_properties: Option<ConnectWillProperties>,
    will_topic: Option<UTF8String>,
    will_payload: Option<Binary>,
    username: Option<UTF8String>,
    password: Option<Binary>,
}

impl ConnectPayloadBuilder {
    pub(crate) fn will_qos(&mut self, val: QoS) -> &mut Self {
        self.will_qos = Some(val);
        self
    }

    pub(crate) fn will_retain(&mut self, val: Boolean) -> &mut Self {
        self.will_retain = Some(val);
        self
    }

    pub(crate) fn clean_start(&mut self, val: Boolean) -> &mut Self {
        self.clean_start = Some(val);
        self
    }

    pub(crate) fn client_identifier(&mut self, val: UTF8String) -> &mut Self {
        self.client_identifier = Some(val);
        self
    }

    pub(crate) fn will_properties(&mut self, val: ConnectWillProperties) -> &mut Self {
        self.will_properties = Some(val);
        self
    }

    pub(crate) fn will_topic(&mut self, val: UTF8String) -> &mut Self {
        self.will_topic = Some(val);
        self
    }

    pub(crate) fn will_payload(&mut self, val: Binary) -> &mut Self {
        self.will_payload = Some(val);
        self
    }

    pub(crate) fn username(&mut self, val: UTF8String) -> &mut Self {
        self.username = Some(val);
        self
    }

    pub(crate) fn password(&mut self, val: Binary) -> &mut Self {
        self.password = Some(val);
        self
    }

    pub(crate) fn build(self) -> Option<ConnectPayload> {
        Some(ConnectPayload {
            meta: ConnectMetadata {
                will_qos: self.will_qos.unwrap_or_default(),
                will_retain: self.will_retain.unwrap_or_default(),
                clean_start: self.clean_start.unwrap_or_default(),
            },
            client_identifier: self.client_identifier,
            will_properties: self.will_properties,
            will_topic: self.will_topic,
            will_payload: self.will_payload,
            username: self.username,
            password: self.password,
        })
    }
}

#[derive(Default)]
pub(crate) struct Connect {
    keep_alive: TwoByteInteger,
    properties: ConnectProperties,
    payload: ConnectPayload,
}

impl Connect {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;
    const PROTOCOL_NAME: &'static str = "MQTT";
    const PROTOCOL_VERSION: u8 = 5;

    fn remaining_len(&self) -> VarSizeInt {
        let protocol_name_len = Self::PROTOCOL_NAME.property_len();
        let protocol_ver_len = Self::PROTOCOL_VERSION.property_len();
        let connect_flags_len = mem::size_of::<Byte>();
        let keep_alive_len = self.keep_alive.property_len();
        let property_len = VarSizeInt::from(self.properties.property_len());
        let payload_len = self.payload.property_len();

        VarSizeInt::from(
            protocol_name_len
                + protocol_ver_len
                + connect_flags_len
                + keep_alive_len
                + property_len.len() as usize
                + property_len.value() as usize
                + payload_len,
        )
    }
}

impl PacketID for Connect {
    const PACKET_ID: u8 = 1;
}

impl SizedPacket for Connect {
    fn packet_len(&self) -> usize {
        let remaining_len = self.remaining_len();
        mem::size_of_val(&Self::FIXED_HDR) + remaining_len.len() + remaining_len.value() as usize
    }
}

impl TryToByteBuffer for Connect {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let packet_len = self.packet_len();

        let result = buf.get_mut(0..packet_len)?;
        let mut writer = ByteWriter::from(result);

        writer.write(&Self::FIXED_HDR);

        let remaining_len = self.remaining_len();
        debug_assert!(remaining_len.value() as usize <= writer.remaining());

        writer.write(&Self::PROTOCOL_NAME);
        writer.write(&Self::PROTOCOL_VERSION);
        writer.write(&self.payload.to_flags());
        writer.write(&self.keep_alive);
        writer.write(&self.properties);
        writer.write(&self.payload);

        Some(result)
    }
}

#[derive(Default)]
pub(crate) struct ConnectPacketBuilder {
    keep_alive: Option<TwoByteInteger>,
    properties: Option<ConnectProperties>,
    payload: Option<ConnectPayload>,
}

impl ConnectPacketBuilder {
    pub(crate) fn keep_alive(&mut self, val: TwoByteInteger) -> &mut Self {
        self.keep_alive = Some(val);
        self
    }

    pub(crate) fn properties(&mut self, val: ConnectProperties) -> &mut Self {
        self.properties = Some(val);
        self
    }

    pub(crate) fn payload(&mut self, val: ConnectPayload) -> &mut Self {
        self.payload = Some(val);
        self
    }

    pub(crate) fn build(self) -> Option<Connect> {
        Some(Connect {
            keep_alive: self.keep_alive?,
            properties: self.properties.unwrap_or_default(),
            payload: self.payload.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::utils::PropertyID;

    #[test]
    fn to_bytes() {
        const EXPECTED: [u8; 16] = [
            Connect::PACKET_ID << 4,
            14, // Remaining length
            0,
            4,
            b'M',
            b'Q',
            b'T',
            b'T',
            Connect::PROTOCOL_VERSION,
            0, // Connect flags
            0,
            10, // Keep alive
            3,  // Property length
            ReceiveMaximum::PROPERTY_ID,
            0,
            128,
        ];

        let mut property_builder = ConnectPropertiesBuilder::default();
        property_builder.receive_maximum(128);

        let mut builder = ConnectPacketBuilder::default();
        builder.keep_alive(10);
        builder.properties(property_builder.build().unwrap());
        let packet = builder.build().unwrap();

        let mut buf = [0u8; 128];
        let result = packet.try_to_byte_buffer(&mut buf).unwrap();

        assert_eq!(result, EXPECTED);
    }
}
