use crate::core::{
    base_types::Byte,
    utils::{ByteReader, PacketID, TryFromBytes},
};

pub(crate) struct Pingresp {}

impl Pingresp {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;
}

#[derive(Default)]
pub(crate) struct PingrespPacketBuilder {}

impl PingrespPacketBuilder {
    fn build(self) -> Option<Pingresp> {
        Some(Pingresp {})
    }
}

impl PacketID for Pingresp {
    const PACKET_ID: u8 = 13;
}

impl TryFromBytes for Pingresp {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let builder = PingrespPacketBuilder::default();
        let mut reader = ByteReader::from(bytes);
        let _fixed_hdr = reader.try_read::<Byte>()?;
        debug_assert!(_fixed_hdr >> 4 == Self::PACKET_ID as u8);

        builder.build()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes() {
        const FIXED_HDR: u8 = ((Pingresp::PACKET_ID as u8) << 4) as u8;
        const PACKET: [u8; 1] = [FIXED_HDR];
        let _ = Pingresp::try_from_bytes(&PACKET).unwrap();
    }
}
