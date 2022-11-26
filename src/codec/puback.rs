use bytes::{Bytes, BytesMut};

use crate::{
    codec::ack::{AckRx, AckRxBuilder, AckTx, AckTxBuilder},
    core::{
        error::{ConversionError, InvalidValue},
        utils::{ByteLen, Encode, PacketID, TryDecode},
    },
};
use core::mem;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PubackReason {
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

impl TryFrom<u8> for PubackReason {
    type Error = ConversionError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x00 => Ok(PubackReason::Success),
            0x10 => Ok(PubackReason::NoMatchingSubscribers),
            0x80 => Ok(PubackReason::UnspecifiedError),
            0x83 => Ok(PubackReason::ImplementationSpecificError),
            0x87 => Ok(PubackReason::NotAuthorized),
            0x90 => Ok(PubackReason::TopicNameInvalid),
            0x91 => Ok(PubackReason::PacketIdentifierInUse),
            0x97 => Ok(PubackReason::QuotaExceeded),
            0x99 => Ok(PubackReason::PayloadFormatInvalid),
            _ => Err(InvalidValue.into()),
        }
    }
}

impl ByteLen for PubackReason {
    fn byte_len(&self) -> usize {
        mem::size_of::<u8>()
    }
}

impl Default for PubackReason {
    fn default() -> Self {
        Self::Success
    }
}

impl TryDecode for PubackReason {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        Self::try_from(u8::try_decode(bytes)?)
    }
}

impl Encode for PubackReason {
    fn encode(&self, buf: &mut BytesMut) {
        (*self as u8).encode(buf)
    }
}

pub(crate) type PubackRx = AckRx<PubackReason>;

impl PacketID for PubackRx {
    const PACKET_ID: u8 = 4;
}

pub(crate) type PubackRxBuilder = AckRxBuilder<PubackReason>;

pub(crate) type PubackTx<'a> = AckTx<'a, PubackReason>;

impl<'a> PacketID for PubackTx<'a> {
    const PACKET_ID: u8 = 4;
}

pub(crate) type PubackTxBuilder<'a> = AckTxBuilder<'a, PubackReason>;

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
