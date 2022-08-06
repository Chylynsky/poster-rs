use crate::{
    codec::ack::{Ack, AckBuilder},
    core::utils::{PacketID, SizedProperty, ToByteBuffer, TryFromBytes},
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum PubrelReason {
    Success = 0x00,
    PacketIdentifierNotFound = 0x92,
}

impl PubrelReason {
    fn try_from(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(PubrelReason::Success),
            0x92 => Some(PubrelReason::PacketIdentifierNotFound),
            _ => None,
        }
    }
}

impl Default for PubrelReason {
    fn default() -> Self {
        Self::Success
    }
}

impl SizedProperty for PubrelReason {
    fn property_len(&self) -> usize {
        (*self as u8).property_len()
    }
}

impl TryFromBytes for PubrelReason {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        Self::try_from(u8::try_from_bytes(bytes)?)
    }
}

impl ToByteBuffer for PubrelReason {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        (*self as u8).to_byte_buffer(buf)
    }
}

pub(crate) type Pubrel = Ack<PubrelReason>;

impl PacketID for Pubrel {
    const PACKET_ID: u8 = 6;
}

pub(crate) type PubrelBuilder = AckBuilder<PubrelReason>;

#[cfg(test)]
mod test {
    use super::*;
    use crate::codec::ack::test::*;

    #[test]
    fn from_bytes_0() {
        from_bytes_impl::<PubrelReason>();
    }

    #[test]
    fn from_bytes_1() {
        from_bytes_short_impl::<PubrelReason>();
    }

    #[test]
    fn to_bytes_0() {
        to_bytes_impl::<PubrelReason>();
    }

    #[test]
    fn to_bytes_1() {
        to_bytes_short_impl::<PubrelReason>();
    }
}
