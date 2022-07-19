use crate::utils::TryFromBytes;

pub struct Pingresp {}

#[derive(Default)]
pub struct PingrespPacketBuilder {}

impl PingrespPacketBuilder {
    fn build(self) -> Option<Pingresp> {
        Some(Pingresp {})
    }
}

impl Pingresp {
    pub const PACKET_ID: isize = 13;
}

impl TryFromBytes for Pingresp {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let packet_builder = PingrespPacketBuilder::default();

        let mut iter = bytes.iter().copied();

        let _fixed_hdr = iter.next()?;
        debug_assert!(_fixed_hdr >> 4 == Self::PACKET_ID as u8);

        packet_builder.build()
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
