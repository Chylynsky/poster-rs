use crate::{
    codec::ack::{Ack, AckPacketBuilder},
    core::{
        base_types::*,
        utils::{
            PacketID, SizedProperty, ToByteBuffer,
            TryFromBytes,
        },
    },
};


#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum PubrecReason {
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

impl Default for PubrecReason {
    fn default() -> Self {
        Self::Success
    }
}

impl SizedProperty for PubrecReason {
    fn property_len(&self) -> usize {
        (*self as Byte).property_len()
    }
}

impl TryFromBytes for PubrecReason {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        Self::try_from(Byte::try_from_bytes(bytes)?)
    }
}

impl ToByteBuffer for PubrecReason {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        (*self as Byte).to_byte_buffer(buf)
    }
}

pub(crate) type Pubrec = Ack<PubrecReason>;

impl PacketID for Pubrec {
    const PACKET_ID: u8 = 5;
}

pub(crate) type PubrecPacketBuilder = AckPacketBuilder<PubrecReason>;

#[cfg(test)]
mod test {
    use super::*;
    use crate::codec::ack::test::*;

    #[test]
    fn from_bytes() {
        from_bytes_impl::<PubrecReason>();
    }

    #[test]
    fn from_bytes_short() {
        from_bytes_short_impl::<PubrecReason>();
    }

    #[test]
    fn to_bytes() {
        to_bytes_impl::<PubrecReason>();
    }

    #[test]
    fn to_bytes_short() {
        to_bytes_short_impl::<PubrecReason>();
    }
}
