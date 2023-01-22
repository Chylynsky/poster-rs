use crate::codec::RxPacket;
use bytes::BytesMut;
use futures::channel::{mpsc, oneshot};

use super::error::MqttError;

pub(crate) struct FireAndForget {
    pub(crate) packet: BytesMut,
    pub(crate) response_channel: oneshot::Sender<Result<(), MqttError>>,
}

pub(crate) struct AwaitAck {
    pub(crate) action_id: usize,
    pub(crate) packet: BytesMut,
    pub(crate) response_channel: oneshot::Sender<Result<RxPacket, MqttError>>,
}

pub(crate) struct Subscribe {
    pub(crate) action_id: usize,
    pub(crate) subscription_identifier: usize,
    pub(crate) packet: BytesMut,
    pub(crate) response_channel: oneshot::Sender<Result<RxPacket, MqttError>>,
    pub(crate) stream: mpsc::UnboundedSender<RxPacket>,
}

pub(crate) enum ContextMessage {
    FireAndForget(FireAndForget),
    AwaitAck(AwaitAck),
    Subscribe(Subscribe),
}
