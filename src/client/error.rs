use crate::{
    codec::{
        AckRx, AuthReason, AuthRx, ConnackRx, ConnectReason, DisconnectReason, DisconnectRx,
        PubackReason, PubcompReason, PubrecReason,
    },
    core::{collections::UserProperties, error::CodecError},
};
use futures::channel::{mpsc::TrySendError, oneshot::Canceled};
use std::{
    error::Error,
    fmt::{self, Display},
    io, str,
    time::{Duration, SystemTimeError},
};

/// Socket was closed.
///
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

/// Error indicating that [ContextHandle](super::handle::ContextHandle) object
/// required for completing the operation was dropped.
///
#[derive(Debug, Clone)]
pub struct HandleClosed;

impl fmt::Display for HandleClosed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "context handle closed")
    }
}

impl Error for HandleClosed {}

/// Error indicating that client [Context](super::context::Context) has
/// exited ([run](super::context::Context::run) has returned).
///
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

impl From<Canceled> for ContextExited {
    fn from(_: Canceled) -> Self {
        Self
    }
}

/// Broker has terminated the connection by sending DISCONNECT packet.
/// Accesses data in DISCONNECT packet.
///
#[derive(Clone)]
pub struct Disconnected {
    packet: DisconnectRx,
}

impl Disconnected {
    /// Accesses reason value.
    ///
    pub fn reason(&self) -> DisconnectReason {
        self.packet.reason
    }

    /// Accesses session expiry interval.
    ///
    pub fn session_expiry_interval(&self) -> Duration {
        Duration::from_secs(u64::from(u32::from(self.packet.session_expiry_interval)))
    }

    /// Accesses reason string.
    ///
    pub fn reason_string(&self) -> Option<&str> {
        self.packet
            .reason_string
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    /// Accesses server reference.
    ///
    pub fn server_reference(&self) -> Option<&str> {
        self.packet
            .server_reference
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    /// Accesses user properties.
    ///
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

/// Struct representing internal errors. In general, these should not happen and should
/// be trated as an implementation defect.
///
#[derive(Debug, Clone)]
pub struct InternalError {
    msg: &'static str,
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

impl From<&'static str> for InternalError {
    fn from(s: &'static str) -> Self {
        Self { msg: s }
    }
}

impl From<SystemTimeError> for InternalError {
    fn from(_: SystemTimeError) -> Self {
        Self {
            msg: "system time error",
        }
    }
}

/// Trying to send more QoS>0 messages than broker allowed in CONNACK
/// [receive_maximum](super::rsp::ConnectRsp::receive_maximum).
///
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

/// Client attemps to send more data to the server than
/// [maximum packet size](super::rsp::ConnectRsp::maximum_packet_size)
/// property allows.
///
#[derive(Debug, Clone, Copy)]
pub struct MaximumPacketSizeExceeded;

impl fmt::Display for MaximumPacketSizeExceeded {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ \"type\": \"MaximumPacketSizeExceeded\", \"message\": \"packet too large\" }}"
        )
    }
}

/// Connection could not be established with the server. Accesses
/// CONNACK packet with reason value greater or equal 0x80.
///
#[derive(Clone)]
pub struct ConnectError {
    packet: ConnackRx,
}

impl ConnectError {
    /// Accesses reason value.
    ///
    pub fn reason(&self) -> ConnectReason {
        self.packet.reason
    }

    /// Accesses reason string.
    ///
    pub fn reason_string(&self) -> Option<&str> {
        self.packet
            .reason_string
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    /// Accesses server reference.
    ///
    pub fn server_reference(&self) -> Option<&str> {
        self.packet
            .server_reference
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    /// Accesses user properties.
    ///
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

/// Extended authorization was refused by the broker.
/// Accesses AUTH packet with reason value greater or equal 0x80.
///
#[derive(Clone)]
pub struct AuthError {
    packet: AuthRx,
}

impl AuthError {
    /// Accesses reason value.
    ///
    pub fn reason(&self) -> AuthReason {
        self.packet.reason
    }

    /// Accesses reason string.
    ///
    pub fn reason_string(&self) -> Option<&str> {
        self.packet
            .reason_string
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    /// Accesses user properties.
    ///
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

/// Response to the publish request with QoS>0 with reason >= 0x80.
///
#[derive(Clone)]
pub struct AckError<ReasonT>
where
    ReasonT: Default,
{
    pub(crate) packet: AckRx<ReasonT>,
}

impl<ReasonT> AckError<ReasonT>
where
    ReasonT: Default + Copy,
{
    /// Accesses reason value.
    ///
    pub fn reason(&self) -> ReasonT {
        self.packet.reason
    }

    /// Accesses reason string property.
    ///
    pub fn reason_string(&self) -> Option<&str> {
        self.packet
            .reason_string
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    /// Accesses user properties.
    ///
    pub fn user_properties(&self) -> &UserProperties {
        &self.packet.user_property
    }
}

impl<ReasonT> fmt::Debug for AckError<ReasonT>
where
    ReasonT: Copy + Default + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AckError")
            .field("reason", &self.reason())
            .field("reason_string", &self.reason_string())
            .field("user_properties", &self.user_properties())
            .finish()
    }
}

impl<ReasonT> From<AckRx<ReasonT>> for AckError<ReasonT>
where
    ReasonT: Default + fmt::Debug,
{
    fn from(packet: AckRx<ReasonT>) -> Self {
        Self { packet }
    }
}

/// QoS==1 publish failed. Accesses
/// PUBACK packet with reason value greater or equal 0x80.
///
pub type PubackError = AckError<PubackReason>;

impl From<PubackError> for MqttError {
    fn from(err: PubackError) -> Self {
        MqttError::PubackError(err)
    }
}

impl Error for PubackError {}

impl Display for PubackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ \"type\": \"PubackError\", \"message\": \"PubackError error: {} [{:?}]\" }}",
            self.packet.reason as u8, self.packet.reason
        )
    }
}

/// QoS==2 publish failed. Accesses
/// PUBREC packet with reason value greater or equal 0x80.
///
pub type PubrecError = AckError<PubrecReason>;

impl From<PubrecError> for MqttError {
    fn from(err: PubrecError) -> Self {
        MqttError::PubrecError(err)
    }
}

impl Error for PubrecError {}

impl Display for PubrecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ \"type\": \"PubrecError\", \"message\": \"PubrecError error: {} [{:?}]\" }}",
            self.packet.reason as u8, self.packet.reason
        )
    }
}

/// QoS==2 publish failed. Accesses
/// PUBCOMP packet with reason value greater or equal 0x80.
///
pub type PubcompError = AckError<PubcompReason>;

impl From<PubcompError> for MqttError {
    fn from(err: PubcompError) -> Self {
        MqttError::PubcompError(err)
    }
}

impl Error for PubcompError {}

impl Display for PubcompError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ \"type\": \"PubcompError\", \"message\": \"PubcompError error: {} [{:?}]\" }}",
            self.packet.reason as u8, self.packet.reason
        )
    }
}

/// Main library error type. All other errors are converted to this type before being returned to the user.
///
#[derive(Debug, Clone)]
pub enum MqttError {
    /// See [InternalError](crate::client::error::InternalError)
    ///
    InternalError(InternalError),

    /// See [ConnectError](crate::client::error::ConnectError)
    ///
    ConnectError(ConnectError),

    /// See [AuthError](crate::client::error::AuthError)
    ///
    AuthError(AuthError),

    /// See [PubackError](crate::client::error::PubackError)
    ///
    PubackError(PubackError),

    /// See [PubrecError](crate::client::error::PubrecError)
    ///
    PubrecError(PubrecError),

    /// See [PubackError](crate::client::error::PubackError)
    ///
    PubcompError(PubcompError),

    /// See [SocketClosed](crate::client::error::SocketClosed)
    ///
    SocketClosed(SocketClosed),

    /// See [HandleClosed](crate::client::error::HandleClosed)
    ///
    HandleClosed(HandleClosed),

    /// See [ContextExited](crate::client::error::ContextExited)
    ///
    ContextExited(ContextExited),

    /// See [Disconnected](crate::client::error::Disconnected)
    ///
    Disconnected(Disconnected),

    /// See [CodecError](crate::core::error::CodecError)
    ///
    CodecError(CodecError),

    /// See [QuotaExceeded](crate::client::error::QuotaExceeded)
    ///
    QuotaExceeded(QuotaExceeded),

    /// See [MaximumPacketSizeExceeded](crate::client::error::MaximumPacketSizeExceeded)
    ///
    MaximumPacketSizeExceeded(MaximumPacketSizeExceeded),
}

impl fmt::Display for MqttError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InternalError(err) => write!(f, "{}", err),
            Self::ConnectError(err) => write!(f, "{}", err),
            Self::AuthError(err) => write!(f, "{}", err),
            Self::PubackError(err) => write!(f, "{}", err),
            Self::PubrecError(err) => write!(f, "{}", err),
            Self::PubcompError(err) => write!(f, "{}", err),
            Self::CodecError(err) => write!(f, "{}", err),
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
            Self::MaximumPacketSizeExceeded(err) => write!(f, "{}", err),
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
        Self::ContextExited(err.into())
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

impl From<MaximumPacketSizeExceeded> for MqttError {
    fn from(err: MaximumPacketSizeExceeded) -> Self {
        Self::MaximumPacketSizeExceeded(err)
    }
}
