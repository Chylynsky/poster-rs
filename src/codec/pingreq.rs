use crate::core::{
    error::CodecError,
    utils::{Encode, Encoder, PacketID, SizedPacket},
};
use bytes::BytesMut;
use core::mem;
use derive_builder::Builder;

#[derive(Builder)]
#[builder(build_fn(error = "CodecError"))]
pub(crate) struct PingreqTx {}

impl PingreqTx {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;
}

impl PacketID for PingreqTx {
    const PACKET_ID: u8 = 12;
}

impl SizedPacket for PingreqTx {
    fn packet_len(&self) -> usize {
        2 * mem::size_of::<u8>()
    }
}

impl Encode for PingreqTx {
    fn encode(&self, buf: &mut BytesMut) {
        let mut encoder = Encoder::from(buf);

        encoder.encode(Self::FIXED_HDR);
        encoder.encode(0u8);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_bytes_0() {
        const EXPECTED: [u8; 2] = [PingreqTx::PACKET_ID << 4, 0];

        let builder = PingreqTxBuilder::default();
        let packet = builder.build().unwrap();
        let mut buf = BytesMut::new();
        packet.encode(&mut buf);

        assert_eq!(&buf.split().freeze()[..], EXPECTED);
    }
}
