use crate::codec::RxPacket;
use bytes::BytesMut;
use futures::channel::{mpsc, oneshot};

pub(crate) struct Connect {
    pub(crate) action_id: usize,
    pub(crate) packet: BytesMut,
    pub(crate) session_expiry_interval: u32,
    pub(crate) response_channel: oneshot::Sender<RxPacket>,
}

pub(crate) struct AwaitAck {
    pub(crate) action_id: usize,
    pub(crate) packet: BytesMut,
    pub(crate) response_channel: oneshot::Sender<RxPacket>,
}

pub(crate) struct Subscribe {
    pub(crate) action_id: usize,
    pub(crate) subscription_identifier: usize,
    pub(crate) packet: BytesMut,
    pub(crate) response_channel: oneshot::Sender<RxPacket>,
    pub(crate) stream: mpsc::UnboundedSender<RxPacket>,
}

pub(crate) enum ContextMessage {
    Connect(Connect),
    FireAndForget(BytesMut),
    AwaitAck(AwaitAck),
    Disconnect(BytesMut),
    Subscribe(Subscribe),
}
