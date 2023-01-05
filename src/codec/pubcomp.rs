use crate::{
    codec::ack::{AckRx, AckTx, AckTxBuilder, FixedHeader},
    core::{
        error::{ConversionError, InvalidValue},
        utils::{ByteLen, Encode, PacketID, TryDecode},
    },
};
use bytes::{Bytes, BytesMut};
use core::mem;

/// Reason for PUBCOMP packet.
///
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PubcompReason {
    Success = 0x00,
    PacketIdentifierNotFound = 0x92,
}

impl TryFrom<u8> for PubcompReason {
    type Error = ConversionError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x00 => Ok(PubcompReason::Success),
            0x92 => Ok(PubcompReason::PacketIdentifierNotFound),
            _ => Err(InvalidValue.into()),
        }
    }
}

impl Default for PubcompReason {
    fn default() -> Self {
        Self::Success
    }
}

impl ByteLen for PubcompReason {
    fn byte_len(&self) -> usize {
        mem::size_of::<u8>()
    }
}

impl TryDecode for PubcompReason {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        Self::try_from(u8::try_decode(bytes)?)
    }
}

impl Encode for PubcompReason {
    fn encode(&self, buf: &mut BytesMut) {
        (*self as u8).encode(buf)
    }
}

pub(crate) type PubcompRx = AckRx<PubcompReason>;

impl PacketID for PubcompRx {
    const PACKET_ID: u8 = 7;
}

impl FixedHeader for PubcompRx {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;
}

pub(crate) type PubcompTx<'a> = AckTx<'a, PubcompReason>;

impl<'a> PacketID for PubcompTx<'a> {
    const PACKET_ID: u8 = 7;
}

impl<'a> FixedHeader for PubcompTx<'a> {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;
}

pub(crate) type PubcompTxBuilder<'a> = AckTxBuilder<'a, PubcompReason>;

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
