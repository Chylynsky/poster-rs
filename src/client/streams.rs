use crate::{core::utils::SizedPacket, io::PacketStream};
use bytes::Bytes;
use futures::{AsyncWrite, AsyncWriteExt};
use std::io;

pub(crate) type RxPacketStream<RxStreamT> = PacketStream<RxStreamT>;

pub(crate) struct TxPacketStream<TxStreamT> {
    stream: TxStreamT,
}

impl<TxStreamT> From<TxStreamT> for TxPacketStream<TxStreamT> {
   fn from(inner: TxStreamT) -> Self {
        Self {
            stream: inner,
        }
    }
}

impl<TxStreamT> TxPacketStream<TxStreamT> {
    pub(crate) async fn write(&mut self, packet: Bytes) -> Result<usize, io::Error>
    where
        TxStreamT: AsyncWrite + Unpin,
    {
        let mut remaining = packet.len();
        while remaining != 0 {
            remaining -= self
                .stream
                .write(&packet[(packet.len() - remaining)..])
                .await?;
        }

        Ok(packet.len())
    }
}
