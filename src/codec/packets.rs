use bytes::{Bytes, BytesMut};

use crate::{
    codec::*,
    core::{
        error::{CodecError, InvalidPacketHeader},
        utils::{Encode, PacketID, SizedPacket, TryDecode},
    },
};

pub(crate) enum RxPacket {
    Connack(ConnackRx),
    Publish(PublishRx),
    Puback(PubackRx),
    Pubrec(PubrecRx),
    Pubrel(PubrelRx),
    Pubcomp(PubcompRx),
    Suback(SubackRx),
    Unsuback(UnsubackRx),
    Pingresp(PingrespRx),
    Disconnect(DisconnectRx),
    Auth(AuthRx),
}

impl TryDecode for RxPacket {
    type Error = CodecError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        match bytes[0] >> 4 {
            ConnackRx::PACKET_ID => ConnackRx::try_decode(bytes).map(RxPacket::Connack),
            PublishRx::PACKET_ID => PublishRx::try_decode(bytes).map(RxPacket::Publish),
            PubackRx::PACKET_ID => PubackRx::try_decode(bytes).map(RxPacket::Puback),
            PubrecRx::PACKET_ID => PubrecRx::try_decode(bytes).map(RxPacket::Pubrec),
            PubrelRx::PACKET_ID => PubrelRx::try_decode(bytes).map(RxPacket::Pubrel),
            PubcompRx::PACKET_ID => PubcompRx::try_decode(bytes).map(RxPacket::Pubcomp),
            SubackRx::PACKET_ID => SubackRx::try_decode(bytes).map(RxPacket::Suback),
            UnsubackRx::PACKET_ID => UnsubackRx::try_decode(bytes).map(RxPacket::Unsuback),
            PingrespRx::PACKET_ID => PingrespRx::try_decode(bytes).map(RxPacket::Pingresp),
            DisconnectRx::PACKET_ID => DisconnectRx::try_decode(bytes).map(RxPacket::Disconnect),
            AuthRx::PACKET_ID => AuthRx::try_decode(bytes).map(RxPacket::Auth),
            _ => Err(InvalidPacketHeader.into()),
        }
    }
}

pub(crate) enum TxPacket<'a> {
    Connect(ConnectTx<'a>),
    Publish(PublishTx<'a>),
    Puback(PubackTx<'a>),
    Pubrec(PubrecTx<'a>),
    Pubrel(PubrelTx<'a>),
    Pubcomp(PubcompTx<'a>),
    Subscribe(SubscribeTx<'a>),
    Unsubscribe(UnsubscribeTx<'a>),
    Pingreq(PingreqTx),
    Disconnect(DisconnectTx<'a>),
    Auth(AuthTx<'a>),
}

impl<'a> SizedPacket for TxPacket<'a> {
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

impl<'a> Encode for TxPacket<'a> {
    fn encode(&self, buf: &mut BytesMut) {
        match self {
            TxPacket::Connect(packet) => packet.encode(buf),
            TxPacket::Publish(packet) => packet.encode(buf),
            TxPacket::Puback(packet) => packet.encode(buf),
            TxPacket::Pubrec(packet) => packet.encode(buf),
            TxPacket::Pubrel(packet) => packet.encode(buf),
            TxPacket::Pubcomp(packet) => packet.encode(buf),
            TxPacket::Subscribe(packet) => packet.encode(buf),
            TxPacket::Unsubscribe(packet) => packet.encode(buf),
            TxPacket::Pingreq(packet) => packet.encode(buf),
            TxPacket::Disconnect(packet) => packet.encode(buf),
            TxPacket::Auth(packet) => packet.encode(buf),
        }
    }
}
