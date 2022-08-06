use crate::{
    codec::ack::{Ack, AckBuilder},
    core::utils::{PacketID, SizedProperty, ToByteBuffer, TryFromBytes},
};
use core::mem;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum PubcompReason {
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

impl Default for PubcompReason {
    fn default() -> Self {
        Self::Success
    }
}

impl SizedProperty for PubcompReason {
    fn property_len(&self) -> usize {
        mem::size_of::<u8>()
    }
}

impl TryFromBytes for PubcompReason {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        Self::try_from(u8::try_from_bytes(bytes)?)
    }
}

impl ToByteBuffer for PubcompReason {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        (*self as u8).to_byte_buffer(buf)
    }
}

pub(crate) type Pubcomp = Ack<PubcompReason>;

impl PacketID for Pubcomp {
    const PACKET_ID: u8 = 7;
}

pub(crate) type PubcompBuilder = AckBuilder<PubcompReason>;

#[cfg(test)]
mod test {
    use super::*;
    use crate::codec::ack::test::*;

    #[test]
    fn from_bytes_0() {
        from_bytes_impl::<PubcompReason>();
    }

    #[test]
    fn from_bytes_1() {
        from_bytes_short_impl::<PubcompReason>();
    }

    #[test]
    fn to_bytes_0() {
        to_bytes_impl::<PubcompReason>();
    }

    #[test]
    fn to_bytes_1() {
        to_bytes_short_impl::<PubcompReason>();
    }
}
