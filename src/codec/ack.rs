use crate::core::{
    base_types::*,
    properties::*,
    utils::{
        ByteReader, ByteWriter, PacketID, SizedPacket, SizedProperty, ToByteBuffer, TryFromBytes,
        TryToByteBuffer,
    },
};
use std::{cmp::PartialEq, mem};

pub(crate) struct Ack<ReasonT> {
    pub(crate) packet_identifier: NonZero<TwoByteInteger>,
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
{
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut builder = AckBuilder::<ReasonT>::default();
        let mut reader = ByteReader::from(bytes);

        let fixed_hdr = reader.try_read::<Byte>()?;
        if fixed_hdr != Self::FIXED_HDR {
            return None; // Invalid header
        }

        let remaining_len = reader.try_read::<VarSizeInt>()?;
        let packet_size =
            mem::size_of_val(&fixed_hdr) + remaining_len.len() + remaining_len.value() as usize;
        if packet_size > bytes.len() {
            return None; // Invalid packet size
        }

        let packet_id = reader.try_read::<NonZero<TwoByteInteger>>()?;
        builder.packet_identifier(packet_id);

        // When remaining length is 2, the Reason is 0x00 and there are no properties.
        if remaining_len.value() == 2 {
            return builder.build();
        }

        let reason = reader.try_read::<ReasonT>()?;
        builder.reason(reason);

        let property_len = reader.try_read::<VarSizeInt>()?;
        if property_len.value() as usize > reader.remaining() {
            return None; // Invalid property length
        }

        for property in PropertyIterator::from(reader.get_buf()) {
            match property {
                Property::ReasonString(val) => {
                    builder.reason_string(val.0);
                }
                Property::UserProperty(val) => {
                    builder.user_property(val.0);
                }
                _ => {
                    return None;
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
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.packet_len())?;
        let mut writer = ByteWriter::from(result);

        writer.write(&Self::FIXED_HDR);

        let remaining_len = self.remaining_len();
        debug_assert!(remaining_len.value() as usize <= writer.remaining());

        writer.write(&remaining_len);
        writer.write(&self.packet_identifier);

        if remaining_len.value() == 2 {
            return Some(result);
        }

        writer.write(&self.reason);
        writer.write(&self.property_len());

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
pub(crate) struct AckBuilder<ReasonT>
where
    ReasonT: Default,
{
    packet_identifier: Option<NonZero<TwoByteInteger>>,
    reason: Option<ReasonT>,
    reason_string: Option<ReasonString>,
    user_property: Vec<UserProperty>,
}

impl<ReasonT> AckBuilder<ReasonT>
where
    ReasonT: Default,
{
    pub(crate) fn packet_identifier(&mut self, val: NonZero<TwoByteInteger>) -> &mut Self {
        self.packet_identifier = Some(val);
        self
    }

    pub(crate) fn reason(&mut self, val: ReasonT) -> &mut Self {
        self.reason = Some(val);
        self
    }

    pub(crate) fn reason_string(&mut self, val: UTF8String) -> &mut Self {
        self.reason_string = Some(ReasonString(val));
        self
    }

    pub(crate) fn user_property(&mut self, val: UTF8StringPair) -> &mut Self {
        self.user_property.push(UserProperty(val));
        self
    }

    pub(crate) fn build(self) -> Option<Ack<ReasonT>> {
        Some(Ack {
            packet_identifier: self.packet_identifier?,
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
    use std::{cmp::PartialEq, fmt::Debug};

    pub(crate) fn from_bytes_impl<ReasonT>()
    where
        ReasonT: Debug + PartialEq + Default + TryFromBytes + SizedProperty,
        Ack<ReasonT>: PacketID,
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
        assert_eq!(packet.reason_string.unwrap().0, "Success");
        assert_eq!(packet.user_property.len(), 1);
        assert_eq!(
            packet.user_property[0],
            UserProperty((String::from("key"), String::from("val")))
        );
    }

    pub(crate) fn from_bytes_short_impl<ReasonT>()
    where
        ReasonT: Debug + PartialEq + Default + TryFromBytes + SizedProperty,
        Ack<ReasonT>: PacketID,
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
