use crate::{
    base_types::*,
    properties::*,
    utils::{TryFromBytes, TryFromIterator},
};
use std::mem;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DisconnectReason {
    Success = 0x00,
    DisconnectWithWillMessage = 0x04,
    UnspecifiedError = 0x80,
    MalformedPacket = 0x81,
    ProtocolError = 0x82,
    ImplementationSpecificError = 0x83,
    NotAuthorized = 0x87,
    ServerBusy = 0x89,
    ServerShuttingDown = 0x8b,
    KeepAliveTimeout = 0x8d,
    SessionTakenOver = 0x8e,
    TopicFilterInvalid = 0x8f,
    TopicNameInvalid = 0x90,
    ReceiveMaximumExcceeded = 0x93,
    TopicAliasInvalid = 0x94,
    PacketTooLarge = 0x95,
    MessageRateTooHigh = 0x96,
    QuotaExceeded = 0x97,
    AdministrativeAction = 0x98,
    PayloadFormatInvalid = 0x99,
    RetainNotSupported = 0x9a,
    QoSNotSupported = 0x9b,
    UseAnotherServer = 0x9c,
    ServerMoved = 0x9d,
    SharedSubscriptionsNotSupported = 0x9e,
    ConnectionRateExceeded = 0x9f,
    MaximumConnectTime = 0xa0,
    SubscriptionIdentifiersNotSupported = 0xa1,
    WildcardSubscriptionsNotSupported = 0xa2,
}

impl DisconnectReason {
    fn try_from(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(DisconnectReason::Success),
            0x04 => Some(DisconnectReason::DisconnectWithWillMessage),
            0x80 => Some(DisconnectReason::UnspecifiedError),
            0x81 => Some(DisconnectReason::MalformedPacket),
            0x82 => Some(DisconnectReason::ProtocolError),
            0x83 => Some(DisconnectReason::ImplementationSpecificError),
            0x87 => Some(DisconnectReason::NotAuthorized),
            0x89 => Some(DisconnectReason::ServerBusy),
            0x8b => Some(DisconnectReason::ServerShuttingDown),
            0x8d => Some(DisconnectReason::KeepAliveTimeout),
            0x8e => Some(DisconnectReason::SessionTakenOver),
            0x8f => Some(DisconnectReason::TopicFilterInvalid),
            0x90 => Some(DisconnectReason::TopicNameInvalid),
            0x93 => Some(DisconnectReason::ReceiveMaximumExcceeded),
            0x94 => Some(DisconnectReason::TopicAliasInvalid),
            0x95 => Some(DisconnectReason::PacketTooLarge),
            0x96 => Some(DisconnectReason::MessageRateTooHigh),
            0x97 => Some(DisconnectReason::QuotaExceeded),
            0x98 => Some(DisconnectReason::AdministrativeAction),
            0x99 => Some(DisconnectReason::PayloadFormatInvalid),
            0x9a => Some(DisconnectReason::RetainNotSupported),
            0x9b => Some(DisconnectReason::QoSNotSupported),
            0x9c => Some(DisconnectReason::UseAnotherServer),
            0x9d => Some(DisconnectReason::ServerMoved),
            0x9e => Some(DisconnectReason::SharedSubscriptionsNotSupported),
            0x9f => Some(DisconnectReason::ConnectionRateExceeded),
            0xa0 => Some(DisconnectReason::MaximumConnectTime),
            0xa1 => Some(DisconnectReason::SubscriptionIdentifiersNotSupported),
            0xa2 => Some(DisconnectReason::WildcardSubscriptionsNotSupported),
            _ => None,
        }
    }
}

pub struct Disconnect {
    reason: DisconnectReason,

    session_expiry_interval: Option<SessionExpiryInterval>,
    reason_string: Option<ReasonString>,
    server_reference: Option<ServerReference>,
    user_property: Vec<UserProperty>,
}

#[derive(Default)]
pub struct DisconnectPacketBuilder {
    reason: Option<DisconnectReason>,
    session_expiry_interval: Option<SessionExpiryInterval>,
    reason_string: Option<ReasonString>,
    server_reference: Option<ServerReference>,
    user_property: Vec<UserProperty>,
}

impl DisconnectPacketBuilder {
    fn reason(&mut self, val: DisconnectReason) -> &mut Self {
        self.reason = Some(val);
        self
    }

    fn session_expiry_interval(&mut self, val: SessionExpiryInterval) -> &mut Self {
        self.session_expiry_interval = Some(val);
        self
    }

    fn reason_string(&mut self, val: ReasonString) -> &mut Self {
        self.reason_string = Some(val);
        self
    }

    fn server_reference(&mut self, val: ServerReference) -> &mut Self {
        self.server_reference = Some(val);
        self
    }

    fn user_property(&mut self, val: UserProperty) -> &mut Self {
        self.user_property.push(val);
        self
    }

    fn build(self) -> Option<Disconnect> {
        Some(Disconnect {
            reason: self.reason?,
            session_expiry_interval: self.session_expiry_interval,
            reason_string: self.reason_string,
            user_property: self.user_property,
            server_reference: self.server_reference,
        })
    }
}

impl Disconnect {
    pub const PACKET_ID: isize = 14;
}

impl TryFromBytes for Disconnect {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut packet_builder = DisconnectPacketBuilder::default();

        let mut iter = bytes.iter().copied();
        let fixed_hdr = iter.next()?;

        debug_assert!(fixed_hdr >> 4 == Self::PACKET_ID as u8);
        let remaining_len = VarSizeInt::try_from_iter(iter)?;
        if mem::size_of_val(&fixed_hdr) + remaining_len.len() > bytes.len() {
            return None;
        }

        let (_, var_hdr) = bytes.split_at(mem::size_of_val(&fixed_hdr) + remaining_len.len());
        if remaining_len.value() as usize > var_hdr.len() {
            return None;
        }

        let (var_hdr, _) = var_hdr.split_at(remaining_len.into());

        iter = var_hdr.iter().copied();
        packet_builder.reason(DisconnectReason::try_from(iter.next()?)?);

        let property_len = VarSizeInt::try_from_iter(iter)?;
        if 1 + property_len.len() > var_hdr.len() {
            return None;
        }

        let (_, remaining) = var_hdr.split_at(1 + property_len.len());
        if property_len.value() as usize > remaining.len() {
            return None;
        }

        let (properties, _) = remaining.split_at(property_len.into());

        for property in PropertyIterator::from(properties) {
            match property {
                Property::SessionExpiryInterval(val) => {
                    packet_builder.session_expiry_interval(val);
                }
                Property::ReasonString(val) => {
                    packet_builder.reason_string(val);
                }
                Property::ServerReference(val) => {
                    packet_builder.server_reference(val);
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
        const FIXED_HDR: u8 = ((Disconnect::PACKET_ID as u8) << 4) as u8;
        const PACKET: [u8; 25] = [
            FIXED_HDR,
            23, // Remaining length
            (DisconnectReason::Success as u8),
            21, // Property length
            (ReasonString::PROPERTY_ID),
            0, // Reason string size
            7,
            b'S',
            b'u',
            b'c',
            b'c',
            b'e',
            b's',
            b's',
            (UserProperty::PROPERTY_ID),
            0, // User property key size
            3,
            b'k',
            b'e',
            b'y',
            0, // User property value size
            3,
            b'v',
            b'a',
            b'l',
        ];

        let packet = Disconnect::try_from_bytes(&PACKET).unwrap();

        assert_eq!(packet.reason, DisconnectReason::Success);
        assert_eq!(packet.reason_string.unwrap().0, "Success");
        assert_eq!(packet.user_property.len(), 1);
        assert_eq!(
            packet.user_property[0],
            UserProperty((String::from("key"), String::from("val")))
        );
    }
}
