use crate::{client::rsp::PublishData, codec::RxPacket};
use core::pin::Pin;
use futures::{
    channel::mpsc,
    task::{self, Poll},
    Stream, StreamExt,
};

pub struct SubscribeStream {
    pub(crate) receiver: mpsc::UnboundedReceiver<RxPacket>,
}

impl Stream for SubscribeStream {
    type Item = PublishData;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self).receiver.poll_next_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(packet) => {
                if let Some(RxPacket::Publish(publish)) = packet {
                    return Poll::Ready(Some(PublishData::from(publish)));
                }

                return Poll::Ready(None);
            }
        }
    }
}
