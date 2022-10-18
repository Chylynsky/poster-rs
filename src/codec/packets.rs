use crate::{
    codec::{
        auth::Auth, connack::Connack, connect::Connect, disconnect::Disconnect, pingreq::Pingreq,
        pingresp::Pingresp, puback::Puback, pubcomp::Pubcomp, publish::Publish, pubrec::Pubrec,
        pubrel::Pubrel, suback::Suback, subscribe::Subscribe, unsuback::Unsuback,
        unsubscribe::Unsubscribe,
    },
    core::{
        error::{CodecError, InvalidPacketHeader},
        utils::{PacketID, SizedPacket, TryFromBytes, TryToByteBuffer},
    },
};

pub(crate) enum RxPacket {
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

impl TryFromBytes for RxPacket {
    type Error = CodecError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        match bytes[0] >> 4 {
            Connack::PACKET_ID => Connack::try_from_bytes(bytes).map(RxPacket::Connack),
            Publish::PACKET_ID => Publish::try_from_bytes(bytes).map(RxPacket::Publish),
            Puback::PACKET_ID => Puback::try_from_bytes(bytes).map(RxPacket::Puback),
            Pubrec::PACKET_ID => Pubrec::try_from_bytes(bytes).map(RxPacket::Pubrec),
            Pubrel::PACKET_ID => Pubrel::try_from_bytes(bytes).map(RxPacket::Pubrel),
            Pubcomp::PACKET_ID => Pubcomp::try_from_bytes(bytes).map(RxPacket::Pubcomp),
            Suback::PACKET_ID => Suback::try_from_bytes(bytes).map(RxPacket::Suback),
            Unsuback::PACKET_ID => Unsuback::try_from_bytes(bytes).map(RxPacket::Unsuback),
            Pingresp::PACKET_ID => Pingresp::try_from_bytes(bytes).map(RxPacket::Pingresp),
            Disconnect::PACKET_ID => Disconnect::try_from_bytes(bytes).map(RxPacket::Disconnect),
            Auth::PACKET_ID => Auth::try_from_bytes(bytes).map(RxPacket::Auth),
            _ => Err(InvalidPacketHeader.into()),
        }
    }
}

pub(crate) enum TxPacket {
    Connect(Connect),
    Publish(Publish),
    Puback(Puback),
    Pubrec(Pubrec),
    Pubrel(Pubrel),
    Pubcomp(Pubcomp),
    Subscribe(Subscribe),
    Unsubscribe(Unsubscribe),
    Pingreq(Pingreq),
    Disconnect(Disconnect),
    Auth(Auth),
}

impl SizedPacket for TxPacket {
    fn packet_len(&self) -> usize {
        match self {
            TxPacket::Connect(packet) => packet.packet_len(),
            TxPacket::Publish(packet) => packet.packet_len(),
            TxPacket::Puback(packet) => packet.packet_len(),
            TxPacket::Pubrec(packet) => packet.packet_len(),
            TxPacket::Pubrel(packet) => packet.packet_len(),
            TxPacket::Pubcomp(packet) => packet.packet_len(),
            TxPacket::Subscribe(packet) => packet.packet_len(),
            TxPacket::Unsubscribe(packet) => packet.packet_len(),
            TxPacket::Pingreq(packet) => packet.packet_len(),
            TxPacket::Disconnect(packet) => packet.packet_len(),
            TxPacket::Auth(packet) => packet.packet_len(),
        }
    }
}

impl TryToByteBuffer for TxPacket {
    type Error = CodecError;

    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Result<&'a [u8], Self::Error> {
        match self {
            TxPacket::Connect(packet) => packet.try_to_byte_buffer(buf),
            TxPacket::Publish(packet) => packet.try_to_byte_buffer(buf),
            TxPacket::Puback(packet) => packet.try_to_byte_buffer(buf),
            TxPacket::Pubrec(packet) => packet.try_to_byte_buffer(buf),
            TxPacket::Pubrel(packet) => packet.try_to_byte_buffer(buf),
            TxPacket::Pubcomp(packet) => packet.try_to_byte_buffer(buf),
            TxPacket::Subscribe(packet) => packet.try_to_byte_buffer(buf),
            TxPacket::Unsubscribe(packet) => packet.try_to_byte_buffer(buf),
            TxPacket::Pingreq(packet) => packet.try_to_byte_buffer(buf),
            TxPacket::Disconnect(packet) => packet.try_to_byte_buffer(buf),
            TxPacket::Auth(packet) => packet.try_to_byte_buffer(buf),
        }
    }
}
