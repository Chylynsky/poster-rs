use crate::core::{
    base_types::*,
    collections::UserProperties,
    error::{
        CodecError, ConversionError, InvalidPacketHeader, InvalidPacketSize, InvalidPropertyLength,
        InvalidValue, MandatoryPropertyMissing, UnexpectedProperty,
    },
    properties::*,
    utils::{ByteLen, Decoder, Encode, Encoder, PacketID, SizedPacket, TryDecode},
};
use bytes::{BufMut, Bytes, BytesMut};
use core::mem;
use derive_builder::Builder;

/// Reason for AUTH packet.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AuthReason {
    /// Success
    ///
    Success = 0x00,

    /// Continue authentication
    ///
    ContinueAuthentication = 0x18,

    /// Re-authenticate
    ///
    ReAuthenticate = 0x19,
}

impl TryFrom<u8> for AuthReason {
    type Error = ConversionError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x00 => Ok(AuthReason::Success),
            0x18 => Ok(AuthReason::ContinueAuthentication),
            0x19 => Ok(AuthReason::ReAuthenticate),
            _ => Err(InvalidValue.into()),
        }
    }
}

impl Default for AuthReason {
    fn default() -> Self {
        Self::Success
    }
}

impl ByteLen for AuthReason {
    fn byte_len(&self) -> usize {
        (*self as u8).byte_len()
    }
}

impl TryDecode for AuthReason {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        Self::try_from(u8::try_decode(bytes)?)
    }
}

impl Encode for AuthReason {
    fn encode(&self, buf: &mut BytesMut) {
        (*self as u8).encode(buf)
    }
}

#[derive(Builder)]
#[builder(build_fn(error = "CodecError", validate = "Self::validate"))]
pub(crate) struct AuthTx<'a> {
    #[builder(default)]
    pub(crate) reason: AuthReason,

    #[builder(setter(strip_option), default)]
    pub(crate) authentication_method: Option<AuthenticationMethodRef<'a>>,
    #[builder(setter(strip_option), default)]
    pub(crate) authentication_data: Option<AuthenticationDataRef<'a>>,
    #[builder(setter(strip_option), default)]
    pub(crate) reason_string: Option<ReasonStringRef<'a>>,
    #[builder(setter(custom), default)]
    pub(crate) user_property: Vec<UserPropertyRef<'a>>,
}

impl<'a> AuthTxBuilder<'a> {
    pub(crate) fn user_property(&mut self, value: UserPropertyRef<'a>) {
        match self.user_property.as_mut() {
            Some(user_property) => {
                user_property.push(value);
            }
            None => {
                self.user_property = Some(Vec::new());
                self.user_property.as_mut().unwrap().push(value);
            }
        }
    }

    fn validate(&self) -> Result<(), CodecError> {
        let shortened = self
            .reason
            .filter(|&reason| reason != AuthReason::Success)
            .is_none()
            && self.authentication_method.is_none()
            && self.authentication_data.is_none()
            && self.reason_string.is_none()
            && self.user_property.is_none();

        if !shortened
            && (self.authentication_method.is_none() || self.authentication_data.is_none())
        {
            Err(MandatoryPropertyMissing.into())
        } else {
            Ok(())
        }
    }
}

impl<'a> AuthTx<'a> {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;

    fn is_shortened(&self) -> bool {
        self.reason == AuthReason::Success
            && self.authentication_method.is_none()
            && self.authentication_data.is_none()
            && self.reason_string.is_none()
            && self.user_property.is_empty()
    }

    fn property_len(&self) -> VarSizeInt {
        VarSizeInt::try_from(
            self.authentication_method
                .as_ref()
                .map(|val| val.byte_len())
                .unwrap_or(0)
                + self
                    .authentication_data
                    .as_ref()
                    .map(|val| val.byte_len())
                    .unwrap_or(0)
                + self
                    .reason_string
                    .as_ref()
                    .map(|val| val.byte_len())
                    .unwrap_or(0)
                + self
                    .user_property
                    .iter()
                    .map(|val| val.byte_len())
                    .sum::<usize>(),
        )
        .unwrap()
    }

    fn remaining_len(&self) -> VarSizeInt {
        if self.is_shortened() {
            return VarSizeInt::from(0u8);
        }

        let property_len = self.property_len();
        VarSizeInt::try_from(
            self.reason.byte_len() + property_len.len() + property_len.value() as usize,
        )
        .unwrap()
    }
}

impl<'a> PacketID for AuthTx<'a> {
    const PACKET_ID: u8 = 15;
}

impl<'a> SizedPacket for AuthTx<'a> {
    fn packet_len(&self) -> usize {
        let remaining_len = self.remaining_len();
        mem::size_of_val(&Self::FIXED_HDR) + remaining_len.len() + remaining_len.value() as usize
    }
}

impl<'a> Encode for AuthTx<'a> {
    fn encode(&self, buf: &mut BytesMut) {
        if self.is_shortened() {
            const AUTH_RAW_DEFAULT: [u8; 2] = [AuthTx::FIXED_HDR, 0];
            buf.put(&AUTH_RAW_DEFAULT[..]);
            return;
        }

        let remaining_len = self.remaining_len();
        let mut encoder = Encoder::from(buf);

        encoder.encode(AuthTx::FIXED_HDR);
        encoder.encode(remaining_len);
        encoder.encode(self.reason);
        encoder.encode(self.authentication_method.unwrap());
        encoder.encode(self.authentication_data.unwrap());

        if self.reason_string.is_some() {
            encoder.encode(self.reason_string.unwrap());
        }

        for val in self.user_property.iter().copied() {
            encoder.encode(val);
        }
    }
}

#[derive(Builder, Default, Clone)]
#[builder(build_fn(error = "CodecError", validate = "Self::validate"))]
pub(crate) struct AuthRx {
    #[builder(default)]
    pub(crate) reason: AuthReason,

    #[builder(setter(strip_option), default)]
    pub(crate) authentication_method: Option<AuthenticationMethod>,
    #[builder(setter(strip_option), default)]
    pub(crate) authentication_data: Option<AuthenticationData>,
    #[builder(setter(strip_option), default)]
    pub(crate) reason_string: Option<ReasonString>,
    #[builder(setter(custom), default)]
    pub(crate) user_property: UserProperties,
}

impl AuthRxBuilder {
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

    fn validate(&self) -> Result<(), CodecError> {
        let shortened = self
            .reason
            .filter(|&reason| reason != AuthReason::Success)
            .is_none()
            && self.authentication_method.is_none()
            && self.authentication_data.is_none()
            && self.reason_string.is_none()
            && self.user_property.is_none();

        if !shortened
            && (self.authentication_method.is_none() || self.authentication_data.is_none())
        {
            Err(MandatoryPropertyMissing.into())
        } else {
            Ok(())
        }
    }
}

impl AuthRx {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;

    fn property_len(&self) -> VarSizeInt {
        VarSizeInt::try_from(
            self.authentication_method
                .as_ref()
                .map(|val| val.byte_len())
                .unwrap_or(0)
                + self
                    .authentication_data
                    .as_ref()
                    .map(|val| val.byte_len())
                    .unwrap_or(0)
                + self
                    .reason_string
                    .as_ref()
                    .map(|val| val.byte_len())
                    .unwrap_or(0)
                + self
                    .user_property
                    .iter()
                    .map(|(key, val)| UTF8StringPairRef(key, val).byte_len())
                    .sum::<usize>(),
        )
        .unwrap()
    }

    fn remaining_len(&self) -> VarSizeInt {
        let byte_len = self.property_len();
        let is_shortened = self.reason == AuthReason::default() && byte_len.value() == 0;
        if is_shortened {
            return VarSizeInt::default();
        }

        VarSizeInt::try_from(self.reason.byte_len() + byte_len.len() + byte_len.value() as usize)
            .unwrap()
    }
}

impl PacketID for AuthRx {
    const PACKET_ID: u8 = 15;
}

impl SizedPacket for AuthRx {
    fn packet_len(&self) -> usize {
        let remaining_len = self.remaining_len();
        mem::size_of_val(&Self::FIXED_HDR) + remaining_len.len() + remaining_len.value() as usize
    }
}

impl TryDecode for AuthRx {
    type Error = CodecError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        let mut builder = AuthRxBuilder::default();
        let mut decoder = Decoder::from(bytes.clone());

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

        // When remaining length is 0, the Reason is 0x00
        if remaining_len.value() == 0 {
            return Ok(AuthRx::default());
        }

        if remaining_len.value() as usize > bytes.len() {
            return Err(InvalidPacketSize.into());
        }

        let reason = decoder.try_decode::<AuthReason>()?;
        builder.reason(reason);

        let property_len = decoder.try_decode::<VarSizeInt>()?;
        if property_len.value() as usize > decoder.remaining() {
            return Err(InvalidPropertyLength.into());
        }

        for maybe_property in decoder.iter::<Property>() {
            match maybe_property {
                Ok(property) => match property {
                    Property::AuthenticationMethod(val) => {
                        builder.authentication_method(val);
                    }
                    Property::AuthenticationData(val) => {
                        builder.authentication_data(val);
                    }
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

        builder.build()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes_1() {
        const FIXED_HDR: u8 = AuthRx::PACKET_ID << 4;
        const PACKET: [u8; 2] = [
            FIXED_HDR, 0, // Remaining length
        ];

        let packet = AuthRx::try_decode(Bytes::from_static(&PACKET));
        assert!(packet.is_ok());
    }

    #[test]
    fn to_bytes_1() {
        const FIXED_HDR: u8 = AuthRx::PACKET_ID << 4;
        const EXPECTED: [u8; 2] = [
            FIXED_HDR, 0, // Remaining length
        ];

        let builder = AuthTxBuilder::default();
        let packet = builder.build().unwrap();

        let mut buf = BytesMut::new();
        packet.encode(&mut buf);

        assert_eq!(&buf.split().freeze()[..], EXPECTED);
    }
}
