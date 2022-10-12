use crate::{
    codec::ack::{Ack, AckBuilder},
    core::{
        error::{CodecError, ConversionError, InvalidValue},
        utils::{PacketID, SizedProperty, ToByteBuffer, TryFromBytes},
    },
};
use core::mem;

#[derive(Copy, Clone, Debug, PartialEq)]
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

impl SizedProperty for PubackReason {
    fn property_len(&self) -> usize {
        mem::size_of::<u8>()
    }
}

impl Default for PubackReason {
    fn default() -> Self {
        Self::Success
    }
}

impl TryFromBytes for PubackReason {
    type Error = ConversionError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from(u8::try_from_bytes(bytes)?)
    }
}

impl ToByteBuffer for PubackReason {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        (*self as u8).to_byte_buffer(buf)
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
