use crate::core::{
    base_types::*,
    properties::*,
    utils::{ByteReader, PacketID, TryFromBytes},
};
use std::mem;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum UnsubackReason {
    Success = 0x00,
    NoSubscriptionExisted = 0x11,
    UnspecifiedError = 0x80,
    ImplementationSpecificError = 0x83,
    NotAuthorized = 0x87,
    TopicFilterInvalid = 0x8f,
    PacketIdentifierInUse = 0x91,
}

impl UnsubackReason {
    pub(crate) fn try_from(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(UnsubackReason::Success),
            0x11 => Some(UnsubackReason::NoSubscriptionExisted),
            0x80 => Some(UnsubackReason::UnspecifiedError),
            0x83 => Some(UnsubackReason::ImplementationSpecificError),
            0x87 => Some(UnsubackReason::NotAuthorized),
            0x8f => Some(UnsubackReason::TopicFilterInvalid),
            0x91 => Some(UnsubackReason::PacketIdentifierInUse),
            _ => None,
        }
    }
}

pub(crate) struct Unsuback {
    packet_identifier: TwoByteInteger,

    reason_string: Option<ReasonString>,
    user_property: Vec<UserProperty>,

    payload: Vec<UnsubackReason>,
}

impl Unsuback {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;
}

impl PacketID for Unsuback {
    const PACKET_ID: u8 = 11;
}

impl TryFromBytes for Unsuback {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut builder = UnsubackBuilder::default();
        let mut reader = ByteReader::from(bytes);

        let fixed_hdr = reader.try_read::<Byte>()?;
        if fixed_hdr != Self::FIXED_HDR {
            return None; // Invalid header
        }

        let remaining_len = reader.try_read::<VarSizeInt>()?;
        let packet_size =
            mem::size_of_val(&fixed_hdr) + remaining_len.len() + remaining_len.value() as usize;
        if packet_size > bytes.len() {
            return None; // Invalid packet size
        }

        let packet_id = reader.try_read::<TwoByteInteger>()?;
        builder.packet_identifier(packet_id);

        let property_len = reader.try_read::<VarSizeInt>()?;
        if property_len.value() as usize > reader.remaining() {
            return None; // Invalid property length
        }

        let (properties, payload) = reader.get_buf().split_at(property_len.into());

        for property in PropertyIterator::from(properties) {
            match property {
                Property::ReasonString(val) => {
                    builder.reason_string(val.0);
                }
                Property::UserProperty(val) => {
                    builder.user_property(val.0);
                }
                _ => {
                    return None;
                }
            }
        }

        builder.payload(
            payload
                .iter()
                .map(|&val| UnsubackReason::try_from(val))
                .collect::<Option<Vec<UnsubackReason>>>()?,
        );
        builder.build()
    }
}

#[derive(Default)]
pub(crate) struct UnsubackBuilder {
    packet_identifier: Option<TwoByteInteger>,
    reason_string: Option<ReasonString>,
    user_property: Vec<UserProperty>,
    payload: Vec<UnsubackReason>,
}

impl UnsubackBuilder {
    pub(crate) fn packet_identifier(&mut self, val: TwoByteInteger) -> &mut Self {
        self.packet_identifier = Some(val);
        self
    }

    pub(crate) fn reason_string(&mut self, val: UTF8String) -> &mut Self {
        self.reason_string = Some(ReasonString(val));
        self
    }

    pub(crate) fn user_property(&mut self, val: UTF8StringPair) -> &mut Self {
        self.user_property.push(UserProperty(val));
        self
    }

    pub(crate) fn payload(&mut self, val: Vec<UnsubackReason>) -> &mut Self {
        self.payload = val;
        self
    }

    pub(crate) fn build(self) -> Option<Unsuback> {
        Some(Unsuback {
            packet_identifier: self.packet_identifier?,
            reason_string: self.reason_string,
            user_property: self.user_property,
            payload: self.payload,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::utils::PropertyID;

    #[test]
    fn from_bytes() {
        const FIXED_HDR: u8 = ((Unsuback::PACKET_ID as u8) << 4) as u8;
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
            (UnsubackReason::Success as u8),
        ];

        let packet = Unsuback::try_from_bytes(&PACKET).unwrap();

        assert_eq!(packet.packet_identifier, 0x4573);
        assert_eq!(packet.reason_string.unwrap().0, "test");
        assert_eq!(packet.user_property.len(), 1);
        assert_eq!(
            packet.user_property[0],
            UserProperty((String::from("key"), String::from("val")))
        );
        assert_eq!(packet.payload.len(), 1);
        assert_eq!(packet.payload[0], UnsubackReason::Success)
    }
}