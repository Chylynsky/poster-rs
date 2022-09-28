use crate::{
    client::rsp::{AuthRsp, ConnectRsp, SubscribeRsp},
    codec::RxPacket,
};
use either::Either;
use futures::{
    channel::{mpsc, oneshot},
    task::{self, Poll},
    Future, Stream,
};
use std::pin::Pin;
use Either::{Left, Right};

pub struct ConnectFuture {
    pub(crate) receiver: oneshot::Receiver<RxPacket>,
}

impl Future for ConnectFuture {
    type Output = Option<Either<ConnectRsp, AuthRsp>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        if let Poll::Ready(packet_result) = Pin::new(&mut self.receiver).poll(cx) {
            if packet_result.is_err() {
                return Poll::Ready(None); // Channel cancelled.
            }

            match packet_result.unwrap() {
                RxPacket::Connack(connack) => {
                    return Poll::Ready(Some(Left(ConnectRsp::from(connack))));
                }
                RxPacket::Auth(auth) => {
                    return Poll::Ready(Some(Right(AuthRsp::from(auth))));
                }
                _ => {
                    panic!("Unexpected packet type.");
                }
            }
        }

        Poll::Pending
    }
}

pub struct PingFuture {
    pub(crate) receiver: oneshot::Receiver<RxPacket>,
}

impl Future for PingFuture {
    type Output = Option<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        if let Poll::Ready(packet_result) = Pin::new(&mut self.receiver).poll(cx) {
            if let Ok(RxPacket::Pingresp(pingresp)) = packet_result {
                return Poll::Ready(Some(()));
            }

            return Poll::Ready(None); // Channel cancelled.
        }

        Poll::Pending
    }
}

pub struct SubscribeFuture {
    pub(crate) receiver: oneshot::Receiver<RxPacket>,
}

impl Future for SubscribeFuture {
    type Output = Option<(SubscribeRsp, SubscribeStream)>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        if let Poll::Ready(packet_result) = Pin::new(&mut self.receiver).poll(cx) {
            if let Ok(RxPacket::Suback(suback)) = packet_result {
                // return Poll::Ready(Some((SubscribeRsp::from(suback), SubscribeStream {})));
            }

            return Poll::Ready(None);
        }

        Poll::Pending
    }
}

pub struct SubscribeStream {
    pub(crate) receiver: mpsc::UnboundedReceiver<RxPacket>,
}

impl Stream for SubscribeStream {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self).poll_next(cx)
    }
}

pub struct UnsubscribeFuture {
    pub(crate) receiver: oneshot::Receiver<RxPacket>,
}

impl Future for UnsubscribeFuture {
    type Output = Option<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        if let Poll::Ready(packet_result) = Pin::new(&mut self.receiver).poll(cx) {
            if let Ok(RxPacket::Unsuback(unsuback)) = packet_result {
                return Poll::Ready(None); // TODO
            }

            return Poll::Ready(None); // Channel cancelled.
        }

        Poll::Pending
    }
}
