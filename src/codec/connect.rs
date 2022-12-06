use bytes::BytesMut;

use crate::core::{
    base_types::*,
    error::{CodecError, UnexpectedProperty},
    properties::*,
    utils::{ByteLen, Encode, Encoder, PacketID, SizedPacket},
};
use core::mem;
use derive_builder::Builder;

#[derive(Builder)]
#[builder(build_fn(error = "CodecError", validate = "Self::validate"))]
pub(crate) struct ConnectTx<'a> {
    #[builder(default)]
    keep_alive: u16,

    #[builder(setter(strip_option), default)]
    session_expiry_interval: Option<SessionExpiryInterval>,
    #[builder(setter(strip_option), default)]
    receive_maximum: Option<ReceiveMaximum>,
    #[builder(setter(strip_option), default)]
    maximum_packet_size: Option<MaximumPacketSize>,
    #[builder(setter(strip_option), default)]
    topic_alias_maximum: Option<TopicAliasMaximum>,
    #[builder(setter(strip_option), default)]
    request_response_information: Option<RequestResponseInformation>,
    #[builder(setter(strip_option), default)]
    request_problem_information: Option<RequestProblemInformation>,
    #[builder(setter(strip_option), default)]
    authentication_method: Option<AuthenticationMethodRef<'a>>,
    #[builder(setter(strip_option), default)]
    authentication_data: Option<AuthenticationDataRef<'a>>,
    #[builder(setter(custom), default)]
    user_property: Vec<UserPropertyRef<'a>>,

    #[builder(default)]
    will_qos: QoS,
    #[builder(default)]
    will_retain: bool,
    #[builder(default)]
    clean_start: bool,

    client_identifier: UTF8StringRef<'a>,

    #[builder(setter(strip_option), default)]
    will_delay_interval: Option<WillDelayInterval>,
    #[builder(setter(strip_option), default)]
    will_payload_format_indicator: Option<PayloadFormatIndicator>,
    #[builder(setter(strip_option), default)]
    will_message_expiry_interval: Option<MessageExpiryInterval>,
    #[builder(setter(strip_option), default)]
    will_content_type: Option<ContentTypeRef<'a>>,
    #[builder(setter(strip_option), default)]
    will_reponse_topic: Option<ResponseTopicRef<'a>>,
    #[builder(setter(strip_option), default)]
    will_correlation_data: Option<CorrelationDataRef<'a>>,
    #[builder(setter(custom), default)]
    will_user_property: Vec<UserPropertyRef<'a>>,

    #[builder(setter(strip_option), default)]
    will_topic: Option<UTF8StringRef<'a>>,
    #[builder(setter(strip_option), default)]
    will_payload: Option<BinaryRef<'a>>,
    #[builder(setter(strip_option), default)]
    username: Option<UTF8StringRef<'a>>,
    #[builder(setter(strip_option), default)]
    password: Option<BinaryRef<'a>>,
}

impl<'a> ConnectTxBuilder<'a> {
    fn validate(&self) -> Result<(), CodecError> {
        if self.authentication_method.is_none() && self.authentication_data.is_some() {
            Err(UnexpectedProperty.into()) // Cannot include authentication data when authentication method is absent.
        } else {
            Ok(())
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

    pub(crate) fn will_user_property(&mut self, value: UserPropertyRef<'a>) {
        match self.will_user_property.as_mut() {
            Some(will_user_property) => {
                will_user_property.push(value);
            }
            None => {
                self.will_user_property = Some(Vec::new());
                self.will_user_property.as_mut().unwrap().push(value);
            }
        }
    }
}

impl<'a> ConnectTx<'a> {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;
    const PROTOCOL_NAME: UTF8StringRef<'static> = UTF8StringRef("MQTT");
    const PROTOCOL_VERSION: u8 = 5;

    fn property_len(&self) -> VarSizeInt {
        VarSizeInt::try_from(
            self.session_expiry_interval
                .as_ref()
                .map(ByteLen::byte_len)
                .unwrap_or(0)
                + self
                    .maximum_packet_size
                    .as_ref()
                    .map(ByteLen::byte_len)
                    .unwrap_or(0)
                + self
                    .topic_alias_maximum
                    .as_ref()
                    .map(ByteLen::byte_len)
                    .unwrap_or(0)
                + self
                    .request_response_information
                    .as_ref()
                    .map(ByteLen::byte_len)
                    .unwrap_or(0)
                + self
                    .request_problem_information
                    .as_ref()
                    .map(ByteLen::byte_len)
                    .unwrap_or(0)
                + self
                    .authentication_method
                    .as_ref()
                    .map(ByteLen::byte_len)
                    .unwrap_or(0)
                + self
                    .authentication_data
                    .as_ref()
                    .map(ByteLen::byte_len)
                    .unwrap_or(0)
                + self
                    .user_property
                    .iter()
                    .map(ByteLen::byte_len)
                    .sum::<usize>(),
        )
        .unwrap()
    }

    fn will_property_len(&self) -> VarSizeInt {
        VarSizeInt::try_from(
            self.will_delay_interval
                .as_ref()
                .map(ByteLen::byte_len)
                .unwrap_or(0)
                + self
                    .will_payload_format_indicator
                    .as_ref()
                    .map(ByteLen::byte_len)
                    .unwrap_or(0)
                + self
                    .will_message_expiry_interval
                    .as_ref()
                    .map(ByteLen::byte_len)
                    .unwrap_or(0)
                + self
                    .will_content_type
                    .as_ref()
                    .map(ByteLen::byte_len)
                    .unwrap_or(0)
                + self
                    .will_reponse_topic
                    .as_ref()
                    .map(ByteLen::byte_len)
                    .unwrap_or(0)
                + self
                    .will_correlation_data
                    .as_ref()
                    .map(ByteLen::byte_len)
                    .unwrap_or(0)
                + self
                    .user_property
                    .iter()
                    .map(ByteLen::byte_len)
                    .sum::<usize>(),
        )
        .unwrap()
    }

    fn payload_len(&self) -> usize {
        let payload_remaining_len = self.client_identifier.byte_len()
            + self.username.as_ref().map(ByteLen::byte_len).unwrap_or(0)
            + self.password.as_ref().map(ByteLen::byte_len).unwrap_or(0);

        if self.will_flag() != 0 {
            let will_properties_len = self.will_property_len();
            will_properties_len.len()
                + will_properties_len.value() as usize
                + payload_remaining_len
                + self.will_topic.as_ref().map(ByteLen::byte_len).unwrap_or(0)
                + self
                    .will_payload
                    .as_ref()
                    .map(ByteLen::byte_len)
                    .unwrap_or(0)
        } else {
            payload_remaining_len
        }
    }

    fn remaining_len(&self) -> VarSizeInt {
        const CONNECT_FLAGS_LEN: usize = mem::size_of::<u8>();
        let property_len = self.property_len();

        VarSizeInt::try_from(
            Self::PROTOCOL_NAME.byte_len()
                + Self::PROTOCOL_VERSION.byte_len()
                + CONNECT_FLAGS_LEN
                + self.keep_alive.byte_len()
                + property_len.len()
                + property_len.value() as usize
                + self.payload_len(),
        )
        .unwrap()
    }

    fn will_flag(&self) -> u8 {
        self.will_topic
            .as_ref()
            .and(self.will_payload.as_ref())
            .map(|_| 1)
            .unwrap_or(0)
    }

    fn payload_flags(&self) -> u8 {
        (self.username.as_ref().map(|_| 1).unwrap_or(0) << 7)
            | (self.password.as_ref().map(|_| 1).unwrap_or(0) << 6)
            | ((self.will_retain as u8) << 5)
            | ((self.will_qos as u8) << 3)
            | (self.will_flag() << 2)
            | ((self.clean_start as u8) << 1)
    }
}

impl<'a> PacketID for ConnectTx<'a> {
    const PACKET_ID: u8 = 1;
}

impl<'a> SizedPacket for ConnectTx<'a> {
    fn packet_len(&self) -> usize {
        let remaining_len = self.remaining_len();
        mem::size_of_val(&Self::FIXED_HDR) + remaining_len.len() + remaining_len.value() as usize
    }
}

impl<'a> Encode for ConnectTx<'a> {
    fn encode(&self, buf: &mut BytesMut) {
        let mut encoder = Encoder::from(buf);

        let will_flag = self.will_flag();
        let remaining_len = self.remaining_len();

        encoder.encode(Self::FIXED_HDR);

        encoder.encode(remaining_len);

        encoder.encode(Self::PROTOCOL_NAME);
        encoder.encode(Self::PROTOCOL_VERSION);
        encoder.encode(self.payload_flags());
        encoder.encode(self.keep_alive);

        // Properties

        encoder.encode(self.property_len());

        if let Some(val) = self.session_expiry_interval {
            encoder.encode(val)
        }

        if let Some(val) = self.receive_maximum {
            encoder.encode(val)
        }

        if let Some(val) = self.maximum_packet_size {
            encoder.encode(val)
        }

        if let Some(val) = self.topic_alias_maximum {
            encoder.encode(val)
        }

        if let Some(val) = self.request_response_information {
            encoder.encode(val)
        }

        if let Some(val) = self.request_problem_information {
            encoder.encode(val)
        }

        if let Some(val) = self.authentication_method {
            encoder.encode(val)
        }

        if let Some(val) = self.authentication_data {
            encoder.encode(val)
        }

        for val in self.user_property.iter().copied() {
            encoder.encode(val)
        }

        // Payload

        encoder.encode(self.client_identifier);

        if will_flag != 0 {
            encoder.encode(self.will_property_len());

            if let Some(val) = self.will_delay_interval {
                encoder.encode(val)
            }

            if let Some(val) = self.will_payload_format_indicator {
                encoder.encode(val)
            }

            if let Some(val) = self.will_message_expiry_interval {
                encoder.encode(val)
            }

            if let Some(val) = self.will_content_type {
                encoder.encode(val)
            }

            if let Some(val) = self.will_reponse_topic {
                encoder.encode(val)
            }

            if let Some(val) = self.will_correlation_data {
                encoder.encode(val)
            }
        }

        for val in self.user_property.iter().copied() {
            encoder.encode(val)
        }

        if will_flag != 0 {
            if let Some(val) = self.will_topic {
                encoder.encode(val)
            }

            if let Some(val) = self.will_payload {
                encoder.encode(val)
            }
        }

        if let Some(val) = self.username {
            encoder.encode(val)
        }

        if let Some(val) = self.password {
            encoder.encode(val)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_bytes_0() {
        const EXPECTED: [u8; 22] = [
            ConnectTx::FIXED_HDR,
            20,
            0,
            4,
            b'M',
            b'Q',
            b'T',
            b'T',
            ConnectTx::PROTOCOL_VERSION,
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

        let mut builder = ConnectTxBuilder::default();
        builder.client_identifier(UTF8StringRef("test123"));
        let packet = builder.build().unwrap();

        let mut buf = BytesMut::new();
        packet.encode(&mut buf);

        assert_eq!(&buf.split().freeze()[..], &EXPECTED[..]);
    }
}
