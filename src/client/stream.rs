use crate::{client::rsp::PublishData, codec::RxPacket};
use futures::{
    channel::mpsc::{self},
    Stream, StreamExt,
};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub struct SubscribeStream {
    pub(crate) receiver: mpsc::UnboundedReceiver<RxPacket>,
}

impl Stream for SubscribeStream {
    type Item = PublishData;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.receiver.poll_next_unpin(cx) {
            Poll::Ready(rx_packet) => {
                if let Some(RxPacket::Publish(publish)) = rx_packet {
                    return Poll::Ready(Some(PublishData::from(publish)));
                }

                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
