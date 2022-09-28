use crate::{
    codec::RxPacket,
    core::base_types::VarSizeInt,
    core::utils::{ByteReader, TryFromBytes},
};
use futures::{AsyncBufRead, Stream};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

enum PacketStreamState {
    ReadPacketSize,
    ReadPacket(usize),
}

pub(crate) struct PacketStream<StreamT> {
    state: PacketStreamState,
    stream: StreamT,
}

impl<'a, StreamT> From<StreamT> for PacketStream<StreamT> {
    fn from(stream: StreamT) -> Self {
        Self {
            state: PacketStreamState::ReadPacketSize,
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
                PacketStreamState::ReadPacketSize => {
                    let mut reader = ByteReader::from(&buf[1..]);
                    if let Some(remaining_len) = reader.try_read::<VarSizeInt>() {
                        *state = PacketStreamState::ReadPacket(
                            1 + remaining_len.len() + remaining_len.value() as usize,
                        );
                        return self.poll_next(cx);
                    }
                }
                PacketStreamState::ReadPacket(packet_size) => {
                    if buf.len() >= *packet_size {
                        let result = RxPacket::try_from_bytes(buf);

                        println!("[RX] Got packet: {:?}", &buf);

                        Pin::new(&mut stream).consume(*packet_size); // Consume the packet
                        *state = PacketStreamState::ReadPacketSize;
                        return Poll::Ready(result);
                    }
                }
            }
        }

        Poll::Pending
    }
}
