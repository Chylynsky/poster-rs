use crate::{
    codec::ack::{Ack, AckBuilder},
    core::{
        base_types::*,
        utils::{PacketID, SizedProperty, ToByteBuffer, TryFromBytes},
    },
};
use std::mem;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum PubackReason {
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

impl PubackReason {
    fn try_from(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(PubackReason::Success),
            0x10 => Some(PubackReason::NoMatchingSubscribers),
            0x80 => Some(PubackReason::UnspecifiedError),
            0x83 => Some(PubackReason::ImplementationSpecificError),
            0x87 => Some(PubackReason::NotAuthorized),
            0x90 => Some(PubackReason::TopicNameInvalid),
            0x91 => Some(PubackReason::PacketIdentifierInUse),
            0x97 => Some(PubackReason::QuotaExceeded),
            0x99 => Some(PubackReason::PayloadFormatInvalid),
            _ => None,
        }
    }
}

impl SizedProperty for PubackReason {
    fn property_len(&self) -> usize {
        mem::size_of::<Byte>()
    }
}

impl Default for PubackReason {
    fn default() -> Self {
        Self::Success
    }
}

impl TryFromBytes for PubackReason {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        Self::try_from(Byte::try_from_bytes(bytes)?)
    }
}

impl ToByteBuffer for PubackReason {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        (*self as Byte).to_byte_buffer(buf)
    }
}

pub(crate) type Puback = Ack<PubackReason>;

impl PacketID for Puback {
    const PACKET_ID: u8 = 4;
}

pub(crate) type PubackBuilder = AckBuilder<PubackReason>;

#[cfg(test)]
mod test {
    use super::*;
    use crate::codec::ack::test::*;

    #[test]
    fn from_bytes_0() {
        from_bytes_impl::<PubackReason>();
    }

    #[test]
    fn from_bytes_1() {
        from_bytes_short_impl::<PubackReason>();
    }

    #[test]
    fn to_bytes_0() {
        to_bytes_impl::<PubackReason>();
    }

    #[test]
    fn to_bytes_1() {
        to_bytes_short_impl::<PubackReason>();
    }
}
