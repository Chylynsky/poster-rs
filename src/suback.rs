use crate::{
    base_types::*,
    properties::*,
    utils::{SizedProperty, TryFromBytes, TryFromIterator},
};
use std::mem;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SubackReason {
    GranteedQoS0 = 0x00,
    GranteedQoS1 = 0x01,
    GranteedQoS2 = 0x02,
    UnspecifiedError = 0x80,
    ImplementationSpecificError = 0x83,
    NotAuthorized = 0x87,
    TopicFilterInvalid = 0x8f,
    PacketIdentifierInUse = 0x91,
    QuotaExceeded = 0x97,
    SharedSubscriptionsNotSupported = 0x9e,
    SubscriptionIdentifiersNotSupported = 0xa1,
    WildcardSubscriptionsNotSupported = 0xa2,
}

impl SubackReason {
    fn try_from(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(SubackReason::GranteedQoS0),
            0x01 => Some(SubackReason::GranteedQoS1),
            0x02 => Some(SubackReason::GranteedQoS2),
            0x80 => Some(SubackReason::UnspecifiedError),
            0x83 => Some(SubackReason::ImplementationSpecificError),
            0x87 => Some(SubackReason::NotAuthorized),
            0x8f => Some(SubackReason::TopicFilterInvalid),
            0x91 => Some(SubackReason::PacketIdentifierInUse),
            0x97 => Some(SubackReason::QuotaExceeded),
            0x9e => Some(SubackReason::SharedSubscriptionsNotSupported),
            0xa1 => Some(SubackReason::SubscriptionIdentifiersNotSupported),
            0xa2 => Some(SubackReason::WildcardSubscriptionsNotSupported),
            _ => None,
        }
    }
}

pub struct Suback {
    packet_identifier: TwoByteInteger,

    reason_string: Option<ReasonString>,
    user_property: Vec<UserProperty>,

    payload: Vec<SubackReason>,
}

#[derive(Default)]
pub struct SubackPacketBuilder {
    packet_identifier: Option<TwoByteInteger>,
    reason_string: Option<ReasonString>,
    user_property: Vec<UserProperty>,
    payload: Vec<SubackReason>,
}

impl SubackPacketBuilder {
    fn packet_identifier(&mut self, val: TwoByteInteger) -> &mut Self {
        self.packet_identifier = Some(val);
        self
    }

    fn reason_string(&mut self, val: ReasonString) -> &mut Self {
        self.reason_string = Some(val);
        self
    }

    fn user_property(&mut self, val: UserProperty) -> &mut Self {
        self.user_property.push(val);
        self
    }

    fn payload(&mut self, val: Vec<SubackReason>) -> &mut Self {
        self.payload = val;
        self
    }

    fn build(self) -> Option<Suback> {
        Some(Suback {
            packet_identifier: self.packet_identifier?,
            reason_string: self.reason_string,
            user_property: self.user_property,
            payload: self.payload,
        })
    }
}

impl Suback {
    pub const PACKET_ID: isize = 9;
}

impl TryFromBytes for Suback {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut packet_builder = SubackPacketBuilder::default();

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

        let packet_id = TwoByteInteger::try_from_bytes(var_hdr)?;
        let (_, var_hdr) = var_hdr.split_at(packet_id.property_len());
        packet_builder.packet_identifier(packet_id);

        iter = var_hdr.iter().copied();

        let property_len = VarSizeInt::try_from_iter(iter)?;
        if property_len.len() > var_hdr.len() {
            return None;
        }

        let (_, remaining) = var_hdr.split_at(property_len.len());
        if property_len.value() as usize > remaining.len() {
            return None;
        }

        let (properties, payload) = remaining.split_at(property_len.into());

        for property in PropertyIterator::from(properties) {
            match property {
                Property::ReasonString(val) => {
                    packet_builder.reason_string(val);
                }
                Property::UserProperty(val) => {
                    packet_builder.user_property(val);
                }
                _ => {
                    return None;
                }
            }
        }

        let payload: Option<Vec<SubackReason>> = payload
            .iter()
            .map(|&val| SubackReason::try_from(val))
            .collect();
        packet_builder.payload(payload?);

        packet_builder.build()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes() {
        const FIXED_HDR: u8 = ((Suback::PACKET_ID as u8) << 4) as u8;
        const PACKET: [u8; 24] = [
            FIXED_HDR,
            22,   // Remaining length
            0x45, // Packet ID MSB
            0x73, // Packet ID LSB
            18,   // Property length
            (ReasonString::PROPERTY_ID),
            0, // Reason string size
            4,
            b't',
            b'e',
            b's',
            b't',
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
            (SubackReason::GranteedQoS2 as u8),
        ];

        let packet = Suback::try_from_bytes(&PACKET).unwrap();

        assert_eq!(packet.packet_identifier, 0x4573);
        assert_eq!(packet.reason_string.unwrap().0, "test");
        assert_eq!(packet.user_property.len(), 1);
        assert_eq!(
            packet.user_property[0],
            UserProperty((String::from("key"), String::from("val")))
        );
        assert_eq!(packet.payload.len(), 1);
        assert_eq!(packet.payload[0], SubackReason::GranteedQoS2)
    }
}
