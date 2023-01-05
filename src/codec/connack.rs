use crate::core::{
    base_types::*,
    collections::UserProperties,
    error::{
        CodecError, ConversionError, InvalidPacketHeader, InvalidPacketSize, InvalidPropertyLength,
        InvalidValue, UnexpectedProperty,
    },
    properties::*,
    utils::{ByteLen, Decoder, PacketID, TryDecode},
};
use bytes::Bytes;
use core::mem;
use derive_builder::Builder;

/// Reason for CONNACK packet.
///
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ConnectReason {
    Success = 0x00,
    UnspecifiedError = 0x80,
    MalformedPacket = 0x81,
    ProtocolError = 0x82,
    ImplementationSpecificError = 0x83,
    UnsupportedProtocolVersion = 0x84,
    ClientIdentifierNotValid = 0x85,
    BadUserNameOrPassword = 0x86,
    NotAuthorized = 0x87,
    ServerUnavailable = 0x88,
    ServerBusy = 0x89,
    Banned = 0x8a,
    BadUthenticationMethod = 0x8c,
    TopicNameInvalid = 0x90,
    PacketTooLarge = 0x95,
    QuotaExceeded = 0x97,
    PayloadFormatInvalid = 0x99,
    RetainNotSupported = 0x9a,
    QoSNotSupported = 0x9b,
    UseAnotherServer = 0x9c,
    ServerMoved = 0x9d,
    ConnectionRateExceeded = 0x9f,
}

impl TryFrom<u8> for ConnectReason {
    type Error = ConversionError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x00 => Ok(ConnectReason::Success),
            0x80 => Ok(ConnectReason::UnspecifiedError),
            0x81 => Ok(ConnectReason::MalformedPacket),
            0x82 => Ok(ConnectReason::ProtocolError),
            0x83 => Ok(ConnectReason::ImplementationSpecificError),
            0x84 => Ok(ConnectReason::UnsupportedProtocolVersion),
            0x85 => Ok(ConnectReason::ClientIdentifierNotValid),
            0x86 => Ok(ConnectReason::BadUserNameOrPassword),
            0x87 => Ok(ConnectReason::NotAuthorized),
            0x88 => Ok(ConnectReason::ServerUnavailable),
            0x89 => Ok(ConnectReason::ServerBusy),
            0x8a => Ok(ConnectReason::Banned),
            0x8c => Ok(ConnectReason::BadUthenticationMethod),
            0x90 => Ok(ConnectReason::TopicNameInvalid),
            0x95 => Ok(ConnectReason::PacketTooLarge),
            0x97 => Ok(ConnectReason::QuotaExceeded),
            0x99 => Ok(ConnectReason::PayloadFormatInvalid),
            0x9a => Ok(ConnectReason::RetainNotSupported),
            0x9b => Ok(ConnectReason::QoSNotSupported),
            0x9c => Ok(ConnectReason::UseAnotherServer),
            0x9d => Ok(ConnectReason::ServerMoved),
            0x9f => Ok(ConnectReason::ConnectionRateExceeded),
            _ => Err(InvalidValue.into()),
        }
    }
}

impl Default for ConnectReason {
    fn default() -> Self {
        Self::Success
    }
}

impl ByteLen for ConnectReason {
    fn byte_len(&self) -> usize {
        (*self as u8).byte_len()
    }
}

impl TryDecode for ConnectReason {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        Self::try_from(u8::try_decode(bytes)?)
    }
}

#[derive(Builder, Clone)]
#[builder(build_fn(error = "CodecError"))]
pub(crate) struct ConnackRx {
    // Connack variable header
    pub(crate) session_present: bool,
    pub(crate) reason: ConnectReason,

    // Connack properties
    #[builder(default)]
    pub(crate) wildcard_subscription_available: WildcardSubscriptionAvailable,
    #[builder(default)]
    pub(crate) subscription_identifier_available: SubscriptionIdentifierAvailable,
    #[builder(default)]
    pub(crate) shared_subscription_available: SharedSubscriptionAvailable,
    #[builder(default)]
    pub(crate) maximum_qos: MaximumQoS,
    #[builder(default)]
    pub(crate) retain_available: RetainAvailable,
    #[builder(setter(strip_option), default)]
    pub(crate) server_keep_alive: Option<ServerKeepAlive>,
    #[builder(default)]
    pub(crate) receive_maximum: ReceiveMaximum,
    #[builder(default)]
    pub(crate) topic_alias_maximum: TopicAliasMaximum,
    #[builder(setter(strip_option), default)]
    pub(crate) session_expiry_interval: Option<SessionExpiryInterval>,
    #[builder(setter(strip_option), default)]
    pub(crate) maximum_packet_size: Option<MaximumPacketSize>,
    #[builder(setter(strip_option), default)]
    pub(crate) authentication_data: Option<AuthenticationData>,
    #[builder(setter(strip_option), default)]
    pub(crate) assigned_client_identifier: Option<AssignedClientIdentifier>,
    #[builder(setter(strip_option), default)]
    pub(crate) reason_string: Option<ReasonString>,
    #[builder(setter(strip_option), default)]
    pub(crate) response_information: Option<ResponseInformation>,
    #[builder(setter(strip_option), default)]
    pub(crate) server_reference: Option<ServerReference>,
    #[builder(setter(strip_option), default)]
    pub(crate) authentication_method: Option<AuthenticationMethod>,
    #[builder(setter(custom), default)]
    pub(crate) user_property: UserProperties,
}

impl ConnackRxBuilder {
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

impl PacketID for ConnackRx {
    const PACKET_ID: u8 = 2;
}

impl TryDecode for ConnackRx {
    type Error = CodecError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut builder = ConnackRxBuilder::default();
        let mut decoder = Decoder::from(bytes.clone());

        let fixed_hdr = decoder
            .try_decode::<u8>()
            .map_err(CodecError::from)
            .and_then(|val| {
                if val >> 4 != Self::PACKET_ID {
                    return Err(InvalidPacketHeader.into());
                }

                Ok(val)
            })?;

        let remaining_len = decoder.try_decode::<VarSizeInt>()?;
        let packet_size =
            mem::size_of_val(&fixed_hdr) + remaining_len.len() + remaining_len.value() as usize;
        if packet_size > bytes.len() {
            return Err(InvalidPacketSize.into());
        }

        let session_present = decoder.try_decode::<bool>()?;
        builder.session_present(session_present);

        let reason = decoder.try_decode::<ConnectReason>()?;
        builder.reason(reason);

        let byte_len = decoder.try_decode::<VarSizeInt>()?;
        if byte_len.value() as usize > decoder.remaining() {
            return Err(InvalidPropertyLength.into());
        }

        for maybe_property in decoder.iter::<Property>() {
            match maybe_property {
                Ok(property) => match property {
                    Property::WildcardSubscriptionAvailable(val) => {
                        builder.wildcard_subscription_available(val);
                    }
                    Property::SubscriptionIdentifierAvailable(val) => {
                        builder.subscription_identifier_available(val);
                    }
                    Property::SharedSubscriptionAvailable(val) => {
                        builder.shared_subscription_available(val);
                    }
                    Property::MaximumQoS(val) => {
                        builder.maximum_qos(val);
                    }
                    Property::RetainAvailable(val) => {
                        builder.retain_available(val);
                    }
                    Property::ServerKeepAlive(val) => {
                        builder.server_keep_alive(val);
                    }
                    Property::ReceiveMaximum(val) => {
                        builder.receive_maximum(val);
                    }
                    Property::TopicAliasMaximum(val) => {
                        builder.topic_alias_maximum(val);
                    }
                    Property::SessionExpiryInterval(val) => {
                        builder.session_expiry_interval(val);
                    }
                    Property::MaximumPacketSize(val) => {
                        builder.maximum_packet_size(val);
                    }
                    Property::AuthenticationData(val) => {
                        builder.authentication_data(val);
                    }
                    Property::AssignedClientIdentifier(val) => {
                        builder.assigned_client_identifier(val);
                    }
                    Property::ReasonString(val) => {
                        builder.reason_string(val);
                    }
                    Property::ResponseInformation(val) => {
                        builder.response_information(val);
                    }
                    Property::ServerReference(val) => {
                        builder.server_reference(val);
                    }
                    Property::AuthenticationMethod(val) => {
                        builder.authentication_method(val);
                    }
                    Property::UserProperty(val) => {
                        builder.user_property(val);
                    }
                    _ => {
                        return Err(UnexpectedProperty.into());
                    }
                },
                Err(err) => return Err(err.into()),
            }
        }

        builder.build()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes_0() {
        const PACKET: [u8; 5] = [
            ConnackRx::PACKET_ID << 4, // Fixed header
            3,                         // Remaining length
            0,                         // Connect Acknowledge Flags (No session present)
            0,                         // Reason (Success)
            0,                         // Property length
        ];

        let result = ConnackRx::try_decode(Bytes::from_static(&PACKET)).unwrap();

        assert!(!result.session_present);
        assert_eq!(result.reason, ConnectReason::Success);
        assert_eq!(result.receive_maximum, ReceiveMaximum::default());
        assert_eq!(result.topic_alias_maximum, TopicAliasMaximum::from(0));
        assert_eq!(result.maximum_qos, MaximumQoS::from(QoS::ExactlyOnce));
        assert_eq!(result.retain_available, RetainAvailable::from(true));
        assert_eq!(result.maximum_qos, MaximumQoS::from(QoS::ExactlyOnce));
        assert!(result.maximum_packet_size.is_none());
        assert_eq!(
            result.wildcard_subscription_available,
            WildcardSubscriptionAvailable::from(true)
        );
        assert_eq!(
            result.subscription_identifier_available,
            SubscriptionIdentifierAvailable::from(true)
        );
        assert_eq!(
            result.shared_subscription_available,
            SharedSubscriptionAvailable::from(true)
        );
    }

    #[test]
    fn from_bytes_1() {
        const PACKET: [u8; 14] = [
            ConnackRx::PACKET_ID << 4,
            12,
            0,
            0,
            9,
            34,
            0,
            10,
            19,
            255,
            255,
            33,
            0,
            20,
        ];

        let result = ConnackRx::try_decode(Bytes::from_static(&PACKET)).unwrap();

        assert!(!result.session_present);
        assert!(result.maximum_packet_size.is_none());
        assert_eq!(result.reason, ConnectReason::Success);
        assert_eq!(
            result.receive_maximum,
            ReceiveMaximum::from(NonZero::try_from(20).unwrap())
        );
        assert_eq!(result.topic_alias_maximum, TopicAliasMaximum::from(10));
        assert_eq!(result.maximum_qos, MaximumQoS::from(QoS::ExactlyOnce));
        assert_eq!(
            result.server_keep_alive,
            Some(ServerKeepAlive::from(0xffff))
        );
        assert_eq!(result.retain_available, RetainAvailable::from(true));
        assert_eq!(result.maximum_qos, MaximumQoS::from(QoS::ExactlyOnce));
        assert_eq!(
            result.wildcard_subscription_available,
            WildcardSubscriptionAvailable::from(true)
        );
        assert_eq!(
            result.subscription_identifier_available,
            SubscriptionIdentifierAvailable::from(true)
        );
        assert_eq!(
            result.shared_subscription_available,
            SharedSubscriptionAvailable::from(true)
        );
    }

    #[test]
    fn from_bytes_2() {
        const PACKET: [u8; 65] = [
            ConnackRx::PACKET_ID << 4, // Fixed header
            63,                        // Remaining length
            0x00,                      // Connect Acknowledge Flags (No session present)
            0x00,                      // Reason (Success)
            60,                        // Property length
            0x11,                      // Session Expiry Interval
            0x00,
            0x00,
            0x03,
            0x84, // 900 seconds
            0x21, // Receive maximum
            0x4e,
            0x20, // 20 000
            0x24, // Maximum QoS
            0x01, // 1
            0x25, // Retain available
            0x01, // Yes
            0x27, // Maximum packet size
            0x00,
            0x00,
            0x01,
            0x00, // 256 bytes
            0x12, // Assigned client identifier
            0x00, // String length MSB
            0x04, // String length LSB
            b't',
            b'e',
            b's',
            b't',
            0x22, // Topic alias maximum
            0x00,
            0x0a, // 10
            0x1f, // Reason String
            0x00, // String length MSB
            0x07, // String length LSB
            b's',
            b'u',
            b'c',
            b'c',
            b'e',
            b's',
            b's',
            0x28, // Wildcard subscription available
            0x01, // Yes
            0x29, // Subscription identifiers available
            0x01, // Yes
            0x2a, // Shared subscription avaialble
            0x01, // Yes
            0x13, // Server keep alive
            0x00,
            0x64, // 100 seconds
            0x1a, // Response information
            0x00, // String length MSB
            0x04, // String length LSB
            b't',
            b'e',
            b's',
            b't',
            0x1c, // Server reference
            0x00, // String length MSB
            0x04, // String length LSB
            b't',
            b'e',
            b's',
            b't',
        ];

        // User property, authentication method and authentication data properties are not present.

        let result = ConnackRx::try_decode(Bytes::from_static(&PACKET)).unwrap();

        assert!(result.user_property.is_empty());
        assert!(result.authentication_data.is_none());
        assert!(result.authentication_method.is_none());
        assert!(!result.session_present);
        assert_eq!(result.reason, ConnectReason::Success);
        assert_eq!(
            result.session_expiry_interval.unwrap(),
            SessionExpiryInterval::from(900)
        );
        assert_eq!(
            result.receive_maximum,
            ReceiveMaximum::from(NonZero::try_from(20000).unwrap())
        );
        assert_eq!(result.maximum_qos, MaximumQoS::from(QoS::AtLeastOnce));
        assert_eq!(result.retain_available, RetainAvailable::from(true));
        assert_eq!(
            result.maximum_packet_size,
            Some(MaximumPacketSize::from(NonZero::try_from(256).unwrap()))
        );
        assert_eq!(
            result.assigned_client_identifier,
            Some(AssignedClientIdentifier::from(UTF8String(
                Bytes::from_static("test".as_bytes())
            )))
        );
        assert_eq!(result.topic_alias_maximum, TopicAliasMaximum::from(10));
        assert_eq!(
            result.reason_string,
            Some(ReasonString::from(UTF8String(Bytes::from_static(
                "success".as_bytes()
            ))))
        );
        assert_eq!(
            result.wildcard_subscription_available,
            WildcardSubscriptionAvailable::from(true)
        );
        assert_eq!(
            result.subscription_identifier_available,
            SubscriptionIdentifierAvailable::from(true)
        );
        assert_eq!(
            result.shared_subscription_available,
            SharedSubscriptionAvailable::from(true)
        );
        assert_eq!(result.server_keep_alive, Some(ServerKeepAlive::from(100)));
        assert_eq!(
            result.response_information,
            Some(ResponseInformation::from(UTF8String(Bytes::from_static(
                "test".as_bytes()
            ))))
        );
        assert_eq!(
            result.server_reference,
            Some(ServerReference::from(UTF8String(Bytes::from_static(
                "test".as_bytes()
            ))))
        );
    }
}
