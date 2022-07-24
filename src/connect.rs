use crate::{
    base_types::*,
    properties::*,
    utils::{
        ByteWriter, PacketID, PropertyID, SizedPacket, SizedProperty, ToByteBuffer, TryToByteBuffer,
    },
};
use std::mem;

#[derive(Default)]
pub(crate) struct ConnectWillProperties {
    will_delay_interval: Option<FourByteInteger>,
    payload_format_indicator: Option<Byte>,
    message_expiry_interval: Option<FourByteInteger>,
    content_type: Option<UTF8String>,
    reponse_topic: Option<UTF8String>,
    correlation_data: Option<Binary>,
    user_property: Vec<UTF8StringPair>,
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
        let len = self.property_len();

        debug_assert!(buf.len() >= len);

        let result = &mut buf[0..len];
        let mut writer = ByteWriter::from(result);

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
struct ConnectMetadata {
    will_qos: QoS,
    will_retain: bool,
    clean_start: bool,
}

#[derive(Default)]
pub(crate) struct ConnectPayload {
    meta: ConnectMetadata,

    client_identifier: Option<AssignedClientIdentifier>,
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
        self.client_identifier
            .as_ref()
            .map(|val| val.property_len())
            .unwrap_or(0)
            + self
                .will_properties
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
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
pub(crate) struct Connect {
    keep_alive: TwoByteInteger,
    properties: ConnectProperties,
    payload: ConnectPayload,
}

impl Connect {
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
        const FIXED_HDR_LEN: usize = mem::size_of::<Byte>();
        let remaining_len = self.remaining_len();

        FIXED_HDR_LEN + remaining_len.len() + remaining_len.value() as usize
    }
}

impl TryToByteBuffer for Connect {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let packet_len = self.packet_len();

        if packet_len > buf.len() {
            return None;
        }

        let result = &mut buf[0..packet_len];
        let mut writer = ByteWriter::from(result);

        writer.write(&(Self::PACKET_ID << 4));
        writer.write(&self.remaining_len());
        writer.write(&Self::PROTOCOL_NAME);
        writer.write(&Self::PROTOCOL_VERSION);
        writer.write(&self.payload.to_flags());
        writer.write(&self.keep_alive);
        writer.write(&self.properties);
        writer.write(&self.payload);

        Some(result)
    }
}

pub(crate) struct ConnectPacketBuilder {}

#[cfg(test)]
mod test {
    use super::*;

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

        let mut input = Connect {
            keep_alive: 10,
            ..Default::default()
        };
        input.properties.receive_maximum = Some(ReceiveMaximum(128u16));

        let mut buf = [0u8; 16];
        let result = input.try_to_byte_buffer(&mut buf).unwrap();

        assert_eq!(result, EXPECTED);
    }
}
