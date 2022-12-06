use crate::core::{
    base_types::*,
    error::{
        CodecError, ConversionError, InvalidPacketHeader, InvalidPacketSize, InvalidPropertyLength,
        InvalidValue, UnexpectedProperty,
    },
    properties::*,
    utils::{ByteLen, Decoder, Encode, Encoder, PacketID, SizedPacket, TryDecode},
};
use bytes::{Bytes, BytesMut};
use core::mem;
use derive_builder::Builder;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DisconnectReason {
    Success = 0x00,
    DisconnectWithWillMessage = 0x04,
    UnspecifiedError = 0x80,
    MalformedPacket = 0x81,
    ProtocolError = 0x82,
    ImplementationSpecificError = 0x83,
    NotAuthorized = 0x87,
    ServerBusy = 0x89,
    ServerShuttingDown = 0x8b,
    KeepAliveTimeout = 0x8d,
    SessionTakenOver = 0x8e,
    TopicFilterInvalid = 0x8f,
    TopicNameInvalid = 0x90,
    ReceiveMaximumExcceeded = 0x93,
    TopicAliasInvalid = 0x94,
    PacketTooLarge = 0x95,
    MessageRateTooHigh = 0x96,
    QuotaExceeded = 0x97,
    AdministrativeAction = 0x98,
    PayloadFormatInvalid = 0x99,
    RetainNotSupported = 0x9a,
    QoSNotSupported = 0x9b,
    UseAnotherServer = 0x9c,
    ServerMoved = 0x9d,
    SharedSubscriptionsNotSupported = 0x9e,
    ConnectionRateExceeded = 0x9f,
    MaximumConnectTime = 0xa0,
    SubscriptionIdentifiersNotSupported = 0xa1,
    WildcardSubscriptionsNotSupported = 0xa2,
}

impl TryFrom<u8> for DisconnectReason {
    type Error = ConversionError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x00 => Ok(DisconnectReason::Success),
            0x04 => Ok(DisconnectReason::DisconnectWithWillMessage),
            0x80 => Ok(DisconnectReason::UnspecifiedError),
            0x81 => Ok(DisconnectReason::MalformedPacket),
            0x82 => Ok(DisconnectReason::ProtocolError),
            0x83 => Ok(DisconnectReason::ImplementationSpecificError),
            0x87 => Ok(DisconnectReason::NotAuthorized),
            0x89 => Ok(DisconnectReason::ServerBusy),
            0x8b => Ok(DisconnectReason::ServerShuttingDown),
            0x8d => Ok(DisconnectReason::KeepAliveTimeout),
            0x8e => Ok(DisconnectReason::SessionTakenOver),
            0x8f => Ok(DisconnectReason::TopicFilterInvalid),
            0x90 => Ok(DisconnectReason::TopicNameInvalid),
            0x93 => Ok(DisconnectReason::ReceiveMaximumExcceeded),
            0x94 => Ok(DisconnectReason::TopicAliasInvalid),
            0x95 => Ok(DisconnectReason::PacketTooLarge),
            0x96 => Ok(DisconnectReason::MessageRateTooHigh),
            0x97 => Ok(DisconnectReason::QuotaExceeded),
            0x98 => Ok(DisconnectReason::AdministrativeAction),
            0x99 => Ok(DisconnectReason::PayloadFormatInvalid),
            0x9a => Ok(DisconnectReason::RetainNotSupported),
            0x9b => Ok(DisconnectReason::QoSNotSupported),
            0x9c => Ok(DisconnectReason::UseAnotherServer),
            0x9d => Ok(DisconnectReason::ServerMoved),
            0x9e => Ok(DisconnectReason::SharedSubscriptionsNotSupported),
            0x9f => Ok(DisconnectReason::ConnectionRateExceeded),
            0xa0 => Ok(DisconnectReason::MaximumConnectTime),
            0xa1 => Ok(DisconnectReason::SubscriptionIdentifiersNotSupported),
            0xa2 => Ok(DisconnectReason::WildcardSubscriptionsNotSupported),
            _ => Err(InvalidValue.into()),
        }
    }
}

impl ByteLen for DisconnectReason {
    fn byte_len(&self) -> usize {
        mem::size_of::<u8>()
    }
}

impl Default for DisconnectReason {
    fn default() -> Self {
        Self::Success
    }
}

impl TryDecode for DisconnectReason {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        Self::try_from(u8::try_decode(bytes)?)
    }
}

impl Encode for DisconnectReason {
    fn encode(&self, buf: &mut BytesMut) {
        (*self as u8).encode(buf)
    }
}

#[derive(Builder)]
#[builder(build_fn(error = "CodecError"))]
pub(crate) struct DisconnectRx {
    #[builder(default)]
    pub(crate) reason: DisconnectReason,
    #[builder(default)]
    pub(crate) session_expiry_interval: SessionExpiryInterval,
    #[builder(setter(strip_option), default)]
    pub(crate) reason_string: Option<ReasonString>,
    #[builder(setter(strip_option), default)]
    pub(crate) server_reference: Option<ServerReference>,
    #[builder(setter(custom), default)]
    pub(crate) user_property: Vec<UserProperty>,
}

impl DisconnectRxBuilder {
    fn user_property(&mut self, value: UserProperty) {
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
}

impl DisconnectRx {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;
}

impl PacketID for DisconnectRx {
    const PACKET_ID: u8 = 14;
}

impl TryDecode for DisconnectRx {
    type Error = CodecError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        let mut builder = DisconnectRxBuilder::default();
        let mut decoder = Decoder::from(bytes);

        let fixed_hdr = decoder.try_decode::<u8>()?;
        if fixed_hdr != Self::FIXED_HDR {
            return Err(InvalidPacketHeader.into());
        }

        let remaining_len = decoder.try_decode::<VarSizeInt>()?;
        if remaining_len > decoder.remaining() {
            return Err(InvalidPacketSize.into());
        }

        let reason = decoder.try_decode::<DisconnectReason>()?;
        builder.reason(reason);

        if decoder.remaining() == 0 {
            return builder.build();
        }

        let byte_len = decoder.try_decode::<VarSizeInt>()?;
        if byte_len > decoder.remaining() {
            return Err(InvalidPropertyLength.into());
        }

        for property in decoder.iter::<Property>() {
            if let Err(err) = property {
                return Err(err.into());
            }

            match property.unwrap() {
                Property::SessionExpiryInterval(_) => {
                    // The Session Expiry Interval MUST NOT be sent on a DISCONNECT by the Server
                    return Err(UnexpectedProperty.into());
                }
                Property::ReasonString(val) => {
                    builder.reason_string(val);
                }
                Property::ServerReference(val) => {
                    builder.server_reference(val);
                }
                Property::UserProperty(val) => {
                    builder.user_property(val);
                }
                _ => {
                    return Err(UnexpectedProperty.into());
                }
            }
        }

        builder.build()
    }
}

impl<'a> DisconnectTx<'a> {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;

    fn property_len(&self) -> VarSizeInt {
        let session_expiry_interval_len = Some(&self.session_expiry_interval)
            .map(|val| {
                if *val == SessionExpiryInterval::default() {
                    return 0;
                }

                val.byte_len()
            })
            .unwrap();

        let reason_string_len = self
            .reason_string
            .as_ref()
            .map(|val| val.byte_len())
            .unwrap_or(0);

        let server_reference_len = self
            .server_reference
            .as_ref()
            .map(|val| val.byte_len())
            .unwrap_or(0);

        let user_property_len = self
            .user_property
            .iter()
            .map(|val| val.byte_len())
            .sum::<usize>();

        VarSizeInt::try_from(
            session_expiry_interval_len
                + reason_string_len
                + server_reference_len
                + user_property_len,
        )
        .unwrap()
    }

    fn remaining_len(&self) -> VarSizeInt {
        let property_len = self.property_len();
        VarSizeInt::try_from(
            mem::size_of::<DisconnectReason>() + property_len.len() + property_len.value() as usize,
        )
        .unwrap()
    }
}

impl<'a> PacketID for DisconnectTx<'a> {
    const PACKET_ID: u8 = 14;
}

impl<'a> SizedPacket for DisconnectTx<'a> {
    fn packet_len(&self) -> usize {
        let remaining_len = self.remaining_len();
        mem::size_of::<u8>() + remaining_len.len() + remaining_len.value() as usize
    }
}

#[derive(Builder)]
#[builder(build_fn(error = "CodecError"))]
pub(crate) struct DisconnectTx<'a> {
    #[builder(default)]
    pub(crate) reason: DisconnectReason,
    #[builder(default)]
    pub(crate) session_expiry_interval: SessionExpiryInterval,
    #[builder(setter(strip_option), default)]
    pub(crate) reason_string: Option<ReasonStringRef<'a>>,
    #[builder(setter(strip_option), default)]
    pub(crate) server_reference: Option<ServerReferenceRef<'a>>,
    #[builder(setter(custom), default)]
    pub(crate) user_property: Vec<UserPropertyRef<'a>>,
}

impl<'a> DisconnectTxBuilder<'a> {
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
}

impl<'a> Encode for DisconnectTx<'a> {
    fn encode(&self, buf: &mut BytesMut) {
        let mut encoder = Encoder::from(buf);

        encoder.encode(Self::FIXED_HDR);
        encoder.encode(self.remaining_len());
        encoder.encode(self.reason);
        encoder.encode(self.property_len());

        if self.session_expiry_interval != SessionExpiryInterval::default() {
            encoder.encode(self.session_expiry_interval);
        }

        if let Some(val) = self.reason_string {
            encoder.encode(val);
        }

        if let Some(val) = self.server_reference {
            encoder.encode(val);
        }

        for val in self.user_property.iter().copied() {
            encoder.encode(val)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::utils::PropertyID;

    const PACKET: [u8; 25] = [
        ((DisconnectRx::PACKET_ID as u8) << 4) as u8,
        23, // Remaining length
        (DisconnectReason::Success as u8),
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

    #[test]
    fn from_bytes_0() {
        let packet = DisconnectRx::try_decode(Bytes::from_static(&PACKET)).unwrap();

        assert_eq!(packet.reason, DisconnectReason::Success);
        assert_eq!(
            packet.reason_string.unwrap(),
            ReasonString::from(UTF8String(Bytes::from_static("Success".as_bytes())))
        );
        assert_eq!(packet.user_property.len(), 1);
        assert_eq!(
            packet.user_property[0],
            UserProperty::from(UTF8StringPair(
                Bytes::from_static("key".as_bytes()),
                Bytes::from_static("val".as_bytes())
            ))
        );
    }

    #[test]
    fn to_bytes_0() {
        let mut builder = DisconnectTxBuilder::default();

        builder.reason(DisconnectReason::Success);
        builder.reason_string(ReasonStringRef::from(UTF8StringRef("Success")));
        builder.user_property(UserPropertyRef::from(UTF8StringPairRef("key", "val")));

        let packet = builder.build().unwrap();

        let mut buf = BytesMut::new();
        packet.encode(&mut buf);

        assert_eq!(&buf.split().freeze()[..], &PACKET);
    }
}
