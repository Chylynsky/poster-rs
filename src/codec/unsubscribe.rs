use crate::core::{
    base_types::*,
    error::{CodecError, MandatoryPropertyMissing},
    properties::*,
    utils::{ByteLen, Encode, Encoder, PacketID, SizedPacket},
};
use bytes::BytesMut;
use derive_builder::Builder;

#[derive(Builder)]
#[builder(build_fn(error = "CodecError", validate = "Self::validate"))]
pub(crate) struct UnsubscribeTx<'a> {
    pub(crate) packet_identifier: NonZero<u16>,
    #[builder(setter(custom), default)]
    pub(crate) user_property: Vec<UserPropertyRef<'a>>,
    #[builder(setter(custom), default)]
    pub(crate) payload: Vec<UTF8StringRef<'a>>,
}

impl<'a> UnsubscribeTxBuilder<'a> {
    fn validate(&self) -> Result<(), CodecError> {
        if self.payload.is_none() {
            Err(MandatoryPropertyMissing.into()) // Empty payload is a protocol error
        } else {
            Ok(())
        }
    }

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

    pub(crate) fn payload(&mut self, topic: UTF8StringRef<'a>) {
        match self.payload.as_mut() {
            Some(payload) => {
                payload.push(topic);
            }
            None => {
                self.payload = Some(Vec::new());
                self.payload.as_mut().unwrap().push(topic);
            }
        }
    }
}

impl<'a> UnsubscribeTx<'a> {
    const FIXED_HDR: u8 = (Self::PACKET_ID << 4) | 0b0010;

    fn property_len(&self) -> VarSizeInt {
        VarSizeInt::try_from(
            self.user_property
                .iter()
                .map(|val| val.byte_len())
                .sum::<usize>(),
        )
        .unwrap()
    }

    fn remaining_len(&self) -> VarSizeInt {
        let property_len = self.property_len();
        VarSizeInt::try_from(
            self.packet_identifier.byte_len()
                + property_len.len()
                + property_len.value() as usize
                + self.payload.iter().map(|val| val.byte_len()).sum::<usize>(),
        )
        .unwrap()
    }
}

impl<'a> PacketID for UnsubscribeTx<'a> {
    const PACKET_ID: u8 = 10;
}

impl<'a> SizedPacket for UnsubscribeTx<'a> {
    fn packet_len(&self) -> usize {
        let remaining_len = self.remaining_len();
        Self::FIXED_HDR.byte_len() + remaining_len.len() + remaining_len.value() as usize
    }
}

impl<'a> Encode for UnsubscribeTx<'a> {
    fn encode(&self, buf: &mut BytesMut) {
        let mut encoder = Encoder::from(buf);

        encoder.encode(Self::FIXED_HDR);
        encoder.encode(self.remaining_len());
        encoder.encode(self.packet_identifier);

        encoder.encode(self.property_len());

        for val in self.user_property.iter().copied() {
            encoder.encode(val);
        }

        for val in self.payload.iter().copied() {
            encoder.encode(val);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_bytes_0() {
        const EXPECTED: [u8; 15] = [
            UnsubscribeTx::FIXED_HDR,
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

        let mut builder = UnsubscribeTxBuilder::default();
        builder.packet_identifier(NonZero::try_from(13).unwrap());
        builder.payload(UTF8StringRef("a/b"));
        builder.payload(UTF8StringRef("c/d"));

        let packet = builder.build().unwrap();
        let mut buf = BytesMut::new();
        packet.encode(&mut buf);

        assert_eq!(&buf.split().freeze()[..], &EXPECTED);
    }
}
