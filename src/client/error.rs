use crate::{
    codec::{AuthRx, ConnackRx, DisconnectRx},
    core::{collections::UserProperties, error::CodecError},
    AuthReason, ConnectReason, DisconnectReason, PubackReason, PubcompReason, PubrecReason,
    SubackReason, UnsubackReason,
};
use futures::channel::{
    mpsc::{SendError, TrySendError},
    oneshot::Canceled,
};
use std::{
    error::Error,
    fmt::{self, Display},
    io, str,
    time::{Duration, SystemTimeError},
};

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

#[derive(Clone)]
pub struct Disconnected {
    packet: DisconnectRx,
}

impl Disconnected {
    pub fn reason(&self) -> DisconnectReason {
        self.packet.reason
    }

    pub fn session_expiry_interval(&self) -> Duration {
        Duration::from_secs(u64::from(u32::from(self.packet.session_expiry_interval)))
    }

    pub fn reason_string(&self) -> Option<&str> {
        self.packet
            .reason_string
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    pub fn server_reference(&self) -> Option<&str> {
        self.packet
            .server_reference
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    pub fn user_properties(&self) -> &UserProperties {
        &self.packet.user_property
    }
}

impl fmt::Debug for Disconnected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Disconnected")
            .field("reason", &self.reason())
            .field("session_expiry_interval", &self.session_expiry_interval())
            .field("reason_string", &self.reason_string())
            .field("server_reference", &self.server_reference())
            .field("user_properties", &self.user_properties())
            .finish()
    }
}

impl fmt::Display for Disconnected {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "disconnected with reason: {} [{:?}]",
            self.reason() as u8,
            self.reason()
        )
    }
}

impl Error for Disconnected {}

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

impl From<SystemTimeError> for InternalError {
    fn from(err: SystemTimeError) -> Self {
        Self {
            msg: err.to_string(),
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

#[derive(Clone)]
pub struct ConnectError {
    packet: ConnackRx,
}

impl ConnectError {
    pub fn reason(&self) -> ConnectReason {
        self.packet.reason
    }

    pub fn reason_string(&self) -> Option<&str> {
        self.packet
            .reason_string
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    pub fn server_reference(&self) -> Option<&str> {
        self.packet
            .server_reference
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    pub fn user_properties(&self) -> &UserProperties {
        &self.packet.user_property
    }
}

impl fmt::Debug for ConnectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectError")
            .field("reason", &self.reason())
            .field("reason_string", &self.reason_string())
            .field("server_reference", &self.server_reference())
            .field("user_properties", &self.user_properties())
            .finish()
    }
}

impl fmt::Display for ConnectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ \"type\": \"ConnectError\", \"message\": \"connect error: {} [{:?}]\" }}",
            self.reason() as u8,
            self.reason()
        )
    }
}

impl Error for ConnectError {}

impl From<ConnackRx> for ConnectError {
    fn from(packet: ConnackRx) -> Self {
        debug_assert!(packet.reason as u8 >= 0x80);
        Self { packet }
    }
}

impl From<ConnectError> for MqttError {
    fn from(err: ConnectError) -> Self {
        MqttError::ConnectError(err)
    }
}

#[derive(Clone)]
pub struct AuthError {
    packet: AuthRx,
}

impl AuthError {
    pub fn reason(&self) -> AuthReason {
        self.packet.reason
    }

    pub fn reason_string(&self) -> Option<&str> {
        self.packet
            .reason_string
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    pub fn user_properties(&self) -> &UserProperties {
        &self.packet.user_property
    }
}

impl fmt::Debug for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthError")
            .field("reason", &self.reason())
            .field("reason_string", &self.reason_string())
            .field("user_properties", &self.user_properties())
            .finish()
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ \"type\": \"AuthError\", \"message\": \"authorization error: {} [{:?}]\" }}",
            self.reason() as u8,
            self.reason()
        )
    }
}

impl Error for AuthError {}

impl From<AuthRx> for AuthError {
    fn from(packet: AuthRx) -> Self {
        debug_assert!(packet.reason as u8 >= 0x80);
        Self { packet }
    }
}

impl From<AuthError> for MqttError {
    fn from(err: AuthError) -> Self {
        MqttError::AuthError(err)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PublishError {
    Puback(PubackReason),
    Pubrec(PubrecReason),
    Pubcomp(PubcompReason),
}

impl From<PublishError> for MqttError {
    fn from(err: PublishError) -> Self {
        MqttError::PublishError(err)
    }
}

impl Error for PublishError {}

impl Display for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Puback(reason) => write!(
                f,
                "{{ \"type\": \"PublishError\", \"message\": \"QoS 1 publish error: {} [{:?}]\" }}",
                *reason as u8, reason,
            ),

            Self::Pubrec(reason) => write!(
                f,
                "{{ \"type\": \"PublishError\", \"message\": \"QoS 2 publish error: {} [{:?}]\" }}",
                *reason as u8, reason,
            ),

            Self::Pubcomp(reason) => write!(
                f,
                "{{ \"type\": \"PublishError\", \"message\": \"QoS 2 publish error: {} [{:?}]\" }}",
                *reason as u8, reason,
            ),
        }
    }
}

impl From<PubackReason> for PublishError {
    fn from(reason: PubackReason) -> Self {
        debug_assert!(reason as u8 >= 0x80);
        Self::Puback(reason)
    }
}

impl From<PubrecReason> for PublishError {
    fn from(reason: PubrecReason) -> Self {
        debug_assert!(reason as u8 >= 0x80);
        Self::Pubrec(reason)
    }
}

impl From<PubcompReason> for PublishError {
    fn from(reason: PubcompReason) -> Self {
        debug_assert!(reason as u8 >= 0x80);
        Self::Pubcomp(reason)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UnsubscribeError {
    pub reason: UnsubackReason,
}

impl From<UnsubscribeError> for MqttError {
    fn from(err: UnsubscribeError) -> Self {
        MqttError::UnsubscribeError(err)
    }
}

impl Error for UnsubscribeError {}

impl Display for UnsubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ \"type\": \"UnsubscribeError\", \"message\": \"unsubscribe error: {} [{:?}]\" }}",
            self.reason as u8, self.reason,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SubscribeError {
    pub reason: SubackReason,
}

impl From<SubscribeError> for MqttError {
    fn from(err: SubscribeError) -> Self {
        MqttError::SubscribeError(err)
    }
}

impl Error for SubscribeError {}

impl Display for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ \"type\": \"SubscribeError\", \"message\": \"subscribe error: {} [{:?}]\" }}",
            self.reason as u8, self.reason,
        )
    }
}

#[derive(Debug, Clone)]
pub enum MqttError {
    InternalError(InternalError),
    ConnectError(ConnectError),
    AuthError(AuthError),
    PublishError(PublishError),
    UnsubscribeError(UnsubscribeError),
    SubscribeError(SubscribeError),
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
            Self::InternalError(err) => write!(f, "{}", err),
            Self::ConnectError(err) => write!(f, "{}", err),
            Self::AuthError(err) => write!(f, "{}", err),
            Self::CodecError(err) => write!(f, "{}", err),
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

impl From<SystemTimeError> for MqttError {
    fn from(err: SystemTimeError) -> Self {
        Self::InternalError(err.into())
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
        Self::Disconnected(Disconnected { packet })
    }
}

impl From<QuotaExceeded> for MqttError {
    fn from(err: QuotaExceeded) -> Self {
        Self::QuotaExceeded(err)
    }
}
