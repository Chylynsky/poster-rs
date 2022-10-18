use crate::core::{
    error::{CodecError, InvalidPacketHeader},
    utils::{ByteReader, PacketID, TryFromBytes},
};

pub(crate) struct Pingresp {}

impl Pingresp {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;
}

#[derive(Default)]
struct PingrespBuilder {}

impl PingrespBuilder {
    fn build(self) -> Result<Pingresp, CodecError> {
        Ok(Pingresp {})
    }
}

impl PacketID for Pingresp {
    const PACKET_ID: u8 = 13;
}

impl TryFromBytes for Pingresp {
    type Error = CodecError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        let builder = PingrespBuilder::default();
        let mut reader = ByteReader::from(bytes);

        reader
            .try_read::<u8>()
            .map_err(CodecError::from)
            .and_then(|val| {
                if val != Self::FIXED_HDR {
                    return Err(InvalidPacketHeader.into());
                }

                return Ok(val);
            })?;

        builder.build()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes_0() {
        const FIXED_HDR: u8 = ((Pingresp::PACKET_ID as u8) << 4) as u8;
        const PACKET: [u8; 1] = [FIXED_HDR];
        let _ = Pingresp::try_from_bytes(&PACKET).unwrap();
    }
}
