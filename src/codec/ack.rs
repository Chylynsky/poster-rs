use crate::core::{
    base_types::*,
    error::{
        CodecError, InsufficientBufferSize, InvalidPacketHeader, InvalidPacketSize,
        InvalidPropertyLength, MandatoryPropertyMissing, UnexpectedProperty,
    },
    properties::*,
    utils::{
        ByteReader, ByteWriter, PacketID, SizedPacket, SizedProperty, ToByteBuffer, TryFromBytes,
        TryToByteBuffer,
    },
};
use core::{cmp::PartialEq, mem};

pub(crate) struct Ack<ReasonT> {
    pub(crate) packet_identifier: NonZero<u16>,
    pub(crate) reason: ReasonT,

    pub(crate) reason_string: Option<ReasonString>,
    pub(crate) user_property: Vec<UserProperty>,
}

impl<ReasonT> Ack<ReasonT>
where
    Self: PacketID,
    ReasonT: Default + PartialEq + SizedProperty,
{
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;

    fn property_len(&self) -> VarSizeInt {
        VarSizeInt::from(
            self.reason_string
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
        let is_shortened = self.reason == ReasonT::default() && property_len.value() == 0;
        if is_shortened {
            return VarSizeInt::from(self.packet_identifier.property_len());
        }

        VarSizeInt::from(
            self.packet_identifier.property_len()
                + self.reason.property_len()
                + property_len.len()
                + property_len.value() as usize,
        )
    }
}

impl<ReasonT> SizedPacket for Ack<ReasonT>
where
    Self: PacketID,
    ReasonT: Default + PartialEq + SizedProperty,
{
    fn packet_len(&self) -> usize {
        let remaining_len = self.remaining_len();
        mem::size_of_val(&Self::FIXED_HDR) + remaining_len.len() + remaining_len.value() as usize
    }
}

impl<ReasonT> TryFromBytes for Ack<ReasonT>
where
    Self: PacketID,
    ReasonT: Default + PartialEq + TryFromBytes + SizedProperty,
    <ReasonT as TryFromBytes>::Error: Into<CodecError>,
{
    type Error = CodecError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut builder = AckBuilder::<ReasonT>::default();
        let mut reader = ByteReader::from(bytes);

        let fixed_hdr = reader
            .try_read::<u8>()
            .map_err(CodecError::from)
            .and_then(|val| {
                if val != Self::FIXED_HDR {
                    return Err(InvalidPacketHeader.into());
                }

                Ok(val)
            })?;

        let remaining_len = reader.try_read::<VarSizeInt>()?;
        let packet_size =
            mem::size_of_val(&fixed_hdr) + remaining_len.len() + remaining_len.value() as usize;
        if packet_size > bytes.len() {
            return Err(InvalidPacketSize.into());
        }

        let packet_id = reader.try_read::<NonZero<u16>>()?;
        builder.packet_identifier(packet_id);

        // When remaining length is 2, the Reason is 0x00 and there are no properties.
        if remaining_len.value() == 2 {
            return builder.build();
        }

        let reason = reader.try_read::<ReasonT>().map_err(|err| err.into())?;
        builder.reason(reason);

        let property_len = reader.try_read::<VarSizeInt>()?;
        if property_len.value() as usize > reader.remaining() {
            return Err(InvalidPropertyLength.into());
        }

        for property in PropertyIterator::from(reader.get_buf()) {
            if let Err(err) = property {
                return Err(err.into());
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

        builder.build()
    }
}

impl<ReasonT> TryToByteBuffer for Ack<ReasonT>
where
    Self: PacketID,
    ReasonT: Default + SizedProperty + PartialEq + ToByteBuffer,
{
    type Error = CodecError;

    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Result<&'a [u8], Self::Error> {
        let result = buf
            .get_mut(0..self.packet_len())
            .ok_or(InsufficientBufferSize)?;
        let mut writer = ByteWriter::from(result);

        writer.write(&Self::FIXED_HDR);

        let remaining_len = self.remaining_len();
        debug_assert!(remaining_len.value() as usize <= writer.remaining());

        writer.write(&remaining_len);
        writer.write(&self.packet_identifier);

        if remaining_len.value() == 2 {
            return Ok(result);
        }

        writer.write(&self.reason);
        writer.write(&self.property_len());

        if let Some(val) = self.reason_string.as_ref() {
            writer.write(val);
        }

        for val in self.user_property.iter() {
            writer.write(val);
        }

        Ok(result)
    }
}

#[derive(Default)]
pub(crate) struct AckBuilder<ReasonT>
where
    ReasonT: Default,
{
    packet_identifier: Option<NonZero<u16>>,
    reason: Option<ReasonT>,
    reason_string: Option<ReasonString>,
    user_property: Vec<UserProperty>,
}

impl<ReasonT> AckBuilder<ReasonT>
where
    ReasonT: Default,
{
    pub(crate) fn packet_identifier(&mut self, val: NonZero<u16>) -> &mut Self {
        self.packet_identifier = Some(val);
        self
    }

    pub(crate) fn reason(&mut self, val: ReasonT) -> &mut Self {
        self.reason = Some(val);
        self
    }

    pub(crate) fn reason_string(&mut self, val: String) -> &mut Self {
        self.reason_string = Some(val.into());
        self
    }

    pub(crate) fn user_property(&mut self, val: StringPair) -> &mut Self {
        self.user_property.push(val.into());
        self
    }

    pub(crate) fn build(self) -> Result<Ack<ReasonT>, CodecError> {
        Ok(Ack {
            packet_identifier: self.packet_identifier.ok_or(MandatoryPropertyMissing)?,
            reason: self.reason.unwrap_or_default(),
            reason_string: self.reason_string,
            user_property: self.user_property,
        })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::core::utils::PropertyID;
    use core::{cmp::PartialEq, fmt::Debug};

    pub(crate) fn from_bytes_impl<ReasonT>()
    where
        ReasonT: Debug + PartialEq + Default + TryFromBytes + SizedProperty,
        Ack<ReasonT>: PacketID,
        <ReasonT as TryFromBytes>::Error: Debug + Into<CodecError>,
    {
        let fixed_hdr = ((Ack::<ReasonT>::PACKET_ID as u8) << 4) as u8;
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

        let packet = Ack::<ReasonT>::try_from_bytes(&input_packet).unwrap();

        assert_eq!(packet.packet_identifier, NonZero::from(0x4573));
        assert_eq!(packet.reason, ReasonT::default());
        assert_eq!(
            String::from(packet.reason_string.unwrap()),
            String::from("Success")
        );
        assert_eq!(packet.user_property.len(), 1);
        assert_eq!(
            packet.user_property[0],
            UserProperty::from((String::from("key"), String::from("val")))
        );
    }

    pub(crate) fn from_bytes_short_impl<ReasonT>()
    where
        ReasonT: Debug + PartialEq + Default + TryFromBytes + SizedProperty,
        Ack<ReasonT>: PacketID,
        <ReasonT as TryFromBytes>::Error: Debug + Into<CodecError>,
    {
        let fixed_hdr = ((Ack::<ReasonT>::PACKET_ID as u8) << 4) as u8;
        let input_packet = [
            fixed_hdr, 2,    // Remaining length
            0x45, // Packet ID MSB
            0x73, // Packet ID LSB
        ];

        let packet = Ack::<ReasonT>::try_from_bytes(&input_packet).unwrap();

        assert_eq!(packet.packet_identifier, 0x4573.into());
    }

    pub(crate) fn to_bytes_impl<ReasonT>()
    where
        ReasonT: PartialEq + Default + TryFromBytes,
        Ack<ReasonT>: PacketID + TryToByteBuffer,
        <Ack<ReasonT> as TryToByteBuffer>::Error: Debug + Into<CodecError>,
    {
        let fixed_hdr = ((Ack::<ReasonT>::PACKET_ID as u8) << 4) as u8;
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

        let mut builder = AckBuilder::<ReasonT>::default();
        builder.packet_identifier(NonZero::from(0x4573));
        builder.reason(ReasonT::default());
        builder.reason_string(String::from("Success"));
        builder.user_property((String::from("key"), String::from("val")));

        let packet = builder.build().unwrap();
        let mut buf = vec![0; expected_packet.len()];

        let result = packet.try_to_byte_buffer(&mut buf).unwrap();

        assert_eq!(result, expected_packet);
    }

    pub(crate) fn to_bytes_short_impl<ReasonT>()
    where
        ReasonT: PartialEq + Default + TryFromBytes,
        Ack<ReasonT>: PacketID + TryToByteBuffer,
        <Ack<ReasonT> as TryToByteBuffer>::Error: Debug,
    {
        let fixed_hdr = ((Ack::<ReasonT>::PACKET_ID as u8) << 4) as u8;
        let expected_packet = [
            fixed_hdr, 2,    // Remaining length
            0x45, // Packet ID MSB
            0x73, // Packet ID LSB
        ];

        let mut builder = AckBuilder::<ReasonT>::default();
        builder.packet_identifier(NonZero::from(0x4573));

        let packet = builder.build().unwrap();
        let mut buf = vec![0; expected_packet.len()];

        let result = packet.try_to_byte_buffer(&mut buf).unwrap();

        assert_eq!(result, expected_packet);
    }
}
