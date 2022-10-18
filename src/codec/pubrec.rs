use crate::{
    codec::ack::{Ack, AckBuilder},
    core::{
        error::{ConversionError, InvalidValue},
        utils::{PacketID, SizedProperty, ToByteBuffer, TryFromBytes},
    },
};

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

impl SizedProperty for PubrecReason {
    fn property_len(&self) -> usize {
        (*self as u8).property_len()
    }
}

impl TryFromBytes for PubrecReason {
    type Error = ConversionError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from(u8::try_from_bytes(bytes)?)
    }
}

impl ToByteBuffer for PubrecReason {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        (*self as u8).to_byte_buffer(buf)
    }
}

pub(crate) type Pubrec = Ack<PubrecReason>;

impl PacketID for Pubrec {
    const PACKET_ID: u8 = 5;
}

pub(crate) type PubrecBuilder = AckBuilder<PubrecReason>;

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
