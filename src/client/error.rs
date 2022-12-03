use crate::{
    client::rsp::{PublishError, SubscribeError, UnsubscribeError},
    codec::DisconnectRx,
    core::error::CodecError,
    DisconnectReason,
};
use futures::channel::{
    mpsc::{SendError, TrySendError},
    oneshot::Canceled,
};
use std::{error::Error, fmt, io};

#[derive(Debug, Clone)]
pub struct SocketClosed;

impl fmt::Display for SocketClosed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "socket closed")
    }
}

impl Error for SocketClosed {}

impl From<io::Error> for SocketClosed {
    fn from(_: io::Error) -> Self {
        Self
    }
}

#[derive(Debug, Clone)]
pub struct HandleClosed;

impl fmt::Display for HandleClosed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "context handle closed")
    }
}

impl Error for HandleClosed {}

impl From<Canceled> for HandleClosed {
    fn from(_: Canceled) -> Self {
        Self
    }
}

impl From<SendError> for HandleClosed {
    fn from(_: SendError) -> Self {
        Self
    }
}

#[derive(Debug, Clone)]
pub struct ContextExited;

impl fmt::Display for ContextExited {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "context exited")
    }
}

impl Error for ContextExited {}

impl<T> From<TrySendError<T>> for ContextExited {
    fn from(_: TrySendError<T>) -> Self {
        Self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Disconnected {
    pub reason: DisconnectReason,
}

impl fmt::Display for Disconnected {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "disconnected with reason: {} [{:?}]",
            self.reason as u8, self.reason
        )
    }
}

impl Error for Disconnected {}

impl From<DisconnectRx> for Disconnected {
    fn from(packet: DisconnectRx) -> Self {
        Self {
            reason: packet.reason,
        }
    }
}

impl From<DisconnectReason> for Disconnected {
    fn from(reason: DisconnectReason) -> Self {
        Self { reason }
    }
}

#[derive(Debug, Clone)]
pub struct InternalError {
    msg: String,
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ \"type\": \"InternalError\", \"message\": \"{}\" }}",
            self.msg
        )
    }
}

impl Error for InternalError {}

impl From<&str> for InternalError {
    fn from(s: &str) -> Self {
        Self {
            msg: String::from(s),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct QuotaExceeded;

impl fmt::Display for QuotaExceeded {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ \"type\": \"QuotaExceeded\", \"message\": \"quota exceeded\" }}"
        )
    }
}

#[derive(Debug, Clone)]
pub enum MqttError {
    InternalError(InternalError),
    UnsubscribeError(UnsubscribeError),
    SubscribeError(SubscribeError),
    PublishError(PublishError),
    SocketClosed(SocketClosed),
    HandleClosed(HandleClosed),
    ContextExited(ContextExited),
    Disconnected(Disconnected),
    CodecError(CodecError),
    QuotaExceeded(QuotaExceeded),
}

impl fmt::Display for MqttError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CodecError(err) => write!(f, "{}", err),
            Self::InternalError(err) => write!(f, "{}", err),
            Self::UnsubscribeError(err) => write!(f, "{}", err),
            Self::SubscribeError(err) => write!(f, "{}", err),
            Self::PublishError(err) => write!(f, "{}", err),
            Self::SocketClosed(err) => {
                write!(f, "{{ \"type\": \"MqttError\", \"message\": \"{}\" }}", err)
            }
            Self::HandleClosed(err) => {
                write!(f, "{{ \"type\": \"MqttError\", \"message\": \"{}\" }}", err)
            }
            Self::ContextExited(err) => {
                write!(f, "{{ \"type\": \"MqttError\", \"message\": \"{}\" }}", err)
            }
            Self::Disconnected(err) => {
                write!(f, "{{ \"type\": \"MqttError\", \"message\": \"{}\" }}", err)
            }
            Self::QuotaExceeded(err) => write!(f, "{}", err),
        }
    }
}

impl Error for MqttError {}

impl From<InternalError> for MqttError {
    fn from(err: InternalError) -> Self {
        Self::InternalError(err)
    }
}

impl From<SocketClosed> for MqttError {
    fn from(err: SocketClosed) -> Self {
        Self::SocketClosed(err)
    }
}

impl From<io::Error> for MqttError {
    fn from(err: io::Error) -> Self {
        Self::SocketClosed(err.into())
    }
}

impl From<HandleClosed> for MqttError {
    fn from(err: HandleClosed) -> Self {
        Self::HandleClosed(err)
    }
}

impl From<Canceled> for MqttError {
    fn from(err: Canceled) -> Self {
        Self::HandleClosed(err.into())
    }
}

impl From<SendError> for MqttError {
    fn from(err: SendError) -> Self {
        Self::HandleClosed(err.into())
    }
}

impl From<ContextExited> for MqttError {
    fn from(err: ContextExited) -> Self {
        Self::ContextExited(err)
    }
}

impl<T> From<TrySendError<T>> for MqttError {
    fn from(err: TrySendError<T>) -> Self {
        Self::ContextExited(err.into())
    }
}

impl From<CodecError> for MqttError {
    fn from(err: CodecError) -> Self {
        Self::CodecError(err)
    }
}

impl From<Disconnected> for MqttError {
    fn from(err: Disconnected) -> Self {
        Self::Disconnected(err)
    }
}

impl From<DisconnectRx> for MqttError {
    fn from(packet: DisconnectRx) -> Self {
        Self::Disconnected(packet.into())
    }
}

impl From<DisconnectReason> for MqttError {
    fn from(reason: DisconnectReason) -> Self {
        Self::Disconnected(reason.into())
    }
}

impl From<QuotaExceeded> for MqttError {
    fn from(err: QuotaExceeded) -> Self {
        Self::QuotaExceeded(err)
    }
}
