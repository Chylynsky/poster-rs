use crate::core::{
    base_types::*,
    properties::*,
    utils::{ByteReader, PacketID, SizedProperty, TryFromBytes},
};
use core::mem;

#[derive(Copy, Clone, PartialEq, Debug)]
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

impl ConnectReason {
    fn try_from(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(ConnectReason::Success),
            0x80 => Some(ConnectReason::UnspecifiedError),
            0x81 => Some(ConnectReason::MalformedPacket),
            0x82 => Some(ConnectReason::ProtocolError),
            0x83 => Some(ConnectReason::ImplementationSpecificError),
            0x84 => Some(ConnectReason::UnsupportedProtocolVersion),
            0x85 => Some(ConnectReason::ClientIdentifierNotValid),
            0x86 => Some(ConnectReason::BadUserNameOrPassword),
            0x87 => Some(ConnectReason::NotAuthorized),
            0x88 => Some(ConnectReason::ServerUnavailable),
            0x89 => Some(ConnectReason::ServerBusy),
            0x8a => Some(ConnectReason::Banned),
            0x8c => Some(ConnectReason::BadUthenticationMethod),
            0x90 => Some(ConnectReason::TopicNameInvalid),
            0x95 => Some(ConnectReason::PacketTooLarge),
            0x97 => Some(ConnectReason::QuotaExceeded),
            0x99 => Some(ConnectReason::PayloadFormatInvalid),
            0x9a => Some(ConnectReason::RetainNotSupported),
            0x9b => Some(ConnectReason::QoSNotSupported),
            0x9c => Some(ConnectReason::UseAnotherServer),
            0x9d => Some(ConnectReason::ServerMoved),
            0x9f => Some(ConnectReason::ConnectionRateExceeded),
            _ => None,
        }
    }
}

impl Default for ConnectReason {
    fn default() -> Self {
        Self::Success
    }
}

impl SizedProperty for ConnectReason {
    fn property_len(&self) -> usize {
        (*self as u8).property_len()
    }
}

impl TryFromBytes for ConnectReason {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        Self::try_from(u8::try_from_bytes(bytes)?)
    }
}

pub(crate) struct Connack {
    // Connack variable header
    pub(crate) session_present: bool,
    pub(crate) reason: ConnectReason,

    // Connack properties
    pub(crate) wildcard_subscription_available: WildcardSubscriptionAvailable,
    pub(crate) subscription_identifier_available: SubscriptionIdentifierAvailable,
    pub(crate) shared_subscription_available: SharedSubscriptionAvailable,
    pub(crate) maximum_qos: MaximumQoS,
    pub(crate) retain_available: RetainAvailable,

    pub(crate) server_keep_alive: Option<ServerKeepAlive>,
    pub(crate) receive_maximum: ReceiveMaximum,
    pub(crate) topic_alias_maximum: TopicAliasMaximum,

    pub(crate) session_expiry_interval: SessionExpiryInterval,
    pub(crate) maximum_packet_size: Option<MaximumPacketSize>,

    pub(crate) authentication_data: Option<AuthenticationData>,

    pub(crate) assigned_client_identifier: Option<AssignedClientIdentifier>,
    pub(crate) reason_string: Option<ReasonString>,
    pub(crate) response_information: Option<ResponseInformation>,
    pub(crate) server_reference: Option<ServerReference>,
    pub(crate) authentication_method: Option<AuthenticationMethod>,

    pub(crate) user_property: Vec<UserProperty>,
}

impl PacketID for Connack {
    const PACKET_ID: u8 = 2;
}

impl TryFromBytes for Connack {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        let mut builder = ConnackBuilder::default();
        let mut reader = ByteReader::from(bytes);

        let fixed_hdr = reader.try_read::<u8>()?;
        if fixed_hdr >> 4 != Self::PACKET_ID {
            return None; // Invalid header
        }

        let remaining_len = reader.try_read::<VarSizeInt>()?;
        let packet_size =
            mem::size_of_val(&fixed_hdr) + remaining_len.len() + remaining_len.value() as usize;
        if packet_size > bytes.len() {
            return None; // Invalid packet size
        }

        let session_present = reader.try_read::<bool>()?;
        builder.session_present(session_present);

        let reason = reader.try_read::<ConnectReason>()?;
        builder.reason(reason);

        let property_len = reader.try_read::<VarSizeInt>()?;
        if property_len.value() as usize > reader.remaining() {
            return None; // Invalid property length
        }

        for property in PropertyIterator::from(reader.get_buf()) {
            match property {
                Property::WildcardSubscriptionAvailable(val) => {
                    builder.wildcard_subscription_available(val.into());
                }
                Property::SubscriptionIdentifierAvailable(val) => {
                    builder.subscription_identifier_available(val.into());
                }
                Property::SharedSubscriptionAvailable(val) => {
                    builder.shared_subscription_available(val.into());
                }
                Property::MaximumQoS(val) => {
                    builder.maximum_qos(val.into());
                }
                Property::RetainAvailable(val) => {
                    builder.retain_available(val.into());
                }
                Property::ServerKeepAlive(val) => {
                    builder.server_keep_alive(val.into());
                }
                Property::ReceiveMaximum(val) => {
                    builder.receive_maximum(val.into());
                }
                Property::TopicAliasMaximum(val) => {
                    builder.topic_alias_maximum(val.into());
                }
                Property::SessionExpiryInterval(val) => {
                    builder.session_expiry_interval(val.into());
                }
                Property::MaximumPacketSize(val) => {
                    builder.maximum_packet_size(val.into());
                }
                Property::AuthenticationData(val) => {
                    builder.authentication_data(val.into());
                }
                Property::AssignedClientIdentifier(val) => {
                    builder.assigned_client_identifier(val.into());
                }
                Property::ReasonString(val) => {
                    builder.reason_string(val.into());
                }
                Property::ResponseInformation(val) => {
                    builder.response_information(val.into());
                }
                Property::ServerReference(val) => {
                    builder.server_reference(val.into());
                }
                Property::AuthenticationMethod(val) => {
                    builder.authentication_method(val.into());
                }
                Property::UserProperty(val) => {
                    builder.user_property(val.into());
                }
                _ => {
                    return None;
                }
            }
        }

        builder.build()
    }
}

#[derive(Default)]
struct ConnackBuilder {
    // Connack variable header
    session_present: Option<bool>,
    reason: Option<ConnectReason>,

    // Connack properties
    wildcard_subscription_available: WildcardSubscriptionAvailable,
    subscription_identifier_available: SubscriptionIdentifierAvailable,
    shared_subscription_available: SharedSubscriptionAvailable,
    maximum_qos: MaximumQoS,
    retain_available: RetainAvailable,

    server_keep_alive: Option<ServerKeepAlive>,
    receive_maximum: ReceiveMaximum,
    topic_alias_maximum: TopicAliasMaximum,

    session_expiry_interval: SessionExpiryInterval,
    maximum_packet_size: Option<MaximumPacketSize>,

    authentication_data: Option<AuthenticationData>,

    assigned_client_identifier: Option<AssignedClientIdentifier>,
    reason_string: Option<ReasonString>,
    response_information: Option<ResponseInformation>,
    server_reference: Option<ServerReference>,
    authentication_method: Option<AuthenticationMethod>,

    user_property: Vec<UserProperty>,
}

impl ConnackBuilder {
    fn session_present(&mut self, val: bool) -> &mut Self {
        self.session_present = Some(val);
        self
    }

    fn reason(&mut self, val: ConnectReason) -> &mut Self {
        self.reason = Some(val);
        self
    }

    fn wildcard_subscription_available(&mut self, val: bool) -> &mut Self {
        self.wildcard_subscription_available = val.into();
        self
    }

    fn subscription_identifier_available(&mut self, val: bool) -> &mut Self {
        self.subscription_identifier_available = val.into();
        self
    }

    fn shared_subscription_available(&mut self, val: bool) -> &mut Self {
        self.shared_subscription_available = val.into();
        self
    }

    fn maximum_qos(&mut self, val: QoS) -> &mut Self {
        self.maximum_qos = val.into();
        self
    }

    fn retain_available(&mut self, val: bool) -> &mut Self {
        self.retain_available = val.into();
        self
    }

    fn server_keep_alive(&mut self, val: u16) -> &mut Self {
        self.server_keep_alive = Some(val.into());
        self
    }

    fn receive_maximum(&mut self, val: NonZero<u16>) -> &mut Self {
        self.receive_maximum = val.into();
        self
    }

    fn topic_alias_maximum(&mut self, val: u16) -> &mut Self {
        self.topic_alias_maximum = val.into();
        self
    }

    fn session_expiry_interval(&mut self, val: u32) -> &mut Self {
        self.session_expiry_interval = val.into();
        self
    }

    fn maximum_packet_size(&mut self, val: NonZero<u32>) -> &mut Self {
        self.maximum_packet_size = Some(val.into());
        self
    }

    fn authentication_data(&mut self, val: Binary) -> &mut Self {
        self.authentication_data = Some(val.into());
        self
    }

    fn assigned_client_identifier(&mut self, val: String) -> &mut Self {
        self.assigned_client_identifier = Some(val.into());
        self
    }

    fn reason_string(&mut self, val: String) -> &mut Self {
        self.reason_string = Some(val.into());
        self
    }

    fn response_information(&mut self, val: String) -> &mut Self {
        self.response_information = Some(val.into());
        self
    }
    fn server_reference(&mut self, val: String) -> &mut Self {
        self.server_reference = Some(val.into());
        self
    }

    fn authentication_method(&mut self, val: String) -> &mut Self {
        self.authentication_method = Some(val.into());
        self
    }

    fn user_property(&mut self, val: StringPair) -> &mut Self {
        self.user_property.push(val.into());
        self
    }

    fn build(self) -> Option<Connack> {
        Some(Connack {
            session_present: self.session_present?,
            reason: self.reason?,
            wildcard_subscription_available: self.wildcard_subscription_available,
            subscription_identifier_available: self.subscription_identifier_available,
            shared_subscription_available: self.shared_subscription_available,
            maximum_qos: self.maximum_qos,
            retain_available: self.retain_available,
            server_keep_alive: self.server_keep_alive,
            receive_maximum: self.receive_maximum,
            topic_alias_maximum: self.topic_alias_maximum,
            session_expiry_interval: self.session_expiry_interval,
            maximum_packet_size: self.maximum_packet_size,
            authentication_data: self.authentication_data,
            assigned_client_identifier: self.assigned_client_identifier,
            reason_string: self.reason_string,
            response_information: self.response_information,
            server_reference: self.server_reference,
            authentication_method: self.authentication_method,
            user_property: self.user_property,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes_0() {
        const PACKET: [u8; 5] = [
            Connack::PACKET_ID << 4, // Fixed header
            3,                       // Remaining length
            0,                       // Connect Acknowledge Flags (No session present)
            0,                       // Reason (Success)
            0,                       // Property length
        ];

        let result = Connack::try_from_bytes(&PACKET).unwrap();

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
            Connack::PACKET_ID << 4,
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

        let result = Connack::try_from_bytes(&PACKET).unwrap();

        assert!(!result.session_present);
        assert!(result.maximum_packet_size.is_none());
        assert_eq!(result.reason, ConnectReason::Success);
        assert_eq!(
            result.receive_maximum,
            ReceiveMaximum::from(NonZero::from(20))
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
            Connack::PACKET_ID << 4, // Fixed header
            63,                      // Remaining length
            0x00,                    // Connect Acknowledge Flags (No session present)
            0x00,                    // Reason (Success)
            60,                      // Property length
            0x11,                    // Session Expiry Interval
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

        let result = Connack::try_from_bytes(&PACKET).unwrap();

        assert!(result.user_property.is_empty());
        assert!(result.authentication_data.is_none());
        assert!(result.authentication_method.is_none());
        assert!(!result.session_present);
        assert_eq!(result.reason, ConnectReason::Success);
        assert_eq!(
            result.session_expiry_interval,
            SessionExpiryInterval::from(900)
        );
        assert_eq!(
            result.receive_maximum,
            ReceiveMaximum::from(NonZero::from(20000))
        );
        assert_eq!(result.maximum_qos, MaximumQoS::from(QoS::AtLeastOnce));
        assert_eq!(result.retain_available, RetainAvailable::from(true));
        assert_eq!(
            result.maximum_packet_size,
            Some(MaximumPacketSize::from(NonZero::from(256)))
        );
        assert_eq!(
            result.assigned_client_identifier,
            Some(AssignedClientIdentifier::from(String::from("test")))
        );
        assert_eq!(result.topic_alias_maximum, TopicAliasMaximum::from(10));
        assert_eq!(
            result.reason_string,
            Some(ReasonString::from(String::from("success")))
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
            Some(ResponseInformation::from(String::from("test")))
        );
        assert_eq!(
            result.server_reference,
            Some(ServerReference::from(String::from("test")))
        );
    }
}
