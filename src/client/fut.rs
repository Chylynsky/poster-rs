use crate::codec::RxPacket;
use futures::{
    channel::mpsc,
    task::{self, Poll},
    Stream,
};
use std::pin::Pin;

pub struct SubscribeStream {
    pub(crate) receiver: mpsc::UnboundedReceiver<RxPacket>,
}

impl Stream for SubscribeStream {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self).poll_next(cx)
    }
}
