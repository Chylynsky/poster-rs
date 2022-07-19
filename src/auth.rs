use crate::{
    base_types::*,
    properties::*,
    utils::{TryFromBytes, TryFromIterator},
};
use std::mem;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AuthReason {
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

pub struct Auth {
    reason: AuthReason,

    authentication_data: Option<AuthenticationData>,
    authentication_method: Option<AuthenticationMethod>,
    reason_string: Option<ReasonString>,
    user_property: Vec<UserProperty>,
}

#[derive(Default)]
pub struct AuthPacketBuilder {
    reason: Option<AuthReason>,
    authentication_data: Option<AuthenticationData>,
    authentication_method: Option<AuthenticationMethod>,
    reason_string: Option<ReasonString>,
    user_property: Vec<UserProperty>,
}

impl AuthPacketBuilder {
    fn reason(&mut self, val: AuthReason) -> &mut Self {
        self.reason = Some(val);
        self
    }

    fn authentication_data(&mut self, val: AuthenticationData) -> &mut Self {
        self.authentication_data = Some(val);
        self
    }

    fn authentication_method(&mut self, val: AuthenticationMethod) -> &mut Self {
        self.authentication_method = Some(val);
        self
    }

    fn reason_string(&mut self, val: ReasonString) -> &mut Self {
        self.reason_string = Some(val);
        self
    }

    fn user_property(&mut self, val: UserProperty) -> &mut Self {
        self.user_property.push(val);
        self
    }

    fn build(self) -> Option<Auth> {
        Some(Auth {
            reason: self.reason?,
            authentication_data: self.authentication_data,
            authentication_method: self.authentication_method,
            reason_string: self.reason_string,
            user_property: self.user_property,
        })
    }
}

impl Auth {
    pub const PACKET_ID: isize = 15;
}

impl TryFromBytes for Auth {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut packet_builder = AuthPacketBuilder::default();

        let mut iter = bytes.iter().copied();
        let fixed_hdr = iter.next()?;

        debug_assert!(fixed_hdr >> 4 == Self::PACKET_ID as u8);
        let remaining_len = VarSizeInt::try_from_iter(iter)?;
        if mem::size_of_val(&fixed_hdr) + remaining_len.len() > bytes.len() {
            return None;
        }

        let (_, var_hdr) = bytes.split_at(mem::size_of_val(&fixed_hdr) + remaining_len.len());
        if remaining_len.value() as usize > var_hdr.len() {
            return None;
        }

        let (var_hdr, _) = var_hdr.split_at(remaining_len.into());
        iter = var_hdr.iter().copied();

        let reason = AuthReason::try_from(iter.next()?)?;
        let (_, var_hdr) = var_hdr.split_at(1);
        packet_builder.reason(reason);

        let property_len = VarSizeInt::try_from_iter(iter)?;
        if property_len.len() > var_hdr.len() {
            return None;
        }

        let (_, properties) = var_hdr.split_at(property_len.len());

        for property in PropertyIterator::from(properties) {
            match property {
                Property::AuthenticationData(val) => {
                    packet_builder.authentication_data(val);
                }
                Property::AuthenticationMethod(val) => {
                    packet_builder.authentication_method(val);
                }
                Property::ReasonString(val) => {
                    packet_builder.reason_string(val);
                }
                Property::UserProperty(val) => {
                    packet_builder.user_property(val);
                }
                _ => {
                    return None;
                }
            }
        }

        packet_builder.build()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes() {
        const FIXED_HDR: u8 = ((Auth::PACKET_ID as u8) << 4) as u8;
        const PACKET: [u8; 25] = [
            FIXED_HDR,
            23, // Remaining length
            (AuthReason::Success as u8),
            21, // Property length
            (ReasonString::PROPERTY_ID),
            0, // Reason string size
            7,
            b'S',
            b'u',
            b'c',
            b'c',
            b'e',
            b's',
            b's',
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
        ];

        let packet = Auth::try_from_bytes(&PACKET).unwrap();

        assert_eq!(packet.reason, AuthReason::Success);
        assert_eq!(packet.reason_string.unwrap().0, "Success");
        assert_eq!(packet.user_property.len(), 1);

        assert_eq!(
            packet.user_property[0],
            UserProperty((String::from("key"), String::from("val")))
        );
    }
}
