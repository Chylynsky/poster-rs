use crate::{
    ack::{Ack, AckPacketBuilder},
    base_types::*,
    properties::*,
    utils::{
        ByteWriter, PacketID, PropertyID, SizedPacket, SizedProperty, ToByteBuffer, TryFromBytes,
        TryFromIterator, TryToByteBuffer,
    },
};
use std::mem;

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
        (*self as Byte).property_len()
    }
}

impl TryFromBytes for PubrelReason {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        Self::try_from(Byte::try_from_bytes(bytes)?)
    }
}

impl ToByteBuffer for PubrelReason {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        (*self as Byte).to_byte_buffer(buf)
    }
}

pub(crate) type Pubrel = Ack<PubrelReason>;

impl PacketID for Pubrel {
    const PACKET_ID: u8 = 6;
}

pub(crate) type PubrelPacketBuilder = AckPacketBuilder<PubrelReason>;

#[cfg(test)]
mod test {
    use super::*;
    use crate::ack::test::*;

    #[test]
    fn from_bytes() {
        from_bytes_impl::<PubrelReason>();
    }

    #[test]
    fn from_bytes_short() {
        from_bytes_short_impl::<PubrelReason>();
    }

    #[test]
    fn to_bytes() {
        to_bytes_impl::<PubrelReason>();
    }

    #[test]
    fn to_bytes_short() {
        to_bytes_short_impl::<PubrelReason>();
    }
}
