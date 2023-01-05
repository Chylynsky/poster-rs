use crate::core::{
    base_types::*,
    collections::UserProperties,
    error::{
        CodecError, ConversionError, InvalidPacketHeader, InvalidPacketSize, InvalidPropertyLength,
        InvalidValue, UnexpectedProperty,
    },
    properties::*,
    utils::{ByteLen, Decoder, PacketID, TryDecode},
};
use bytes::Bytes;

use derive_builder::Builder;

/// Reason for SUBACK packet.
///
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SubackReason {
    GranteedQoS0 = 0x00,
    GranteedQoS1 = 0x01,
    GranteedQoS2 = 0x02,
    UnspecifiedError = 0x80,
    ImplementationSpecificError = 0x83,
    NotAuthorized = 0x87,
    TopicFilterInvalid = 0x8f,
    PacketIdentifierInUse = 0x91,
    QuotaExceeded = 0x97,
    SharedSubscriptionsNotSupported = 0x9e,
    SubscriptionIdentifiersNotSupported = 0xa1,
    WildcardSubscriptionsNotSupported = 0xa2,
}

impl TryFrom<u8> for SubackReason {
    type Error = ConversionError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x00 => Ok(SubackReason::GranteedQoS0),
            0x01 => Ok(SubackReason::GranteedQoS1),
            0x02 => Ok(SubackReason::GranteedQoS2),
            0x80 => Ok(SubackReason::UnspecifiedError),
            0x83 => Ok(SubackReason::ImplementationSpecificError),
            0x87 => Ok(SubackReason::NotAuthorized),
            0x8f => Ok(SubackReason::TopicFilterInvalid),
            0x91 => Ok(SubackReason::PacketIdentifierInUse),
            0x97 => Ok(SubackReason::QuotaExceeded),
            0x9e => Ok(SubackReason::SharedSubscriptionsNotSupported),
            0xa1 => Ok(SubackReason::SubscriptionIdentifiersNotSupported),
            0xa2 => Ok(SubackReason::WildcardSubscriptionsNotSupported),
            _ => Err(InvalidValue.into()),
        }
    }
}

impl ByteLen for SubackReason {
    fn byte_len(&self) -> usize {
        (*self as u8).byte_len()
    }
}

impl Default for SubackReason {
    fn default() -> Self {
        Self::GranteedQoS0
    }
}

impl TryDecode for SubackReason {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        Self::try_from(u8::try_decode(bytes)?)
    }
}

#[derive(Builder)]
#[builder(build_fn(error = "CodecError"))]
pub(crate) struct SubackRx {
    pub(crate) packet_identifier: NonZero<u16>,

    #[builder(setter(strip_option), default)]
    pub(crate) reason_string: Option<ReasonString>,
    #[builder(setter(custom), default)]
    pub(crate) user_property: UserProperties,

    #[builder(setter(custom), default)]
    pub(crate) payload: Vec<SubackReason>,
}

impl SubackRxBuilder {
    fn user_property(&mut self, value: UserProperty) {
        match self.user_property.as_mut() {
            Some(user_property) => {
                user_property.push(value);
            }
            None => {
                self.user_property = Some(UserProperties::new());
                self.user_property.as_mut().unwrap().push(value);
            }
        }
    }

    fn payload(&mut self, value: SubackReason) {
        match self.payload.as_mut() {
            Some(payload) => {
                payload.push(value);
            }
            None => {
                self.payload = Some(Vec::new());
                self.payload.as_mut().unwrap().push(value);
            }
        }
    }
}

impl SubackRx {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;
}

impl PacketID for SubackRx {
    const PACKET_ID: u8 = 9;
}

impl TryDecode for SubackRx {
    type Error = CodecError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        let mut builder = SubackRxBuilder::default();
        let mut decoder = Decoder::from(bytes);

        let _fixed_hdr = decoder
            .try_decode::<u8>()
            .map_err(CodecError::from)
            .and_then(|val| {
                if val != Self::FIXED_HDR {
                    return Err(InvalidPacketHeader.into());
                }

                Ok(val)
            })?;

        let remaining_len = decoder.try_decode::<VarSizeInt>()?;
        if remaining_len > decoder.remaining() {
            return Err(InvalidPacketSize.into());
        }

        let packet_id = decoder.try_decode::<NonZero<u16>>()?;
        builder.packet_identifier(packet_id);

        let property_len = decoder.try_decode::<VarSizeInt>()?;
        if property_len > decoder.remaining() {
            return Err(InvalidPropertyLength.into());
        }

        let property_iterator =
            Decoder::from(decoder.get_buf().split_to(property_len.value() as usize))
                .iter::<Property>();
        for maybe_property in property_iterator {
            match maybe_property {
                Ok(property) => match property {
                    Property::ReasonString(val) => {
                        builder.reason_string(val);
                    }
                    Property::UserProperty(val) => {
                        builder.user_property(val);
                    }
                    _ => {
                        return Err(UnexpectedProperty.into());
                    }
                },
                Err(err) => return Err(err.into()),
            }
        }

        decoder.advance_by(usize::from(property_len));
        for reason in decoder.iter::<SubackReason>() {
            builder.payload(reason?);
        }

        builder.build()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::utils::PropertyID;

    #[test]
    fn from_bytes_0() {
        const FIXED_HDR: u8 = ((SubackRx::PACKET_ID as u8) << 4) as u8;
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
            (SubackReason::GranteedQoS2 as u8),
        ];

        let packet = SubackRx::try_decode(Bytes::from_static(&PACKET)).unwrap();

        assert_eq!(packet.packet_identifier, NonZero::try_from(0x4573).unwrap());
        assert_eq!(
            packet.reason_string,
            Some(ReasonString::from(UTF8String(Bytes::from_static(
                "test".as_bytes()
            ))))
        );
        assert_eq!(packet.user_property.len(), 1);
        assert_eq!(packet.user_property.get("key").next().unwrap(), "val");
        assert_eq!(packet.payload.len(), 1);
        assert_eq!(packet.payload[0], SubackReason::GranteedQoS2)
    }
}
