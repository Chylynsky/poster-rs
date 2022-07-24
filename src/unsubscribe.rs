use crate::{
    base_types::{Byte, TwoByteInteger, UTF8String, VarSizeInt},
    properties::UserProperty,
    utils::{ByteWriter, PacketID, SizedPacket, SizedProperty, ToByteBuffer, TryToByteBuffer},
};

pub(crate) struct UnsubscribeProperties {
    user_property: Vec<UserProperty>,
}

impl SizedProperty for UnsubscribeProperties {
    fn property_len(&self) -> usize {
        self.user_property
            .iter()
            .map(|val| val.property_len())
            .sum::<usize>()
    }
}

impl ToByteBuffer for UnsubscribeProperties {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let property_len = VarSizeInt::from(self.property_len());
        let len = property_len.len() + property_len.value() as usize;

        debug_assert!(len <= buf.len());

        let result = &mut buf[0..len];
        let mut writer = ByteWriter::from(result);

        for val in self.user_property.iter() {
            writer.write(val);
        }

        result
    }
}

pub(crate) struct Unsubscribe {
    packet_identifier: TwoByteInteger,
    properties: UnsubscribeProperties,
    payload: Vec<UTF8String>,
}

impl Unsubscribe {
    const FIXED_HDR: Byte = (Self::PACKET_ID << 4) | 0b0010;

    fn remaining_len(&self) -> VarSizeInt {
        let property_len = VarSizeInt::from(self.properties.property_len());
        VarSizeInt::from(
            self.packet_identifier.property_len()
                + property_len.len()
                + property_len.value() as usize
                + self
                    .payload
                    .iter()
                    .map(|val| val.property_len())
                    .sum::<usize>(),
        )
    }
}

impl PacketID for Unsubscribe {
    const PACKET_ID: u8 = 10;
}

impl SizedPacket for Unsubscribe {
    fn packet_len(&self) -> usize {
        let remaining_len = self.remaining_len();
        Self::FIXED_HDR.property_len() + remaining_len.len() + remaining_len.value() as usize
    }
}

impl TryToByteBuffer for Unsubscribe {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let packet_len = self.packet_len();
        if packet_len > buf.len() {
            return None;
        }

        let result = &mut buf[0..packet_len];
        let mut writer = ByteWriter::from(result);

        writer.write(&Self::FIXED_HDR);
        writer.write(&self.remaining_len());
        writer.write(&self.packet_identifier);
        writer.write(&self.properties);

        for val in self.payload.iter() {
            writer.write(val);
        }

        Some(result)
    }
}

#[derive(Default)]
pub(crate) struct UnsubscribePacketBuilder {
    packet_identifier: Option<TwoByteInteger>,
    user_property: Vec<UserProperty>,
    payload: Vec<UTF8String>,
}

impl UnsubscribePacketBuilder {
    pub(crate) fn packet_identifier(&mut self, val: TwoByteInteger) -> &mut Self {
        self.packet_identifier = Some(val);
        self
    }

    pub(crate) fn user_property(&mut self, val: UserProperty) -> &mut Self {
        self.user_property.push(val);
        self
    }

    pub(crate) fn payload(&mut self, val: UTF8String) -> &mut Self {
        self.payload.push(val);
        self
    }

    pub(crate) fn build(self) -> Option<Unsubscribe> {
        if self.payload.is_empty() {
            return None;
        }

        let properties = UnsubscribeProperties {
            user_property: self.user_property,
        };

        Some(Unsubscribe {
            packet_identifier: self.packet_identifier?,
            properties,
            payload: self.payload,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        unsubscribe::{Unsubscribe, UnsubscribePacketBuilder},
        utils::TryToByteBuffer,
    };

    #[test]
    fn to_bytes() {
        const EXPECTED: [u8; 15] = [
            Unsubscribe::FIXED_HDR,
            13, // Remaining length
            0,  // Packet ID MSB
            13, // Packet ID LSB
            0,  // Property length
            0,  // Topic length MSB
            3,  // Topic length LSB
            b'a',
            b'/',
            b'b',
            0, // Topic length MSB
            3, // Topic length LSB
            b'c',
            b'/',
            b'd',
        ];

        let mut builder = UnsubscribePacketBuilder::default();
        builder.packet_identifier(13);
        builder.payload(String::from("a/b"));
        builder.payload(String::from("c/d"));

        let packet = builder.build().unwrap();
        let mut buf = [0u8; EXPECTED.len()];
        let result = packet.try_to_byte_buffer(&mut buf).unwrap();

        assert_eq!(result, EXPECTED);
    }
}
