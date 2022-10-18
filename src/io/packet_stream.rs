use crate::{
    codec::RxPacket,
    core::utils::{ByteReader, TryFromBytes},
    core::{base_types::VarSizeInt, error::CodecError},
};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures::{AsyncBufRead, Stream};

enum PacketStreamState {
    ReadPacketSize,
    ReadPacket(usize),
}

pub(crate) struct PacketStream<StreamT> {
    state: PacketStreamState,
    stream: StreamT,
}

impl<StreamT> From<StreamT> for PacketStream<StreamT> {
    fn from(stream: StreamT) -> Self {
        Self {
            state: PacketStreamState::ReadPacketSize,
            stream,
        }
    }
}

impl<StreamT> PacketStream<StreamT> {
    fn split_borrows_mut(&mut self) -> (&mut PacketStreamState, &mut StreamT) {
        (&mut self.state, &mut self.stream)
    }
}

impl<StreamT> Stream for PacketStream<StreamT>
where
    StreamT: AsyncBufRead + Unpin,
{
    type Item = Result<RxPacket, CodecError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let (state, mut stream) = self.split_borrows_mut();

        if let Poll::Ready(result) = Pin::new(&mut stream).poll_fill_buf(cx) {
            if result.is_err() {
                return Poll::Ready(None);
            }

            let buf = result.unwrap();
            if buf.is_empty() {
                return Poll::Ready(None); // EOF
            }

            match state {
                PacketStreamState::ReadPacketSize => {
                    let mut reader = ByteReader::from(&buf[1..]);
                    if let Ok(remaining_len) = reader.try_read::<VarSizeInt>() {
                        let packet_size = 1 + remaining_len.len() + remaining_len.value() as usize;
                        *state = PacketStreamState::ReadPacket(packet_size);
                    }

                    return self.poll_next(cx);
                }
                PacketStreamState::ReadPacket(packet_size) => {
                    if buf.len() >= *packet_size {
                        let result = RxPacket::try_from_bytes(buf);

                        println!("[RX] Got packet: {:?}", &buf);

                        Pin::new(&mut stream).consume(*packet_size); // Consume the packet
                        *state = PacketStreamState::ReadPacketSize;

                        cx.waker().wake_by_ref();
                        return Poll::Ready(Some(result));
                    }
                }
            }
        }

        Poll::Pending
    }
}
