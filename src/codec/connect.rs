use crate::core::{
    base_types::*,
    error::{CodecError, InsufficientBufferSize},
    properties::*,
    utils::{ByteWriter, PacketID, SizedPacket, SizedProperty, ToByteBuffer, TryToByteBuffer},
};
use core::mem;

#[derive(Default)]
struct ConnectWillProperties {
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
struct ConnectProperties {
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
struct ConnectMetadata {
    will_qos: QoS,
    will_retain: bool,
    clean_start: bool,
}

#[derive(Default)]
struct ConnectPayload {
    meta: ConnectMetadata,

    client_identifier: String,
    will_properties: Option<ConnectWillProperties>,
    will_topic: Option<String>,
    will_payload: Option<Binary>,
    username: Option<String>,
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
                // If present, the length of Variable u8 Integer must be added
                len.len() + len.value() as usize
            })
            .unwrap_or(0);

        self.client_identifier.property_len()
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

        writer.write(&self.client_identifier);

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
pub(crate) struct Connect {
    keep_alive: u16,
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
        let connect_flags_len = mem::size_of::<u8>();
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
    type Error = CodecError;

    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Result<&'a [u8], Self::Error> {
        let result = buf
            .get_mut(0..self.packet_len())
            .ok_or(InsufficientBufferSize)?;
        let mut writer = ByteWriter::from(result);

        writer.write(&Self::FIXED_HDR);

        let remaining_len = self.remaining_len();
        debug_assert!(remaining_len.value() as usize <= writer.remaining());
        writer.write(&remaining_len);

        writer.write(&Self::PROTOCOL_NAME);
        writer.write(&Self::PROTOCOL_VERSION);
        writer.write(&self.payload.to_flags());
        writer.write(&self.keep_alive);
        writer.write(&self.properties);
        writer.write(&self.payload);

        Ok(result)
    }
}

#[derive(Default)]
pub(crate) struct ConnectBuilder {
    keep_alive: Option<u16>,

    session_expiry_interval: Option<SessionExpiryInterval>,
    receive_maximum: Option<ReceiveMaximum>,
    maximum_packet_size: Option<MaximumPacketSize>,
    topic_alias_maximum: Option<TopicAliasMaximum>,
    request_response_information: Option<RequestResponseInformation>,
    request_problem_information: Option<RequestProblemInformation>,
    authentication_method: Option<AuthenticationMethod>,
    authentication_data: Option<AuthenticationData>,
    user_property: Vec<UserProperty>,

    will_qos: QoS,
    will_retain: bool,
    clean_start: bool,
    client_identifier: Option<String>,

    will_delay_interval: Option<WillDelayInterval>,
    will_payload_format_indicator: Option<PayloadFormatIndicator>,
    will_message_expiry_interval: Option<MessageExpiryInterval>,
    will_content_type: Option<ContentType>,
    will_reponse_topic: Option<ResponseTopic>,
    will_correlation_data: Option<CorrelationData>,
    will_user_property: Vec<UserProperty>,

    will_topic: Option<String>,
    will_payload: Option<Binary>,
    username: Option<String>,
    password: Option<Binary>,
}

impl ConnectBuilder {
    pub(crate) fn keep_alive(&mut self, val: u16) -> &mut Self {
        self.keep_alive = Some(val);
        self
    }

    pub(crate) fn session_expiry_interval(&mut self, val: u32) -> &mut Self {
        self.session_expiry_interval = Some(val.into());
        self
    }

    pub(crate) fn receive_maximum(&mut self, val: NonZero<u16>) -> &mut Self {
        self.receive_maximum = Some(val.into());
        self
    }

    pub(crate) fn maximum_packet_size(&mut self, val: NonZero<u32>) -> &mut Self {
        self.maximum_packet_size = Some(val.into());
        self
    }

    pub(crate) fn topic_alias_maximum(&mut self, val: u16) -> &mut Self {
        self.topic_alias_maximum = Some(val.into());
        self
    }

    pub(crate) fn request_response_information(&mut self, val: bool) -> &mut Self {
        self.request_response_information = Some(val.into());
        self
    }

    pub(crate) fn request_problem_information(&mut self, val: bool) -> &mut Self {
        self.request_problem_information = Some(val.into());
        self
    }

    pub(crate) fn authentication_method(&mut self, val: String) -> &mut Self {
        self.authentication_method = Some(val.into());
        self
    }

    pub(crate) fn user_property(&mut self, val: StringPair) -> &mut Self {
        self.user_property.push(val.into());
        self
    }

    pub(crate) fn will_qos(&mut self, val: QoS) -> &mut Self {
        self.will_qos = val;
        self
    }

    pub(crate) fn will_retain(&mut self, val: bool) -> &mut Self {
        self.will_retain = val;
        self
    }

    pub(crate) fn clean_start(&mut self, val: bool) -> &mut Self {
        self.clean_start = val;
        self
    }

    pub(crate) fn client_identifier(&mut self, val: String) -> &mut Self {
        self.client_identifier = Some(val);
        self
    }

    pub(crate) fn will_delay_interval(&mut self, val: u32) -> &mut Self {
        self.will_delay_interval = Some(val.into());
        self
    }

    pub(crate) fn will_payload_format_indicator(&mut self, val: bool) -> &mut Self {
        self.will_payload_format_indicator = Some(val.into());
        self
    }

    pub(crate) fn will_message_expiry_interval(&mut self, val: u32) -> &mut Self {
        self.will_message_expiry_interval = Some(val.into());
        self
    }

    pub(crate) fn will_content_type(&mut self, val: String) -> &mut Self {
        self.will_content_type = Some(val.into());
        self
    }

    pub(crate) fn will_reponse_topic(&mut self, val: String) -> &mut Self {
        self.will_reponse_topic = Some(val.into());
        self
    }

    pub(crate) fn will_correlation_data(&mut self, val: Binary) -> &mut Self {
        self.will_correlation_data = Some(val.into());
        self
    }

    pub(crate) fn will_user_property(&mut self, val: StringPair) -> &mut Self {
        self.will_user_property.push(val.into());
        self
    }

    pub(crate) fn will_topic(&mut self, val: String) -> &mut Self {
        self.will_topic = Some(val);
        self
    }

    pub(crate) fn will_payload(&mut self, val: Binary) -> &mut Self {
        self.will_payload = Some(val);
        self
    }

    pub(crate) fn username(&mut self, val: String) -> &mut Self {
        self.username = Some(val);
        self
    }

    pub(crate) fn password(&mut self, val: Binary) -> &mut Self {
        self.password = Some(val);
        self
    }

    pub(crate) fn build(self) -> Option<Connect> {
        let has_will_properties = self.will_delay_interval.is_some()
            || self.will_payload_format_indicator.is_some()
            || self.will_message_expiry_interval.is_some()
            || self.will_content_type.is_some()
            || self.will_reponse_topic.is_some()
            || self.will_correlation_data.is_some()
            || !self.will_user_property.is_empty();

        let mut will_properties = None;
        if has_will_properties {
            will_properties = Some(ConnectWillProperties {
                will_delay_interval: self.will_delay_interval,
                payload_format_indicator: self.will_payload_format_indicator,
                message_expiry_interval: self.will_message_expiry_interval,
                content_type: self.will_content_type,
                reponse_topic: self.will_reponse_topic,
                correlation_data: self.will_correlation_data,
                user_property: self.will_user_property,
            });
        }

        if self.authentication_method.is_none() && self.authentication_data.is_some() {
            return None; // Cannot include authentication data when authentication method is absent.
        }

        Some(Connect {
            keep_alive: self.keep_alive.unwrap_or(0),
            properties: ConnectProperties {
                session_expiry_interval: self.session_expiry_interval,
                receive_maximum: self.receive_maximum,
                maximum_packet_size: self.maximum_packet_size,
                topic_alias_maximum: self.topic_alias_maximum,
                request_response_information: self.request_response_information,
                request_problem_information: self.request_problem_information,
                authentication_method: self.authentication_method,
                authentication_data: self.authentication_data,
                user_property: self.user_property,
            },
            payload: ConnectPayload {
                meta: ConnectMetadata {
                    will_qos: self.will_qos,
                    will_retain: self.will_retain,
                    clean_start: self.clean_start,
                },
                client_identifier: self.client_identifier?,
                will_properties,
                will_topic: self.will_topic,
                will_payload: self.will_payload,
                username: self.username,
                password: self.password,
            },
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_bytes_0() {
        const EXPECTED: [u8; 22] = [
            Connect::FIXED_HDR,
            20,
            0,
            4,
            b'M',
            b'Q',
            b'T',
            b'T',
            Connect::PROTOCOL_VERSION,
            0,
            0,
            0,
            0,
            0,
            7,
            b't',
            b'e',
            b's',
            b't',
            b'1',
            b'2',
            b'3',
        ];

        let mut builder = ConnectBuilder::default();
        builder.client_identifier(String::from("test123"));
        let packet = builder.build().unwrap();

        let mut buf = [0u8; EXPECTED.len()];
        let result = packet.try_to_byte_buffer(&mut buf).unwrap();

        assert_eq!(result, EXPECTED);
    }
}
