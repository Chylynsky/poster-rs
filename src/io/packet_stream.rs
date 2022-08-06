use crate::{
    codec::packets::RxPacket,
    core::base_types::VarSizeInt,
    core::utils::{ByteReader, TryFromBytes},
};
use futures::{AsyncBufRead, Stream};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

enum PacketStreamState {
    ReadRemainingLength,
    ReadRemainingData(usize),
}

pub(crate) struct PacketStream<StreamT> {
    state: PacketStreamState,
    stream: StreamT,
}

impl<'a, StreamT> From<StreamT> for PacketStream<StreamT> {
    fn from(stream: StreamT) -> Self {
        Self {
            state: PacketStreamState::ReadRemainingLength,
            stream,
        }
    }
}

impl<'a, StreamT> PacketStream<StreamT> {
    fn split_borrows_mut(&mut self) -> (&mut PacketStreamState, &mut StreamT) {
        (&mut self.state, &mut self.stream)
    }
}

impl<StreamT> Stream for PacketStream<StreamT>
where
    StreamT: AsyncBufRead + Unpin,
{
    type Item = RxPacket;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let (state, mut stream) = self.split_borrows_mut();
        if let Poll::Ready(Ok(buf)) = Pin::new(&mut stream).poll_fill_buf(cx) {
            if buf.is_empty() {
                return Poll::Ready(None); // EOF
            }

            match state {
                PacketStreamState::ReadRemainingLength => {
                    let mut reader = ByteReader::from(&buf[1..]);
                    if let Some(remaining_len) = reader.try_read::<VarSizeInt>() {
                        *state = PacketStreamState::ReadRemainingData(remaining_len.into());
                        return self.poll_next(cx);
                    }
                }
                PacketStreamState::ReadRemainingData(remaining_len) => {
                    if buf.len() >= *remaining_len {
                        let result = RxPacket::try_from_bytes(buf);
                        Pin::new(&mut stream).consume(*remaining_len); // Consume the packet
                        *state = PacketStreamState::ReadRemainingLength;
                        return Poll::Ready(result);
                    }
                }
            }
        }

        cx.waker().wake_by_ref();
        Poll::Pending
    }
}
