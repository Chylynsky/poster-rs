use crate::{
    client::{error::ConnectError, message::ContextMessage, stream::SubscribeStreamState},
    codec::*,
    core::{
        base_types::{NonZero, QoS},
        collections::UserProperties,
    },
    AuthError,
};
use futures::{
    channel::mpsc::{self},
    stream, Stream,
};
use std::str;

pub struct ConnectRsp {
    packet: ConnackRx,
}

impl TryFrom<ConnackRx> for ConnectRsp {
    type Error = ConnectError;

    fn try_from(packet: ConnackRx) -> Result<Self, Self::Error> {
        if packet.reason as u8 >= 0x80 {
            return Err(ConnectError::from(packet));
        }

        assert!(
            bool::from(packet.subscription_identifier_available),
            "Subscription identifier support is required. Check your broker settings."
        );
        Ok(Self { packet })
    }
}

impl ConnectRsp {
    pub fn session_present(&self) -> bool {
        self.packet.session_present
    }

    pub fn reason(&self) -> ConnectReason {
        self.packet.reason
    }

    pub fn wildcard_subscription_available(&self) -> bool {
        bool::from(self.packet.wildcard_subscription_available)
    }

    pub fn subscription_identifier_available(&self) -> bool {
        bool::from(self.packet.subscription_identifier_available)
    }

    pub fn shared_subscription_available(&self) -> bool {
        bool::from(self.packet.shared_subscription_available)
    }

    pub fn maximum_qos(&self) -> QoS {
        QoS::from(self.packet.maximum_qos)
    }

    pub fn retain_available(&self) -> bool {
        bool::from(self.packet.retain_available)
    }

    pub fn server_keep_alive(&self) -> Option<u16> {
        self.packet.server_keep_alive.map(u16::from)
    }

    pub fn receive_maximum(&self) -> u16 {
        NonZero::from(self.packet.receive_maximum).get()
    }

    pub fn topic_alias_maximum(&self) -> u16 {
        u16::from(self.packet.topic_alias_maximum)
    }

    pub fn session_expiry_interval(&self) -> u32 {
        u32::from(self.packet.session_expiry_interval)
    }

    pub fn maximum_packet_size(&self) -> Option<u32> {
        self.packet
            .maximum_packet_size
            .map(NonZero::from)
            .map(|val| val.get())
    }

    pub fn assigned_client_identifier(&self) -> Option<&str> {
        self.packet
            .assigned_client_identifier
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
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

    pub fn response_information(&self) -> Option<&str> {
        self.packet
            .response_information
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

    pub fn authentication_method(&self) -> Option<&str> {
        self.packet
            .authentication_method
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    pub fn authentication_data(&self) -> Option<&[u8]> {
        self.packet
            .authentication_data
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
    }

    pub fn user_properties(&self) -> &UserProperties {
        &self.packet.user_property
    }
}

pub struct AuthRsp {
    packet: AuthRx,
}

impl TryFrom<AuthRx> for AuthRsp {
    type Error = AuthError;

    fn try_from(packet: AuthRx) -> Result<Self, Self::Error> {
        if packet.reason as u8 >= 0x80 {
            return Err(AuthError::from(packet));
        }

        Ok(Self { packet })
    }
}

impl AuthRsp {
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

    pub fn authentication_method(&self) -> Option<&str> {
        self.packet
            .authentication_method
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    pub fn authentication_data(&self) -> Option<&[u8]> {
        self.packet
            .authentication_data
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
    }

    pub fn user_properties(&self) -> &UserProperties {
        &self.packet.user_property
    }
}

/// Response to the subscription request, representing the Suback packet.
///
/// In order to receive messages published on the subscribed topics use
/// the [stream](SubscribeRsp::stream) method.
///
pub struct SubscribeRsp {
    pub(crate) packet: SubackRx,
    pub(crate) receiver: mpsc::UnboundedReceiver<RxPacket>,
    pub(crate) sender: mpsc::UnboundedSender<ContextMessage>,
}

impl SubscribeRsp {
    /// Transforms this response into the asynchronous stream of messages
    /// published to the subscribed topics.
    ///
    pub fn stream(self) -> impl Stream<Item = PublishData> {
        Box::pin(stream::unfold(
            SubscribeStreamState {
                receiver: self.receiver,
                sender: self.sender,
            },
            |mut state| async { state.impl_next().await.map(move |data| (data, state)) },
        ))
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

    /// Accesses the payload. Payload is a list of [SubackReason] codes,
    /// representing the subscription result for each subscribed topic.
    ///
    pub fn payload(&self) -> &[SubackReason] {
        &self.packet.payload
    }
}

/// Response to the subscription request, representing the Suback packet.
///
/// In order to receive messages published on the subscribed topics use
/// the [stream](SubscribeRsp::stream) method.
///
pub struct UnsubscribeRsp {
    pub(crate) packet: UnsubackRx,
}

impl UnsubscribeRsp {
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

    /// Accesses the payload. Payload is a list of [SubackReason] codes,
    /// representing the subscription result for each subscribed topic.
    ///
    pub fn payload(&self) -> &[UnsubackReason] {
        &self.packet.payload
    }
}

pub struct PublishData {
    packet: PublishRx,
}

impl From<PublishRx> for PublishData {
    fn from(packet: PublishRx) -> Self {
        Self { packet }
    }
}

impl PublishData {
    pub fn dup(&self) -> bool {
        self.packet.dup
    }

    pub fn retain(&self) -> bool {
        self.packet.retain
    }

    pub fn qos(&self) -> QoS {
        self.packet.qos
    }

    pub fn topic_name(&self) -> &str {
        str::from_utf8(self.packet.topic_name.0.as_ref()).unwrap()
    }

    pub fn payload_format_indicator(&self) -> Option<bool> {
        self.packet.payload_format_indicator.map(bool::from)
    }

    pub fn topic_alias(&self) -> Option<u16> {
        self.packet
            .topic_alias
            .map(NonZero::from)
            .map(|val| val.get())
    }

    pub fn message_expiry_interval(&self) -> Option<u32> {
        self.packet.message_expiry_interval.map(u32::from)
    }

    pub fn subscription_identifier(&self) -> Option<u32> {
        self.packet
            .subscription_identifier
            .map(NonZero::from)
            .map(|val| val.get())
            .map(|val| val.value())
    }

    pub fn correlation_data(&self) -> Option<&[u8]> {
        self.packet
            .correlation_data
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
    }

    pub fn response_topic(&self) -> Option<&str> {
        self.packet
            .response_topic
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    pub fn content_type(&self) -> Option<&str> {
        self.packet
            .content_type
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    pub fn payload(&self) -> &[u8] {
        self.packet.payload.0.as_ref()
    }

    pub fn user_properties(&self) -> &UserProperties {
        &self.packet.user_property
    }
}
