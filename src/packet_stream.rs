use crate::{
    base_types::VarSizeInt,
    packets::RxPacketVariant,
    utils::{TryFromBytes, TryFromIterator},
};
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::AsyncBufRead;
use tokio_stream::Stream;

enum PacketStreamState {
    ReadRemainingLength,
    ReadRemainingData(usize),
}

pub struct PacketStream<'a, StreamT> {
    state: PacketStreamState,
    stream: &'a mut StreamT,
}

impl<'a, StreamT> From<&'a mut StreamT> for PacketStream<'a, StreamT> {
    fn from(stream: &'a mut StreamT) -> Self {
        Self {
            state: PacketStreamState::ReadRemainingLength,
            stream,
        }
    }
}

impl<'a, StreamT> PacketStream<'a, StreamT> {
    fn split_borrows_mut(&mut self) -> (&mut PacketStreamState, &mut StreamT) {
        (&mut self.state, &mut self.stream)
    }
}

impl<'a, StreamT> Stream for PacketStream<'a, StreamT>
where
    StreamT: AsyncBufRead + Unpin,
{
    type Item = RxPacketVariant;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let (state, stream) = self.split_borrows_mut();
        if let Poll::Ready(Ok(buf)) = Pin::new(&mut *stream).poll_fill_buf(cx) {
            if buf.is_empty() {
                return Poll::Ready(None); // EOF
            }

            match state {
                PacketStreamState::ReadRemainingLength => {
                    let iter = buf.iter().skip(1); // Ignore the fixed header at this point.
                    if let Some(remaining_len) = VarSizeInt::try_from_iter(iter.copied()) {
                        Pin::new(&mut *stream).consume(remaining_len.len());
                        *state = PacketStreamState::ReadRemainingData(remaining_len.into());
                        return self.poll_next(cx);
                    }
                }
                PacketStreamState::ReadRemainingData(remaining_len) => {
                    if buf.len() >= *remaining_len {
                        let result = RxPacketVariant::try_from_bytes(&buf[0..*remaining_len]); // Take slice to the entire packet
                        Pin::new(&mut *stream).consume(*remaining_len); // Consume the packet
                        *state = PacketStreamState::ReadRemainingLength;
                        return Poll::Ready(result);
                    }
                }
            }
        }

        cx.waker().clone().wake();
        Poll::Pending
    }
}
