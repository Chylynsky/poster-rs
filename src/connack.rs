use crate::{
    base_types::*,
    properties::*,
    utils::{ByteReader, PacketID, SizedProperty, TryFromBytes, TryFromIterator},
};
use std::mem;

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum ConnectReason {
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
        (*self as Byte).property_len()
    }
}

impl TryFromBytes for ConnectReason {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        Self::try_from(Byte::try_from_bytes(bytes)?)
    }
}

pub(crate) struct Connack {
    // Connack variable header
    session_present: bool,
    reason: ConnectReason,

    // Connack properties
    wildcard_subscription_available: Option<WildcardSubscriptionAvailable>,
    subscription_identifier_available: Option<SubscriptionIdentifierAvailable>,
    shared_subscription_available: Option<SharedSubscriptionAvailable>,
    maximum_qos: Option<MaximumQoS>,
    retain_available: Option<RetainAvailable>,

    server_keep_alive: Option<ServerKeepAlive>,
    receive_maximum: Option<ReceiveMaximum>,
    topic_alias_maximum: Option<TopicAliasMaximum>,

    session_expiry_interval: Option<SessionExpiryInterval>,
    maximum_packet_size: Option<MaximumPacketSize>,

    authentication_data: Option<AuthenticationData>,

    assigned_client_identifier: Option<AssignedClientIdentifier>,
    reason_string: Option<ReasonString>,
    response_information: Option<ResponseInformation>,
    server_reference: Option<ServerReference>,
    authentication_method: Option<AuthenticationMethod>,

    user_property: Vec<UserProperty>,
}

impl PacketID for Connack {
    const PACKET_ID: u8 = 2;
}

impl TryFromBytes for Connack {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        let mut builder = ConnackPacketBuilder::default();
        let mut reader = ByteReader::from(bytes);

        let fixed_hdr = reader.try_read::<Byte>()?;
        if fixed_hdr >> 4 != Self::PACKET_ID {
            return None; // Invalid header
        }

        let remaining_len = reader.try_read::<VarSizeInt>()?;
        let packet_size =
            mem::size_of_val(&fixed_hdr) + remaining_len.len() + remaining_len.value() as usize;
        if packet_size > bytes.len() {
            return None; // Invalid packet size
        }

        let session_present = reader.try_read::<Boolean>()?;
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
                    return None;
                }
            }
        }

        builder.build()
    }
}

#[derive(Default)]
pub(crate) struct ConnackPacketBuilder {
    // Connack variable header
    session_present: Option<bool>,
    reason: Option<ConnectReason>,

    // Connack properties
    wildcard_subscription_available: Option<WildcardSubscriptionAvailable>,
    subscription_identifier_available: Option<SubscriptionIdentifierAvailable>,
    shared_subscription_available: Option<SharedSubscriptionAvailable>,
    maximum_qos: Option<MaximumQoS>,
    retain_available: Option<RetainAvailable>,

    server_keep_alive: Option<ServerKeepAlive>,
    receive_maximum: Option<ReceiveMaximum>,
    topic_alias_maximum: Option<TopicAliasMaximum>,

    session_expiry_interval: Option<SessionExpiryInterval>,
    maximum_packet_size: Option<MaximumPacketSize>,

    authentication_data: Option<AuthenticationData>,

    assigned_client_identifier: Option<AssignedClientIdentifier>,
    reason_string: Option<ReasonString>,
    response_information: Option<ResponseInformation>,
    server_reference: Option<ServerReference>,
    authentication_method: Option<AuthenticationMethod>,

    user_property: Vec<UserProperty>,
}

impl ConnackPacketBuilder {
    pub(crate) fn session_present(&mut self, val: bool) -> &mut Self {
        self.session_present = Some(val);
        self
    }

    pub(crate) fn reason(&mut self, val: ConnectReason) -> &mut Self {
        self.reason = Some(val);
        self
    }

    pub(crate) fn wildcard_subscription_available(
        &mut self,
        val: WildcardSubscriptionAvailable,
    ) -> &mut Self {
        self.wildcard_subscription_available = Some(val);
        self
    }

    pub(crate) fn subscription_identifier_available(
        &mut self,
        val: SubscriptionIdentifierAvailable,
    ) -> &mut Self {
        self.subscription_identifier_available = Some(val);
        self
    }

    pub(crate) fn shared_subscription_available(
        &mut self,
        val: SharedSubscriptionAvailable,
    ) -> &mut Self {
        self.shared_subscription_available = Some(val);
        self
    }

    pub(crate) fn maximum_qos(&mut self, val: MaximumQoS) -> &mut Self {
        self.maximum_qos = Some(val);
        self
    }

    pub(crate) fn retain_available(&mut self, val: RetainAvailable) -> &mut Self {
        self.retain_available = Some(val);
        self
    }

    pub(crate) fn server_keep_alive(&mut self, val: ServerKeepAlive) -> &mut Self {
        self.server_keep_alive = Some(val);
        self
    }

    pub(crate) fn receive_maximum(&mut self, val: ReceiveMaximum) -> &mut Self {
        self.receive_maximum = Some(val);
        self
    }

    pub(crate) fn topic_alias_maximum(&mut self, val: TopicAliasMaximum) -> &mut Self {
        self.topic_alias_maximum = Some(val);
        self
    }

    pub(crate) fn session_expiry_interval(&mut self, val: SessionExpiryInterval) -> &mut Self {
        self.session_expiry_interval = Some(val);
        self
    }

    pub(crate) fn maximum_packet_size(&mut self, val: MaximumPacketSize) -> &mut Self {
        self.maximum_packet_size = Some(val);
        self
    }

    pub(crate) fn authentication_data(&mut self, val: AuthenticationData) -> &mut Self {
        self.authentication_data = Some(val);
        self
    }

    pub(crate) fn assigned_client_identifier(
        &mut self,
        val: AssignedClientIdentifier,
    ) -> &mut Self {
        self.assigned_client_identifier = Some(val);
        self
    }

    pub(crate) fn reason_string(&mut self, val: ReasonString) -> &mut Self {
        self.reason_string = Some(val);
        self
    }

    pub(crate) fn response_information(&mut self, val: ResponseInformation) -> &mut Self {
        self.response_information = Some(val);
        self
    }
    pub(crate) fn server_reference(&mut self, val: ServerReference) -> &mut Self {
        self.server_reference = Some(val);
        self
    }

    pub(crate) fn authentication_method(&mut self, val: AuthenticationMethod) -> &mut Self {
        self.authentication_method = Some(val);
        self
    }

    pub(crate) fn user_property(&mut self, val: UserProperty) -> &mut Self {
        self.user_property.push(val);
        self
    }

    pub(crate) fn build(self) -> Option<Connack> {
        Some(Connack {
            session_present: self.session_present?,
            reason: self.reason?,
            wildcard_subscription_available: Some(
                self.wildcard_subscription_available.unwrap_or_default(),
            ),
            subscription_identifier_available: Some(
                self.subscription_identifier_available.unwrap_or_default(),
            ),
            shared_subscription_available: Some(
                self.shared_subscription_available.unwrap_or_default(),
            ),
            maximum_qos: Some(self.maximum_qos.unwrap_or_default()),
            retain_available: Some(self.retain_available.unwrap_or_default()),
            server_keep_alive: self.server_keep_alive,
            receive_maximum: Some(self.receive_maximum.unwrap_or_default()),
            topic_alias_maximum: Some(self.topic_alias_maximum.unwrap_or_default()),
            session_expiry_interval: self.session_expiry_interval,
            maximum_packet_size: Some(self.maximum_packet_size.unwrap_or_default()),
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
    fn from_bytes() {
        const PACKET: [u8; 65] = [
            0x20u8, // Fixed header
            63,     // Remaining length
            0x00,   // Connect Acknowledge Flags (No session present)
            0x00,   // Reason (Success)
            60,     // Property length
            0x11,   // Session Expiry Interval
            0x00, 0x00, 0x03, 0x84, // 900 seconds
            0x21, // Receive maximum
            0x4e, 0x20, // 20 000
            0x24, // Maximum QoS
            0x01, // 1
            0x25, // Retain available
            0x01, // Yes
            0x27, // Maximum packet size
            0x00, 0x00, 0x01, 0x00, // 256 bytes
            0x12, // Assigned client identifier
            0x00, // String length MSB
            0x04, // String length LSB
            b't', b'e', b's', b't', 0x22, // Topic alias maximum
            0x00, 0x0a, // 10
            0x1f, // Reason String
            0x00, // String length MSB
            0x07, // String length LSB
            b's', b'u', b'c', b'c', b'e', b's', b's', 0x28, // Wildcard subscription available
            0x01, // Yes
            0x29, // Subscription identifiers available
            0x01, // Yes
            0x2a, // Shared subscription avaialble
            0x01, // Yes
            0x13, // Server keep alive
            0x00, 0x64, // 100 seconds
            0x1a, // Response information
            0x00, // String length MSB
            0x04, // String length LSB
            b't', b'e', b's', b't', 0x1c, // Server reference
            0x00, // String length MSB
            0x04, // String length LSB
            b't', b'e', b's', b't',
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
            Some(SessionExpiryInterval(900))
        );
        assert_eq!(result.receive_maximum, Some(ReceiveMaximum(20000)));
        assert_eq!(result.maximum_qos, Some(MaximumQoS(QoS::AtLeastOnce)));
        assert_eq!(result.retain_available, Some(RetainAvailable(true)));
        assert_eq!(result.maximum_packet_size, Some(MaximumPacketSize(256)));
        assert_eq!(
            result.assigned_client_identifier,
            Some(AssignedClientIdentifier(String::from("test")))
        );
        assert_eq!(result.topic_alias_maximum, Some(TopicAliasMaximum(10)));
        assert_eq!(
            result.reason_string,
            Some(ReasonString(String::from("success")))
        );
        assert_eq!(
            result.wildcard_subscription_available,
            Some(WildcardSubscriptionAvailable(true))
        );
        assert_eq!(
            result.subscription_identifier_available,
            Some(SubscriptionIdentifierAvailable(true))
        );
        assert_eq!(
            result.shared_subscription_available,
            Some(SharedSubscriptionAvailable(true))
        );
        assert_eq!(result.server_keep_alive, Some(ServerKeepAlive(100)));
        assert_eq!(
            result.response_information,
            Some(ResponseInformation(String::from("test")))
        );
        assert_eq!(
            result.server_reference,
            Some(ServerReference(String::from("test")))
        );
    }

    #[test]
    fn from_bytes_short() {
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

        // Check default values

        assert_eq!(result.receive_maximum, Some(ReceiveMaximum(65535)));
        assert_eq!(result.topic_alias_maximum, Some(TopicAliasMaximum(0)));
        assert_eq!(result.maximum_qos, Some(MaximumQoS(QoS::ExactlyOnce)));
        assert_eq!(result.retain_available, Some(RetainAvailable(true)));
        assert_eq!(
            result.maximum_packet_size,
            Some(MaximumPacketSize(u32::MAX))
        );
        assert_eq!(
            result.wildcard_subscription_available,
            Some(WildcardSubscriptionAvailable(true))
        );
        assert_eq!(
            result.subscription_identifier_available,
            Some(SubscriptionIdentifierAvailable(true))
        );
        assert_eq!(
            result.shared_subscription_available,
            Some(SharedSubscriptionAvailable(true))
        );
    }
}
