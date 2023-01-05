use bytes::{Bytes, BytesMut};

use crate::{
    codec::ack::{AckRx, AckTx, AckTxBuilder, FixedHeader},
    core::{
        error::{ConversionError, InvalidValue},
        utils::{ByteLen, Encode, PacketID, TryDecode},
    },
};

/// Reason for PUBREC packet.
///
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PubrecReason {
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

impl TryFrom<u8> for PubrecReason {
    type Error = ConversionError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x00 => Ok(PubrecReason::Success),
            0x10 => Ok(PubrecReason::NoMatchingSubscribers),
            0x80 => Ok(PubrecReason::UnspecifiedError),
            0x83 => Ok(PubrecReason::ImplementationSpecificError),
            0x87 => Ok(PubrecReason::NotAuthorized),
            0x90 => Ok(PubrecReason::TopicNameInvalid),
            0x91 => Ok(PubrecReason::PacketIdentifierInUse),
            0x97 => Ok(PubrecReason::QuotaExceeded),
            0x99 => Ok(PubrecReason::PayloadFormatInvalid),
            _ => Err(InvalidValue.into()),
        }
    }
}

impl Default for PubrecReason {
    fn default() -> Self {
        Self::Success
    }
}

impl ByteLen for PubrecReason {
    fn byte_len(&self) -> usize {
        (*self as u8).byte_len()
    }
}

impl TryDecode for PubrecReason {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        Self::try_from(u8::try_decode(bytes)?)
    }
}

impl Encode for PubrecReason {
    fn encode<'a>(&self, buf: &mut BytesMut) {
        (*self as u8).encode(buf)
    }
}

pub(crate) type PubrecRx = AckRx<PubrecReason>;

impl PacketID for PubrecRx {
    const PACKET_ID: u8 = 5;
}

impl FixedHeader for PubrecRx {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;
}

pub(crate) type PubrecTx<'a> = AckTx<'a, PubrecReason>;

impl<'a> PacketID for PubrecTx<'a> {
    const PACKET_ID: u8 = 5;
}

impl<'a> FixedHeader for PubrecTx<'a> {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;
}

pub(crate) type PubrecTxBuilder<'a> = AckTxBuilder<'a, PubrecReason>;

#[cfg(test)]
mod test {
    use super::*;
    use crate::codec::ack::test::*;

    #[test]
    fn from_bytes_0() {
        from_bytes_impl::<PubrecReason>();
    }

    #[test]
    fn from_bytes_1() {
        from_bytes_short_impl::<PubrecReason>();
    }

    #[test]
    fn to_bytes_0() {
        to_bytes_impl::<PubrecReason>();
    }

    #[test]
    fn to_bytes_1() {
        to_bytes_short_impl::<PubrecReason>();
    }
}
