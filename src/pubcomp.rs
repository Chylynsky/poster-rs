use crate::{
    base_types::*,
    properties::*,
    utils::{SizedProperty, TryFromBytes, TryFromIterator},
};
use std::mem;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PubcompReason {
    Success = 0x00,
    PacketIdentifierNotFound = 0x92,
}

impl PubcompReason {
    fn try_from(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(PubcompReason::Success),
            0x92 => Some(PubcompReason::PacketIdentifierNotFound),
            _ => None,
        }
    }
}

pub struct Pubcomp {
    packet_identifier: TwoByteInteger,
    reason: PubcompReason,

    reason_string: Option<ReasonString>,
    user_property: Vec<UserProperty>,
}

#[derive(Default)]
pub struct PubcompPacketBuilder {
    packet_identifier: Option<TwoByteInteger>,
    reason: Option<PubcompReason>,
    reason_string: Option<ReasonString>,
    user_property: Vec<UserProperty>,
}

impl PubcompPacketBuilder {
    fn packet_identifier(&mut self, val: TwoByteInteger) -> &mut Self {
        self.packet_identifier = Some(val);
        self
    }

    fn reason(&mut self, val: PubcompReason) -> &mut Self {
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

    fn build(self) -> Option<Pubcomp> {
        Some(Pubcomp {
            packet_identifier: self.packet_identifier?,
            reason: self.reason?,
            reason_string: self.reason_string,
            user_property: self.user_property,
        })
    }
}

impl Pubcomp {
    pub const PACKET_ID: isize = 7;
}

impl TryFromBytes for Pubcomp {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut packet_builder = PubcompPacketBuilder::default();

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
        packet_builder.reason(PubcompReason::try_from(iter.next()?)?);

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
        const FIXED_HDR: u8 = ((Pubcomp::PACKET_ID as u8) << 4) as u8;
        const PACKET: [u8; 27] = [
            FIXED_HDR,
            25,   // Remaining length
            0x45, // Packet ID MSB
            0x73, // Packet ID LSB
            (PubcompReason::Success as u8),
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

        let packet = Pubcomp::try_from_bytes(&PACKET).unwrap();

        assert_eq!(packet.packet_identifier, 0x4573);
        assert_eq!(packet.reason, PubcompReason::Success);
        assert_eq!(packet.reason_string.unwrap().0, "Success");
        assert_eq!(packet.user_property.len(), 1);
        assert_eq!(
            packet.user_property[0],
            UserProperty((String::from("key"), String::from("val")))
        );
    }
}
