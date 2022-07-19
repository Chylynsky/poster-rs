use crate::{
    base_types::*,
    properties::*,
    utils::{TryFromBytes, TryFromIterator},
};
use std::mem;

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
    fn try_from(val: isize) -> Option<Self> {
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

pub struct Connack {
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

#[derive(Default)]
pub struct ConnackPacketBuilder {
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
    fn session_present(&mut self, val: bool) -> &mut Self {
        self.session_present = Some(val);
        self
    }

    fn reason(&mut self, val: ConnectReason) -> &mut Self {
        self.reason = Some(val);
        self
    }

    fn wildcard_subscription_available(&mut self, val: WildcardSubscriptionAvailable) -> &mut Self {
        self.wildcard_subscription_available = Some(val);
        self
    }

    fn subscription_identifier_available(
        &mut self,
        val: SubscriptionIdentifierAvailable,
    ) -> &mut Self {
        self.subscription_identifier_available = Some(val);
        self
    }

    fn shared_subscription_available(&mut self, val: SharedSubscriptionAvailable) -> &mut Self {
        self.shared_subscription_available = Some(val);
        self
    }

    fn maximum_qos(&mut self, val: MaximumQoS) -> &mut Self {
        self.maximum_qos = Some(val);
        self
    }

    fn retain_available(&mut self, val: RetainAvailable) -> &mut Self {
        self.retain_available = Some(val);
        self
    }

    fn server_keep_alive(&mut self, val: ServerKeepAlive) -> &mut Self {
        self.server_keep_alive = Some(val);
        self
    }

    fn receive_maximum(&mut self, val: ReceiveMaximum) -> &mut Self {
        self.receive_maximum = Some(val);
        self
    }

    fn topic_alias_maximum(&mut self, val: TopicAliasMaximum) -> &mut Self {
        self.topic_alias_maximum = Some(val);
        self
    }

    fn session_expiry_interval(&mut self, val: SessionExpiryInterval) -> &mut Self {
        self.session_expiry_interval = Some(val);
        self
    }

    fn maximum_packet_size(&mut self, val: MaximumPacketSize) -> &mut Self {
        self.maximum_packet_size = Some(val);
        self
    }

    fn authentication_data(&mut self, val: AuthenticationData) -> &mut Self {
        self.authentication_data = Some(val);
        self
    }

    fn assigned_client_identifier(&mut self, val: AssignedClientIdentifier) -> &mut Self {
        self.assigned_client_identifier = Some(val);
        self
    }

    fn reason_string(&mut self, val: ReasonString) -> &mut Self {
        self.reason_string = Some(val);
        self
    }

    fn response_information(&mut self, val: ResponseInformation) -> &mut Self {
        self.response_information = Some(val);
        self
    }
    fn server_reference(&mut self, val: ServerReference) -> &mut Self {
        self.server_reference = Some(val);
        self
    }

    fn authentication_method(&mut self, val: AuthenticationMethod) -> &mut Self {
        self.authentication_method = Some(val);
        self
    }

    fn user_property(&mut self, val: UserProperty) -> &mut Self {
        self.user_property.push(val);
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

impl Connack {
    pub const PACKET_ID: isize = 2;
}

impl TryFromBytes for Connack {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        let make_session_present = |val: u8| -> Option<bool> {
            match val {
                0x00 => Some(false),
                0x01 => Some(true),
                _ => None, // Rest of the bits must be set to 0
            }
        };

        let mut packet_builder = ConnackPacketBuilder::default();

        let mut iter = bytes.iter().copied();

        let fixed_hdr = iter.next()?;
        debug_assert!(fixed_hdr >> 4 == Self::PACKET_ID as u8);

        let var_hdr_len = VarSizeInt::try_from_iter(iter)?;
        if (mem::size_of_val(&fixed_hdr) + var_hdr_len.len()) > bytes.len() {
            return None;
        }

        let (_, remaining) = bytes.split_at(mem::size_of_val(&fixed_hdr) + var_hdr_len.len());
        if var_hdr_len.value() as usize > remaining.len() {
            return None;
        }

        let (var_hdr, _) = remaining.split_at(var_hdr_len.into());

        iter = var_hdr.iter().copied();
        packet_builder.session_present(make_session_present(iter.next()?)?);
        packet_builder.reason(ConnectReason::try_from(iter.next()? as isize)?);

        let property_len = VarSizeInt::try_from_iter(iter)?;
        if 2 + property_len.len() > var_hdr.len() {
            return None;
        }

        let (_, remaining) = var_hdr.split_at(2 + property_len.len());
        if property_len.value() as usize > remaining.len() {
            return None;
        }

        let (properties, _) = remaining.split_at(property_len.into());

        for property in PropertyIterator::from(properties) {
            match property {
                Property::WildcardSubscriptionAvailable(val) => {
                    packet_builder.wildcard_subscription_available(val);
                }
                Property::SubscriptionIdentifierAvailable(val) => {
                    packet_builder.subscription_identifier_available(val);
                }
                Property::SharedSubscriptionAvailable(val) => {
                    packet_builder.shared_subscription_available(val);
                }
                Property::MaximumQoS(val) => {
                    packet_builder.maximum_qos(val);
                }
                Property::RetainAvailable(val) => {
                    packet_builder.retain_available(val);
                }
                Property::ServerKeepAlive(val) => {
                    packet_builder.server_keep_alive(val);
                }
                Property::ReceiveMaximum(val) => {
                    packet_builder.receive_maximum(val);
                }
                Property::TopicAliasMaximum(val) => {
                    packet_builder.topic_alias_maximum(val);
                }
                Property::SessionExpiryInterval(val) => {
                    packet_builder.session_expiry_interval(val);
                }
                Property::MaximumPacketSize(val) => {
                    packet_builder.maximum_packet_size(val);
                }
                Property::AuthenticationData(val) => {
                    packet_builder.authentication_data(val);
                }
                Property::AssignedClientIdentifier(val) => {
                    packet_builder.assigned_client_identifier(val);
                }
                Property::ReasonString(val) => {
                    packet_builder.reason_string(val);
                }
                Property::ResponseInformation(val) => {
                    packet_builder.response_information(val);
                }
                Property::ServerReference(val) => {
                    packet_builder.server_reference(val);
                }
                Property::AuthenticationMethod(val) => {
                    packet_builder.authentication_method(val);
                }
                Property::UserProperty(val) => {
                    packet_builder.user_property(val);
                }
                _ => {
                    return None;
                }
            }
        }

        packet_builder.build()
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
}
