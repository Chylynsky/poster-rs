use crate::{
    core::utils::{SizedPacket, TryToByteBuffer},
    io::PacketStream,
};
use futures::{io::BufReader, AsyncWrite, AsyncWriteExt};
use std::io;

pub(crate) type RxPacketStream<RxStreamT> = PacketStream<BufReader<RxStreamT>>;

pub(crate) struct TxPacketStream<TxStreamT> {
    stream: TxStreamT,
    buf: Vec<u8>,
}

impl<TxStreamT> TxPacketStream<TxStreamT> {
    pub(crate) fn with_capacity(capacity: usize, inner: TxStreamT) -> Self {
        Self {
            stream: inner,
            buf: Vec::with_capacity(capacity),
        }
    }

    pub(crate) async fn write<PacketT>(&mut self, packet: PacketT) -> Result<usize, io::Error>
    where
        TxStreamT: AsyncWrite + Unpin,
        PacketT: SizedPacket + TryToByteBuffer,
        <PacketT as TryToByteBuffer>::Error: core::fmt::Debug,
    {
        let packet_len = packet.packet_len();

        self.buf.resize(packet_len, 0u8);
        let raw = packet.try_to_byte_buffer(&mut self.buf).unwrap();

        let mut remaining = packet_len;
        while remaining != 0 {
            remaining -= self.stream.write(&raw[(raw.len() - remaining)..]).await?;
        }

        Ok(packet_len)
    }
}
