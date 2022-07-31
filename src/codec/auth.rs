use crate::core::{
    base_types::*,
    properties::*,
    utils::{
        ByteReader, ByteWriter, PacketID, SizedPacket, SizedProperty, ToByteBuffer, TryFromBytes,
        TryToByteBuffer,
    },
};
use std::mem;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum AuthReason {
    Success = 0x00,
    ContinueAuthentication = 0x18,
    ReAuthenticate = 0x19,
}

impl AuthReason {
    fn try_from(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(AuthReason::Success),
            0x18 => Some(AuthReason::ContinueAuthentication),
            0x19 => Some(AuthReason::ReAuthenticate),
            _ => None,
        }
    }
}

impl Default for AuthReason {
    fn default() -> Self {
        Self::Success
    }
}

impl SizedProperty for AuthReason {
    fn property_len(&self) -> usize {
        (*self as Byte).property_len()
    }
}

impl TryFromBytes for AuthReason {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        Self::try_from(Byte::try_from_bytes(bytes)?)
    }
}

impl ToByteBuffer for AuthReason {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        (*self as Byte).to_byte_buffer(buf)
    }
}

#[derive(Default)]
pub(crate) struct Auth {
    reason: AuthReason,

    authentication_method: Option<AuthenticationMethod>,
    authentication_data: Option<AuthenticationData>,
    reason_string: Option<ReasonString>,
    user_property: Vec<UserProperty>,
}

impl Auth {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;

    fn property_len(&self) -> VarSizeInt {
        VarSizeInt::from(
            self.authentication_method
                .as_ref()
                .map(|val| val.property_len())
                .unwrap_or(0)
                + self
                    .authentication_data
                    .as_ref()
                    .map(|val| val.property_len())
                    .unwrap_or(0)
                + self
                    .reason_string
                    .as_ref()
                    .map(|val| val.property_len())
                    .unwrap_or(0)
                + self
                    .user_property
                    .iter()
                    .map(|val| val.property_len())
                    .sum::<usize>(),
        )
    }

    fn remaining_len(&self) -> VarSizeInt {
        let property_len = self.property_len();
        let is_shortened = self.reason == AuthReason::default() && property_len.value() == 0;
        if is_shortened {
            return VarSizeInt::default();
        }

        VarSizeInt::from(
            self.reason.property_len() + property_len.len() + property_len.value() as usize,
        )
    }
}

impl PacketID for Auth {
    const PACKET_ID: u8 = 15;
}

impl SizedPacket for Auth {
    fn packet_len(&self) -> usize {
        let remaining_len = self.remaining_len();
        mem::size_of_val(&Self::FIXED_HDR) + remaining_len.len() + remaining_len.value() as usize
    }
}

impl TryFromBytes for Auth {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut builder = AuthPacketBuilder::default();
        let mut reader = ByteReader::from(bytes);

        let fixed_hdr = reader.try_read::<Byte>()?;
        if fixed_hdr != Self::FIXED_HDR {
            return None;
        }

        let remaining_len = reader.try_read::<VarSizeInt>()?;

        // When remaining length is 0, the Reason is 0x00
        if remaining_len.value() == 0 {
            return Some(Auth::default());
        }

        let packet_size =
            mem::size_of_val(&fixed_hdr) + remaining_len.len() + remaining_len.value() as usize;
        if packet_size > bytes.len() {
            return None; // Invalid packet size
        }

        let reason = reader.try_read::<AuthReason>()?;
        builder.reason(reason);

        let property_len = reader.try_read::<VarSizeInt>()?;
        if property_len.value() as usize > reader.remaining() {
            return None; // Invalid property length
        }

        for property in PropertyIterator::from(reader.get_buf()) {
            match property {
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
                    return None;
                }
            }
        }

        builder.build()
    }
}

impl TryToByteBuffer for Auth {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.packet_len())?;
        let mut writer = ByteWriter::from(result);

        writer.write(&Self::FIXED_HDR);

        let remaining_len = self.remaining_len();
        if remaining_len.value() == 0 {
            return Some(result);
        }

        writer.write(&self.reason);
        writer.write(self.authentication_method.as_ref()?);

        if let Some(val) = self.authentication_data.as_ref() {
            writer.write(val);
        }

        if let Some(val) = self.reason_string.as_ref() {
            writer.write(val);
        }

        for val in self.user_property.iter() {
            writer.write(val);
        }

        Some(result)
    }
}

#[derive(Default)]
pub(crate) struct AuthPacketBuilder {
    reason: Option<AuthReason>,
    authentication_method: Option<AuthenticationMethod>,
    authentication_data: Option<AuthenticationData>,
    reason_string: Option<ReasonString>,
    user_property: Vec<UserProperty>,
}

impl AuthPacketBuilder {
    pub(crate) fn reason(&mut self, val: AuthReason) -> &mut Self {
        self.reason = Some(val);
        self
    }

    pub(crate) fn authentication_data(&mut self, val: AuthenticationData) -> &mut Self {
        self.authentication_data = Some(val);
        self
    }

    pub(crate) fn authentication_method(&mut self, val: AuthenticationMethod) -> &mut Self {
        self.authentication_method = Some(val);
        self
    }

    pub(crate) fn reason_string(&mut self, val: ReasonString) -> &mut Self {
        self.reason_string = Some(val);
        self
    }

    pub(crate) fn user_property(&mut self, val: UserProperty) -> &mut Self {
        self.user_property.push(val);
        self
    }

    pub(crate) fn build(self) -> Option<Auth> {
        let is_shortened = self.reason.is_none()
            && self.authentication_method.is_none()
            && self.authentication_data.is_none()
            && self.reason_string.is_none()
            && self.user_property.is_empty();
        if is_shortened {
            return Some(Auth::default());
        }

        Some(Auth {
            reason: self.reason?,
            authentication_method: Some(self.authentication_method?),
            authentication_data: self.authentication_data,
            reason_string: self.reason_string,
            user_property: self.user_property,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes_short() {
        const FIXED_HDR: u8 = ((Auth::PACKET_ID as u8) << 4) as u8;
        const PACKET: [u8; 2] = [
            FIXED_HDR, 0, // Remaining length
        ];

        let packet = Auth::try_from_bytes(&PACKET);
        assert!(packet.is_some());
    }

    #[test]
    fn to_bytes_short() {
        const FIXED_HDR: u8 = ((Auth::PACKET_ID as u8) << 4) as u8;
        const EXPECTED: [u8; 2] = [
            FIXED_HDR, 0, // Remaining length
        ];

        let packet = Auth::default();
        let mut buf = [0u8; EXPECTED.len()];

        let result = packet.try_to_byte_buffer(&mut buf).unwrap();
        assert_eq!(result, EXPECTED);
    }
}
