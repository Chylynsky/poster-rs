use bytes::{Bytes, BytesMut};

use crate::core::{
    base_types::*,
    error::{
        CodecError, InvalidPacketHeader, InvalidPacketSize, InvalidPropertyLength,
        UnexpectedProperty,
    },
    properties::*,
    utils::{ByteLen, Decoder, Encode, Encoder, PacketID, SizedPacket, TryDecode},
};
use core::{cmp::PartialEq, mem};
use derive_builder::Builder;

#[derive(Builder)]
#[builder(build_fn(error = "CodecError"))]
pub(crate) struct AckRx<ReasonT>
where
    ReasonT: Default,
{
    pub(crate) packet_identifier: NonZero<u16>,
    #[builder(default)]
    pub(crate) reason: ReasonT,

    #[builder(setter(strip_option), default)]
    pub(crate) reason_string: Option<ReasonString>,
    #[builder(setter(custom), default)]
    pub(crate) user_property: Vec<UserProperty>,
}

impl<ReasonT> AckRxBuilder<ReasonT>
where
    ReasonT: Default,
{
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

impl<ReasonT> AckRx<ReasonT>
where
    Self: PacketID,
    ReasonT: Default + PartialEq + ByteLen,
{
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;
}

impl<ReasonT> TryDecode for AckRx<ReasonT>
where
    Self: PacketID,
    ReasonT: Default + PartialEq + TryDecode + ByteLen + Clone,
    <ReasonT as TryDecode>::Error: Into<CodecError>,
{
    type Error = CodecError;

    fn try_decode(buf: Bytes) -> Result<Self, Self::Error> {
        let mut builder = AckRxBuilder::<ReasonT>::default();
        let mut decoder = Decoder::from(buf);

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

        // When remaining length is 2, the Reason is 0x00 and there are no properties.
        if remaining_len.value() == 2 {
            return builder.build();
        }

        let reason = decoder.try_decode::<ReasonT>().map_err(|err| err.into())?;
        builder.reason(reason);

        let byte_len = decoder.try_decode::<VarSizeInt>()?;
        if byte_len > decoder.remaining() {
            return Err(InvalidPropertyLength.into());
        }

        for maybe_property in decoder.iter::<Property>() {
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
                Err(err) => {
                    return Err(err.into());
                }
            }
        }

        builder.build()
    }
}

#[derive(Builder)]
#[builder(build_fn(error = "CodecError"))]
pub(crate) struct AckTx<'a, ReasonT>
where
    ReasonT: Default,
{
    pub(crate) packet_identifier: NonZero<u16>,
    #[builder(default)]
    pub(crate) reason: ReasonT,

    #[builder(setter(strip_option), default)]
    pub(crate) reason_string: Option<ReasonStringRef<'a>>,
    #[builder(setter(custom), default)]
    pub(crate) user_property: Vec<UserPropertyRef<'a>>,
}

impl<'a, ReasonT> AckTxBuilder<'a, ReasonT>
where
    ReasonT: Default,
{
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

impl<'a, ReasonT> AckTx<'a, ReasonT>
where
    Self: PacketID,
    ReasonT: Default + PartialEq + ByteLen,
{
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;

    fn property_len(&self) -> VarSizeInt {
        VarSizeInt::try_from(
            self.reason_string
                .as_ref()
                .map(ByteLen::byte_len)
                .unwrap_or(0)
                + self
                    .user_property
                    .iter()
                    .map(ByteLen::byte_len)
                    .sum::<usize>(),
        )
        .unwrap()
    }

    fn remaining_len(&self) -> VarSizeInt {
        let byte_len = self.property_len();
        let is_shortened = self.reason == ReasonT::default() && byte_len.value() == 0;
        if is_shortened {
            return VarSizeInt::try_from(self.packet_identifier.byte_len()).unwrap();
        }

        VarSizeInt::try_from(
            self.packet_identifier.byte_len()
                + self.reason.byte_len()
                + byte_len.len()
                + byte_len.value() as usize,
        )
        .unwrap()
    }
}

impl<'a, ReasonT> SizedPacket for AckTx<'a, ReasonT>
where
    Self: PacketID,
    ReasonT: Default + PartialEq + ByteLen,
{
    fn packet_len(&self) -> usize {
        let remaining_len = self.remaining_len();
        mem::size_of_val(&Self::FIXED_HDR) + remaining_len.len() + remaining_len.value() as usize
    }
}

impl<'a, ReasonT> Encode for AckTx<'a, ReasonT>
where
    AckTx<'a, ReasonT>: PacketID,
    ReasonT: Default + Encode + PartialEq + ByteLen + Copy,
{
    fn encode(&self, buf: &mut BytesMut) {
        let rem_len = self.remaining_len();
        let mut encoder = Encoder::from(buf);

        encoder.encode(Self::FIXED_HDR);

        encoder.encode(rem_len);
        encoder.encode(self.packet_identifier);

        if rem_len.value() == 2 {
            return;
        }

        encoder.encode(self.reason);
        encoder.encode(self.property_len());

        if self.reason_string.is_some() {
            encoder.encode(self.reason_string.unwrap());
        }

        for property in self.user_property.iter().copied() {
            encoder.encode(property);
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::core::utils::PropertyID;
    use core::{cmp::PartialEq, fmt::Debug};

    pub(crate) fn from_bytes_impl<ReasonT>()
    where
        ReasonT: Debug + PartialEq + Default + TryDecode + ByteLen + Clone,
        AckRx<ReasonT>: PacketID,
        <ReasonT as TryDecode>::Error: Debug + Into<CodecError>,
    {
        let fixed_hdr = ((AckRx::<ReasonT>::PACKET_ID as u8) << 4) as u8;
        let input_packet = [
            fixed_hdr,
            25,   // Remaining length
            0x45, // Packet ID MSB
            0x73, // Packet ID LSB
            0,    // Success
            21,   // Property length
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

        let packet = AckRx::<ReasonT>::try_decode(Bytes::copy_from_slice(&input_packet)).unwrap();

        assert_eq!(packet.packet_identifier, NonZero::try_from(0x4573).unwrap());
        assert_eq!(packet.reason, ReasonT::default());
        assert_eq!(
            UTF8String::from(packet.reason_string.unwrap()).0,
            "Success".as_bytes()
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

    pub(crate) fn from_bytes_short_impl<ReasonT>()
    where
        ReasonT: Debug + PartialEq + Default + TryDecode + ByteLen + Clone,
        AckRx<ReasonT>: PacketID,
        <ReasonT as TryDecode>::Error: Debug + Into<CodecError>,
    {
        let fixed_hdr = ((AckRx::<ReasonT>::PACKET_ID as u8) << 4) as u8;
        let input_packet = [
            fixed_hdr, 2,    // Remaining length
            0x45, // Packet ID MSB
            0x73, // Packet ID LSB
        ];

        let packet = AckRx::<ReasonT>::try_decode(Bytes::copy_from_slice(&input_packet)).unwrap();

        assert_eq!(packet.packet_identifier, 0x4573);
    }

    pub(crate) fn to_bytes_impl<'a, ReasonT>()
    where
        ReasonT: Copy + PartialEq + Default + Encode + ByteLen,
        AckTx<'a, ReasonT>: PacketID,
    {
        let fixed_hdr = ((AckTx::<ReasonT>::PACKET_ID as u8) << 4) as u8;
        let expected_packet = [
            fixed_hdr,
            25,   // Remaining length
            0x45, // Packet ID MSB
            0x73, // Packet ID LSB
            0,    // Success
            21,   // Property length
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

        let mut builder = AckTxBuilder::<ReasonT>::default();
        builder.packet_identifier(NonZero::try_from(0x4573).unwrap());
        builder.reason(ReasonT::default());
        builder.reason_string(ReasonStringRef::from(UTF8StringRef("Success")));
        builder.user_property(UserPropertyRef::from(UTF8StringPairRef("key", "val")));

        let packet = builder.build().unwrap();
        let mut buf = BytesMut::new();
        packet.encode(&mut buf);
        assert_eq!(buf.split().freeze(), &expected_packet[..]);
    }

    pub(crate) fn to_bytes_short_impl<'a, ReasonT>()
    where
        ReasonT: Copy + PartialEq + Default + Encode + ByteLen,
        AckTx<'a, ReasonT>: PacketID,
    {
        let fixed_hdr = ((AckTx::<ReasonT>::PACKET_ID as u8) << 4) as u8;
        let expected_packet = [
            fixed_hdr, 2,    // Remaining length
            0x45, // Packet ID MSB
            0x73, // Packet ID LSB
        ];

        let mut builder = AckTxBuilder::<ReasonT>::default();
        builder.packet_identifier(NonZero::try_from(0x4573).unwrap());

        let packet = builder.build().unwrap();
        let mut buf = BytesMut::new();
        packet.encode(&mut buf);
        assert_eq!(buf.split().freeze(), &expected_packet[..]);
    }
}
