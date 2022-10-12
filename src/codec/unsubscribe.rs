use crate::core::{
    base_types::*,
    error::{CodecError, InsufficientBufferSize},
    properties::*,
    utils::{ByteWriter, PacketID, SizedPacket, SizedProperty, ToByteBuffer, TryToByteBuffer},
};

pub(crate) struct UnsubscribeProperties {
    pub(crate) user_property: Vec<UserProperty>,
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

        writer.write(&property_len);

        for val in self.user_property.iter() {
            writer.write(val);
        }

        result
    }
}

pub(crate) struct Unsubscribe {
    pub(crate) packet_identifier: NonZero<u16>,
    pub(crate) properties: UnsubscribeProperties,
    pub(crate) payload: Vec<String>,
}

impl Unsubscribe {
    const FIXED_HDR: u8 = (Self::PACKET_ID << 4) | 0b0010;

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
        writer.write(&self.properties);

        for val in self.payload.iter() {
            writer.write(val);
        }

        Ok(result)
    }
}

#[derive(Default)]
pub(crate) struct UnsubscribeBuilder {
    packet_identifier: Option<NonZero<u16>>,
    user_property: Vec<UserProperty>,
    payload: Vec<String>,
}

impl UnsubscribeBuilder {
    pub(crate) fn packet_identifier(&mut self, val: NonZero<u16>) -> &mut Self {
        self.packet_identifier = Some(val);
        self
    }

    pub(crate) fn user_property(&mut self, val: StringPair) -> &mut Self {
        self.user_property.push(val.into());
        self
    }

    pub(crate) fn payload(&mut self, val: String) -> &mut Self {
        self.payload.push(val);
        self
    }

    pub(crate) fn build(self) -> Option<Unsubscribe> {
        if self.payload.is_empty() {
            return None; // Unsubscribe packet with no payload is a Protocol Error
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
    use super::*;

    #[test]
    fn to_bytes_0() {
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

        let mut builder = UnsubscribeBuilder::default();
        builder.packet_identifier(NonZero::from(13));
        builder.payload(String::from("a/b"));
        builder.payload(String::from("c/d"));

        let packet = builder.build().unwrap();
        let mut buf = [0u8; EXPECTED.len()];
        let result = packet.try_to_byte_buffer(&mut buf).unwrap();

        assert_eq!(result, EXPECTED);
    }
}
