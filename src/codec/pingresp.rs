use crate::core::{
    error::{CodecError, InvalidPacketHeader},
    utils::{Decoder, PacketID, TryDecode},
};
use bytes::Bytes;
use derive_builder::Builder;

#[derive(Builder)]
#[builder(build_fn(error = "CodecError"))]
pub(crate) struct PingrespRx {}

impl PingrespRx {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;
}

impl PacketID for PingrespRx {
    const PACKET_ID: u8 = 13;
}

impl TryDecode for PingrespRx {
    type Error = CodecError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        let builder = PingrespRxBuilder::default();
        let mut reader = Decoder::from(bytes);

        reader
            .try_decode::<u8>()
            .map_err(CodecError::from)
            .and_then(|val| {
                if val != Self::FIXED_HDR {
                    return Err(InvalidPacketHeader.into());
                }

                Ok(val)
            })?;

        builder.build()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes_0() {
        const FIXED_HDR: u8 = PingrespRx::PACKET_ID << 4;
        const PACKET: [u8; 1] = [FIXED_HDR];
        let _ = PingrespRx::try_decode(Bytes::from_static(&PACKET)).unwrap();
    }
}
