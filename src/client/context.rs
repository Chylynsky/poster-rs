use crate::{
    codec::connect::Connect,
    core::utils::{SizedPacket, TryToByteBuffer},
    io::packet_stream::PacketStream,
};
use futures::{
    io::BufReader, AsyncBufRead, AsyncRead, AsyncWrite, AsyncWriteExt, Stream, StreamExt,
};

pub struct Context<'a, 'b, RxStreamT, TxStreamT> {
    rx: PacketStream<'a, RxStreamT>,
    tx: &'b mut TxStreamT,
    buf: Vec<u8>,
}

impl<'a, 'b, RxStreamT, TxStreamT> Context<'a, 'b, RxStreamT, TxStreamT>
where
    RxStreamT: AsyncBufRead + Unpin,
    TxStreamT: AsyncWrite + Unpin,
{
    pub fn from(rx: &'a mut RxStreamT, tx: &'b mut TxStreamT) -> Self {
        Self {
            rx: PacketStream::from(rx),
            tx,
            buf: Vec::with_capacity(2048),
        }
    }

    async fn packet_write(&mut self, packet: Connect) -> Option<usize> {
        self.buf.resize(packet.packet_len(), 0u8);
        packet.try_to_byte_buffer(&mut self.buf);
        self.tx.write(&self.buf).await.ok()
    }
}
