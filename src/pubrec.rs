use crate::{
    base_types::*,
    properties::*,
    utils::{SizedProperty, TryFromBytes, TryFromIterator},
};
use std::mem;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PubrecReason {
    Success = 0x00,
    NoMatchingSubscribers = 0x10,
    UnspecifiedError = 0x80,
    ImplementationSpecificError = 0x83,
    NotAuthorized = 0x87,
    TopicNameInvalid = 0x90,
    PacketIdentifierInUse = 0x91,
    QuotaExceeded = 0x97,
    PayloadFormatInvalid = 0x99,
}

impl PubrecReason {
    fn try_from(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(PubrecReason::Success),
            0x10 => Some(PubrecReason::NoMatchingSubscribers),
            0x80 => Some(PubrecReason::UnspecifiedError),
            0x83 => Some(PubrecReason::ImplementationSpecificError),
            0x87 => Some(PubrecReason::NotAuthorized),
            0x90 => Some(PubrecReason::TopicNameInvalid),
            0x91 => Some(PubrecReason::PacketIdentifierInUse),
            0x97 => Some(PubrecReason::QuotaExceeded),
            0x99 => Some(PubrecReason::PayloadFormatInvalid),
            _ => None,
        }
    }
}

pub struct Pubrec {
    packet_identifier: TwoByteInteger,
    reason: PubrecReason,

    reason_string: Option<ReasonString>,
    user_property: Vec<UserProperty>,
}

#[derive(Default)]
pub struct PubrecPacketBuilder {
    packet_identifier: Option<TwoByteInteger>,
    reason: Option<PubrecReason>,
    reason_string: Option<ReasonString>,
    user_property: Vec<UserProperty>,
}

impl PubrecPacketBuilder {
    fn packet_identifier(&mut self, val: TwoByteInteger) -> &mut Self {
        self.packet_identifier = Some(val);
        self
    }

    fn reason(&mut self, val: PubrecReason) -> &mut Self {
        self.reason = Some(val);
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

    fn build(self) -> Option<Pubrec> {
        Some(Pubrec {
            packet_identifier: self.packet_identifier?,
            reason: self.reason?,
            reason_string: self.reason_string,
            user_property: self.user_property,
        })
    }
}

impl Pubrec {
    pub const PACKET_ID: isize = 5;
}

impl TryFromBytes for Pubrec {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut packet_builder = PubrecPacketBuilder::default();

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

        let reason = PubrecReason::try_from(iter.next()?)?;
        let (_, var_hdr) = var_hdr.split_at(1);
        packet_builder.reason(reason);

        let property_len = VarSizeInt::try_from_iter(iter)?;
        if property_len.len() > var_hdr.len() {
            return None;
        }

        let (_, remaining) = var_hdr.split_at(property_len.len());
        if property_len.value() as usize > remaining.len() {
            return None;
        }

        let (properties, _) = remaining.split_at(property_len.into());

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

        packet_builder.build()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes() {
        const FIXED_HDR: u8 = ((Pubrec::PACKET_ID as u8) << 4) as u8;
        const PACKET: [u8; 27] = [
            FIXED_HDR,
            25,   // Remaining length
            0x45, // Packet ID MSB
            0x73, // Packet ID LSB
            (PubrecReason::Success as u8),
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

        let packet = Pubrec::try_from_bytes(&PACKET).unwrap();

        assert_eq!(packet.packet_identifier, 0x4573);
        assert_eq!(packet.reason, PubrecReason::Success);
        assert_eq!(packet.reason_string.unwrap().0, "Success");
        assert_eq!(packet.user_property.len(), 1);
        assert_eq!(
            packet.user_property[0],
            UserProperty((String::from("key"), String::from("val")))
        );
    }
}
