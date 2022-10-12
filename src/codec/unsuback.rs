use crate::core::{
    base_types::*,
    error::{
        CodecError, ConversionError, InvalidPacketHeader, InvalidPacketSize, InvalidPropertyLength,
        InvalidValue, MandatoryPropertyMissing, UnexpectedProperty,
    },
    properties::*,
    utils::{ByteReader, PacketID, TryFromBytes},
};
use core::mem;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UnsubackReason {
    Success = 0x00,
    NoSubscriptionExisted = 0x11,
    UnspecifiedError = 0x80,
    ImplementationSpecificError = 0x83,
    NotAuthorized = 0x87,
    TopicFilterInvalid = 0x8f,
    PacketIdentifierInUse = 0x91,
}

impl TryFrom<u8> for UnsubackReason {
    type Error = ConversionError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x00 => Ok(UnsubackReason::Success),
            0x11 => Ok(UnsubackReason::NoSubscriptionExisted),
            0x80 => Ok(UnsubackReason::UnspecifiedError),
            0x83 => Ok(UnsubackReason::ImplementationSpecificError),
            0x87 => Ok(UnsubackReason::NotAuthorized),
            0x8f => Ok(UnsubackReason::TopicFilterInvalid),
            0x91 => Ok(UnsubackReason::PacketIdentifierInUse),
            _ => Err(InvalidValue.into()),
        }
    }
}

pub(crate) struct Unsuback {
    pub(crate) packet_identifier: NonZero<u16>,

    pub(crate) reason_string: Option<ReasonString>,
    pub(crate) user_property: Vec<UserProperty>,

    pub(crate) payload: Vec<UnsubackReason>,
}

impl Unsuback {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;
}

impl PacketID for Unsuback {
    const PACKET_ID: u8 = 11;
}

impl TryFromBytes for Unsuback {
    type Error = CodecError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut builder = UnsubackBuilder::default();
        let mut reader = ByteReader::from(bytes);

        let fixed_hdr = reader
            .try_read::<u8>()
            .map_err(CodecError::from)
            .and_then(|val| {
                if val != Self::FIXED_HDR {
                    return Err(InvalidPacketHeader.into());
                }

                return Ok(val);
            })?;

        let remaining_len = reader.try_read::<VarSizeInt>()?;
        let packet_size =
            mem::size_of_val(&fixed_hdr) + remaining_len.len() + remaining_len.value() as usize;
        if packet_size > bytes.len() {
            return Err(InvalidPacketSize.into());
        }

        let packet_id = reader.try_read::<NonZero<u16>>()?;
        builder.packet_identifier(packet_id);

        let property_len = reader.try_read::<VarSizeInt>()?;
        if property_len.value() as usize > reader.remaining() {
            return Err(InvalidPropertyLength.into());
        }

        let (property_buf, payload) = reader.get_buf().split_at(property_len.into());

        for property in PropertyIterator::from(property_buf) {
            if property.is_err() {
                return Err(property.unwrap_err().into());
            }

            match property.unwrap() {
                Property::ReasonString(val) => {
                    builder.reason_string(val.into());
                }
                Property::UserProperty(val) => {
                    builder.user_property(val.into());
                }
                _ => {
                    return Err(UnexpectedProperty.into());
                }
            }
        }

        builder.payload(
            payload
                .iter()
                .map(|&val| UnsubackReason::try_from(val))
                .collect::<Result<Vec<UnsubackReason>, ConversionError>>()?,
        );
        builder.build()
    }
}

#[derive(Default)]
struct UnsubackBuilder {
    packet_identifier: Option<NonZero<u16>>,
    reason_string: Option<ReasonString>,
    user_property: Vec<UserProperty>,
    payload: Vec<UnsubackReason>,
}

impl UnsubackBuilder {
    fn packet_identifier(&mut self, val: NonZero<u16>) -> &mut Self {
        self.packet_identifier = Some(val);
        self
    }

    fn reason_string(&mut self, val: String) -> &mut Self {
        self.reason_string = Some(val.into());
        self
    }

    fn user_property(&mut self, val: StringPair) -> &mut Self {
        self.user_property.push(val.into());
        self
    }

    fn payload(&mut self, val: Vec<UnsubackReason>) -> &mut Self {
        self.payload = val;
        self
    }

    fn build(self) -> Result<Unsuback, CodecError> {
        Ok(Unsuback {
            packet_identifier: self.packet_identifier.ok_or(MandatoryPropertyMissing)?,
            reason_string: self.reason_string,
            user_property: self.user_property,
            payload: self.payload,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::utils::PropertyID;

    #[test]
    fn from_bytes_0() {
        const FIXED_HDR: u8 = ((Unsuback::PACKET_ID as u8) << 4) as u8;
        const PACKET: [u8; 24] = [
            FIXED_HDR,
            22,   // Remaining length
            0x45, // Packet ID MSB
            0x73, // Packet ID LSB
            18,   // Property length
            (ReasonString::PROPERTY_ID),
            0, // Reason string size
            4,
            b't',
            b'e',
            b's',
            b't',
            (UserProperty::PROPERTY_ID),
            0, // User property key size
            3,
            b'k',
            b'e',
            b'y',
            0, // User property value size
            3,
            b'v',
            b'a',
            b'l',
            (UnsubackReason::Success as u8),
        ];

        let packet = Unsuback::try_from_bytes(&PACKET).unwrap();

        assert_eq!(packet.packet_identifier, NonZero::from(0x4573));
        assert_eq!(
            String::from(packet.reason_string.unwrap()),
            String::from("test")
        );
        assert_eq!(packet.user_property.len(), 1);
        assert_eq!(
            packet.user_property[0],
            UserProperty::from((String::from("key"), String::from("val")))
        );
        assert_eq!(packet.payload.len(), 1);
        assert_eq!(packet.payload[0], UnsubackReason::Success)
    }
}
