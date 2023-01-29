use crate::{
    codec::RxPacket,
    core::{
        base_types::VarSizeInt,
        error::{CodecError, ConversionError},
        utils::TryDecode,
    },
};
use bytes::BytesMut;
use core::{
    ops::Range,
    pin::Pin,
    task::{Context, Poll},
};
use futures::{AsyncRead, AsyncWrite, AsyncWriteExt, Stream};
use std::{io, mem};

enum PacketStreamState {
    Idle,
    ReadPacketLen,
    ReadPacketData,
}

pub(crate) struct RxPacketStream<StreamT> {
    stream: StreamT,
    buf: BytesMut,
    size: usize,

    packet: Range<usize>,

    state: PacketStreamState,
}

impl<StreamT> From<StreamT> for RxPacketStream<StreamT> {
    fn from(stream: StreamT) -> Self {
        Self {
            stream,
            buf: BytesMut::with_capacity(1024),
            size: 0,
            packet: 0..0,
            state: PacketStreamState::Idle,
        }
    }
}

impl<StreamT> RxPacketStream<StreamT> {
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
            &mut self.size,
            &mut self.packet,
            &mut self.state,
        )
    }
}

impl<StreamT> Stream for RxPacketStream<StreamT>
where
    StreamT: AsyncRead + Unpin,
{
    type Item = Result<RxPacket, CodecError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        const DEFAULT_CHUNK_SIZE: usize = 512;

        let (mut stream, buf, size, packet, state) = self.split_borrows_mut();

        match *state {
            PacketStreamState::Idle => {
                let chunk_size = if packet.end - *size < DEFAULT_CHUNK_SIZE {
                    DEFAULT_CHUNK_SIZE
                } else {
                    packet.end
                };

                buf.resize(*size + chunk_size, 0);

                if let Poll::Ready(result) = Pin::new(&mut stream)
                    .poll_read(cx, &mut buf[*size..*size + chunk_size])
                    .map(|res| res.ok().filter(|&size| size != 0 /* EOF */))
                {
                    if result.is_none() {
                        return Poll::Ready(None);
                    }

                    *size += result.unwrap();

                    // We need to be able to read at least fixed header and one byte of size to proceed.
                    if *size >= 2 {
                        *state = PacketStreamState::ReadPacketLen;
                        return self.poll_next(cx);
                    }
                }

                Poll::Pending
            }
            PacketStreamState::ReadPacketLen => {
                // Omit packet ID, try to read the remaining length.
                let maybe_remaining_len =
                    VarSizeInt::try_from(&buf[1..]).map(Some).or_else(|err| {
                        if let ConversionError::InsufficientBufferSize(_) = err {
                            return Ok(None); // Need to read more data
                        }
                        Err(err)
                    });

                if maybe_remaining_len.is_err() {
                    return Poll::Ready(None);
                }

                if let Some(remaining_len) = maybe_remaining_len.unwrap() {
                    // Fixed header (1 byte), size of Variable Byte Integer
                    // encoding the remaining length and its value.
                    packet.start = 0;
                    packet.end = 1 + remaining_len.len() + remaining_len.value() as usize;
                    *state = PacketStreamState::ReadPacketData;
                    return self.poll_next(cx);
                }

                *state = PacketStreamState::Idle;
                self.poll_next(cx)
            }
            PacketStreamState::ReadPacketData => {
                if *size < packet.end {
                    *state = PacketStreamState::Idle;
                    return self.poll_next(cx);
                }

                *size -= packet.len();
                if *size != 0 {
                    *state = PacketStreamState::ReadPacketLen;
                } else {
                    *state = PacketStreamState::Idle;
                }

                Poll::Ready(Some(RxPacket::try_decode(
                    buf.split_to(mem::replace(&mut packet.end, 0)).freeze(),
                )))
            }
        }
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
