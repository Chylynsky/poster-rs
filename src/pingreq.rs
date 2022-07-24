use crate::{
    base_types::Byte,
    utils::{ByteWriter, PacketID, SizedPacket, TryToByteBuffer},
};
use std::mem;

pub(crate) struct Pingreq {}

impl PacketID for Pingreq {
    const PACKET_ID: u8 = 12;
}

impl SizedPacket for Pingreq {
    fn packet_len(&self) -> usize {
        2 * mem::size_of::<Byte>()
    }
}

impl TryToByteBuffer for Pingreq {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let packet_len = self.packet_len();

        if packet_len > buf.len() {
            return None;
        }

        let result = &mut buf[0..packet_len];
        let mut writer = ByteWriter::from(result);

        writer.write(&(Self::PACKET_ID << 4));
        writer.write(&0u8);

        Some(result)
    }
}

#[derive(Default)]
pub(crate) struct PingreqPacketBuilder {}

impl PingreqPacketBuilder {
    pub(crate) fn build(&self) -> Option<Pingreq> {
        Some(Pingreq {})
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_bytes() {
        const EXPECTED: [u8; 2] = [Pingreq::PACKET_ID << 4, 0];

        let builder = PingreqPacketBuilder::default();
        let packet = builder.build().unwrap();
        let mut buf = [0u8; 2];
        let result = packet.try_to_byte_buffer(&mut buf).unwrap();

        assert_eq!(result, EXPECTED);
    }
}
