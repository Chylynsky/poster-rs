use bytes::{Bytes, BytesMut};

use crate::{
    codec::ack::{AckRx, AckRxBuilder, AckTx, AckTxBuilder, FixedHeader},
    core::{
        error::{ConversionError, InvalidValue},
        utils::{ByteLen, Encode, PacketID, TryDecode},
    },
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PubrelReason {
    Success = 0x00,
    PacketIdentifierNotFound = 0x92,
}

impl TryFrom<u8> for PubrelReason {
    type Error = ConversionError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x00 => Ok(PubrelReason::Success),
            0x92 => Ok(PubrelReason::PacketIdentifierNotFound),
            _ => Err(InvalidValue.into()),
        }
    }
}

impl Default for PubrelReason {
    fn default() -> Self {
        Self::Success
    }
}

impl ByteLen for PubrelReason {
    fn byte_len(&self) -> usize {
        (*self as u8).byte_len()
    }
}

impl TryDecode for PubrelReason {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        Self::try_from(u8::try_decode(bytes)?)
    }
}

impl Encode for PubrelReason {
    fn encode(&self, buf: &mut BytesMut) {
        (*self as u8).encode(buf)
    }
}

pub(crate) type PubrelRx = AckRx<PubrelReason>;

impl PacketID for PubrelRx {
    const PACKET_ID: u8 = 6;
}

impl FixedHeader for PubrelRx {
    const FIXED_HDR: u8 = (Self::PACKET_ID << 4) | 0b0010;
}

pub(crate) type PubrelTx<'a> = AckTx<'a, PubrelReason>;

impl<'a> PacketID for PubrelTx<'a> {
    const PACKET_ID: u8 = 6;
}

impl<'a> FixedHeader for PubrelTx<'a> {
    const FIXED_HDR: u8 = (Self::PACKET_ID << 4) | 0b0010;
}

pub(crate) type PubrelRxBuilder = AckRxBuilder<PubrelReason>;

pub(crate) type PubrelTxBuilder<'a> = AckTxBuilder<'a, PubrelReason>;

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
