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
    const DEFAULT_BUF_SIZE: usize = 1024;

    pub(crate) fn new(inner: TxStreamT) -> Self {
        Self {
            stream: inner,
            buf: Vec::with_capacity(Self::DEFAULT_BUF_SIZE),
        }
    }

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
    {
        self.buf.resize(packet.packet_len(), 0u8);
        let raw = packet.try_to_byte_buffer(&mut self.buf).unwrap();

        println!("[TX] Sending packet: {:?}", &raw);

        self.stream.write(raw).await
    }
}
