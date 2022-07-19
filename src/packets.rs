use crate::{
    auth::*, connack::*, disconnect::*, pingresp::*, puback::*, pubcomp::*, publish::*, pubrec::*,
    pubrel::*, suback::*, unsuback::*, utils::TryFromBytes,
};

pub enum RxPacketVariant {
    Connack(Connack),
    Publish(Publish),
    Puback(Puback),
    Pubrec(Pubrec),
    Pubrel(Pubrel),
    Pubcomp(Pubcomp),
    Suback(Suback),
    Unsuback(Unsuback),
    Pingresp(Pingresp),
    Disconnect(Disconnect),
    Auth(Auth),
}

impl TryFromBytes for RxPacketVariant {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        match (bytes[0] >> 4) as isize {
            Connack::PACKET_ID => Connack::try_from_bytes(bytes).map(RxPacketVariant::Connack),
            Publish::PACKET_ID => Publish::try_from_bytes(bytes).map(RxPacketVariant::Publish),
            Puback::PACKET_ID => Puback::try_from_bytes(bytes).map(RxPacketVariant::Puback),
            Pubrec::PACKET_ID => Pubrec::try_from_bytes(bytes).map(RxPacketVariant::Pubrec),
            Pubrel::PACKET_ID => Pubrel::try_from_bytes(bytes).map(RxPacketVariant::Pubrel),
            Pubcomp::PACKET_ID => Pubcomp::try_from_bytes(bytes).map(RxPacketVariant::Pubcomp),
            Suback::PACKET_ID => Suback::try_from_bytes(bytes).map(RxPacketVariant::Suback),
            Unsuback::PACKET_ID => Unsuback::try_from_bytes(bytes).map(RxPacketVariant::Unsuback),
            Pingresp::PACKET_ID => Pingresp::try_from_bytes(bytes).map(RxPacketVariant::Pingresp),
            Disconnect::PACKET_ID => {
                Disconnect::try_from_bytes(bytes).map(RxPacketVariant::Disconnect)
            }
            Auth::PACKET_ID => Auth::try_from_bytes(bytes).map(RxPacketVariant::Auth),
            _ => None,
        }
    }
}
