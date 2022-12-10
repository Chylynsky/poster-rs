use crate::{
    codec::RxPacket,
    core::{
        base_types::VarSizeInt,
        error::{CodecError, ConversionError},
        utils::TryDecode,
    },
};
use bytes::{Bytes, BytesMut};
use core::{
    ops::Range,
    pin::Pin,
    task::{Context, Poll},
};
use futures::{AsyncRead, AsyncWrite, AsyncWriteExt, Stream};
use std::io;

enum PacketStreamState {
    Idle,
    ReadPacketLen,
    ReadPacketData,
}

pub(crate) struct RxPacketStream<StreamT> {
    stream: StreamT,
    buf: BytesMut,
    offset: usize,

    packet: Range<usize>,

    state: PacketStreamState,
}

impl<StreamT> From<StreamT> for RxPacketStream<StreamT> {
    fn from(stream: StreamT) -> Self {
        Self {
            stream,
            buf: BytesMut::with_capacity(2048),
            offset: 0,
            packet: 0..0,
            state: PacketStreamState::Idle,
        }
    }
}

impl<StreamT> RxPacketStream<StreamT> {
    pub(crate) fn with_capacity(capacity: usize, inner: StreamT) -> Self {
        Self {
            stream: inner,
            buf: BytesMut::with_capacity(capacity),
            offset: 0,
            packet: 0..0,
            state: PacketStreamState::Idle,
        }
    }

    fn split_borrows_mut(
        &mut self,
    ) -> (
        &mut StreamT,
        &mut BytesMut,
        &mut usize,
        &mut Range<usize>,
        &mut PacketStreamState,
    ) {
        (
            &mut self.stream,
            &mut self.buf,
            &mut self.offset,
            &mut self.packet,
            &mut self.state,
        )
    }

    fn step(&mut self, size: usize) -> Option<Result<Bytes, CodecError>> {
        match self.state {
            PacketStreamState::Idle => {
                self.offset = size;
                self.state = PacketStreamState::ReadPacketLen;
                self.step(0) // Size is already consumed for setting the offset
            }
            PacketStreamState::ReadPacketLen => {
                self.offset += size;

                // We need a packet ID and at least one byte encoding the remaiing length.
                if self.offset < 2 {
                    return None;
                }

                // Omit packet ID, try to read the remaining length.
                let maybe_remaining_len = VarSizeInt::try_from(&self.buf[1..self.offset])
                    .map(Some)
                    .or_else(|err| {
                        if let ConversionError::InsufficientBufferSize(_) = err {
                            return Ok(None); // Need to read more data
                        }
                        Err(err)
                    });

                if let Err(err) = maybe_remaining_len {
                    return Some(Err(err.into()));
                }

                if let Some(remaining_len) = maybe_remaining_len.unwrap() {
                    // Packet ID (1 byte), size of Variable Byte Integer
                    // encoding the remaining length and its value.
                    self.packet.end = 1 + remaining_len.len() + remaining_len.value() as usize;
                    self.state = PacketStreamState::ReadPacketData;
                    return self.step(0);
                }

                None
            }
            PacketStreamState::ReadPacketData => {
                self.offset += size;

                if self.offset < self.packet.end {
                    return None;
                }

                self.offset = 0;
                self.state = PacketStreamState::Idle;
                Some(Ok(self.buf.split_to(self.packet.end).freeze()))
            }
        }
    }
}

impl<StreamT> Stream for RxPacketStream<StreamT>
where
    StreamT: AsyncRead + Unpin,
{
    type Item = Result<RxPacket, CodecError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        const CHUNK_SIZE: usize = 2048;

        let (mut stream, buf, offset, _, _) = self.split_borrows_mut();
        buf.resize(*offset + CHUNK_SIZE, 0);

        if let Poll::Ready(result) =
            Pin::new(&mut stream).poll_read(cx, &mut buf[*offset..*offset + CHUNK_SIZE])
        {
            if result.is_err() {
                return Poll::Ready(None);
            }

            let size = result.unwrap();
            if size == 0 {
                return Poll::Ready(None); // EOF
            }

            if let Some(packet) = self
                .step(size)
                .map(|maybe_buf| maybe_buf.and_then(RxPacket::try_decode))
            {
                cx.waker().wake_by_ref();
                return Poll::Ready(Some(packet));
            }

            cx.waker().wake_by_ref();
        }

        Poll::Pending
    }
}

pub(crate) struct TxPacketStream<TxStreamT> {
    stream: TxStreamT,
}

impl<TxStreamT> From<TxStreamT> for TxPacketStream<TxStreamT> {
    fn from(inner: TxStreamT) -> Self {
        Self { stream: inner }
    }
}

impl<TxStreamT> TxPacketStream<TxStreamT> {
    pub(crate) async fn write(&mut self, packet: &[u8]) -> Result<usize, io::Error>
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
