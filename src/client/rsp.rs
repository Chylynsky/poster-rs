use crate::{
    client::{
        error::{AuthError, ConnectError},
        message::ContextMessage,
        stream::SubscribeStreamState,
    },
    codec::*,
    core::{
        base_types::{NonZero, QoS},
        collections::UserProperties,
    },
};
use futures::{
    channel::mpsc::{self},
    stream, Stream,
};
use std::{str, time::Duration};

use super::error::{PubackError, PubcompError, PubrecError};

/// Response from connection request.
/// Accesses data in CONNACK packet.
///
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
    /// Accesses session present flag.
    ///
    pub fn session_present(&self) -> bool {
        self.packet.session_present
    }

    /// Accesses reason value.
    ///
    pub fn reason(&self) -> ConnectReason {
        self.packet.reason
    }

    /// Accesses flag representing if wildcard subscriptions are available.
    ///
    pub fn wildcard_subscription_available(&self) -> bool {
        bool::from(self.packet.wildcard_subscription_available)
    }

    /// Accesses flag representing if subscription identifiers are available.
    ///
    pub fn subscription_identifier_available(&self) -> bool {
        bool::from(self.packet.subscription_identifier_available)
    }

    /// Accesses flag representing if shared subscriptions are available.
    ///
    pub fn shared_subscription_available(&self) -> bool {
        bool::from(self.packet.shared_subscription_available)
    }

    /// Accesses maximum QoS value.
    ///
    pub fn maximum_qos(&self) -> QoS {
        QoS::from(self.packet.maximum_qos)
    }

    /// Accesses flag representing if retain is available.
    ///
    pub fn retain_available(&self) -> bool {
        bool::from(self.packet.retain_available)
    }

    /// Accesses server keep alive.
    ///
    pub fn server_keep_alive(&self) -> Option<Duration> {
        self.packet
            .server_keep_alive
            .map(u16::from)
            .map(u64::from)
            .map(Duration::from_secs)
    }

    /// Accesses server receive maximum value.
    ///
    pub fn receive_maximum(&self) -> u16 {
        NonZero::from(self.packet.receive_maximum).get()
    }

    /// Accesses topic alias maximum value.
    ///
    pub fn topic_alias_maximum(&self) -> u16 {
        u16::from(self.packet.topic_alias_maximum)
    }

    /// Accesses session expiry interval value.
    ///
    pub fn session_expiry_interval(&self) -> Option<Duration> {
        self.packet
            .session_expiry_interval
            .map(u32::from)
            .map(u64::from)
            .map(Duration::from_secs)
    }

    /// Accesses server maximum packet size.
    ///
    pub fn maximum_packet_size(&self) -> Option<u32> {
        self.packet
            .maximum_packet_size
            .map(NonZero::from)
            .map(|val| val.get())
    }

    /// Accesses client identifier assigned by the server.
    ///
    pub fn assigned_client_identifier(&self) -> Option<&str> {
        self.packet
            .assigned_client_identifier
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
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

    /// Accesses response information.
    ///
    pub fn response_information(&self) -> Option<&str> {
        self.packet
            .response_information
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

    /// Accesses authentication method.
    ///
    pub fn authentication_method(&self) -> Option<&str> {
        self.packet
            .authentication_method
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    /// Accesses authentication data.
    ///
    pub fn authentication_data(&self) -> Option<&[u8]> {
        self.packet
            .authentication_data
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
    }

    /// Accesses user properties.
    ///
    pub fn user_properties(&self) -> &UserProperties {
        &self.packet.user_property
    }
}

/// Response from connection request, if extended authorization is performed.
/// Accesses data in AUTH packet.
///
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

    /// Accesses authentication method.
    ///
    pub fn authentication_method(&self) -> Option<&str> {
        self.packet
            .authentication_method
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    /// Accesses authentication data.
    ///
    pub fn authentication_data(&self) -> Option<&[u8]> {
        self.packet
            .authentication_data
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
    }

    /// Accesses user properties.
    ///
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

    /// Accesses the payload being a list of [SubackReason] codes.
    /// Each reason code represents the result of the subscribe operation
    /// for the given topic.
    ///
    pub fn payload(&self) -> &[SubackReason] {
        &self.packet.payload
    }
}

/// Response to the unsubscribe request, representing the UNSUBACK packet.
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

/// Accesses data in the incoming PUBLISH packet.
///
pub struct PublishData {
    packet: PublishRx,
}

impl From<PublishRx> for PublishData {
    fn from(packet: PublishRx) -> Self {
        Self { packet }
    }
}

impl PublishData {
    /// Accesses duplicate flag.
    ///
    pub fn dup(&self) -> bool {
        self.packet.dup
    }

    /// Accesses retain flag.
    ///
    pub fn retain(&self) -> bool {
        self.packet.retain
    }

    /// Accesses QoS value.
    ///
    pub fn qos(&self) -> QoS {
        self.packet.qos
    }

    /// Accesses topic name.
    ///
    pub fn topic_name(&self) -> &str {
        str::from_utf8(self.packet.topic_name.0.as_ref()).unwrap()
    }

    /// Accesses payload format indicator.
    ///
    pub fn payload_format_indicator(&self) -> Option<bool> {
        self.packet.payload_format_indicator.map(bool::from)
    }

    /// Accesses topic alias.
    ///
    pub fn topic_alias(&self) -> Option<u16> {
        self.packet
            .topic_alias
            .map(NonZero::from)
            .map(|val| val.get())
    }

    /// Accesses message expiry interval.
    ///
    pub fn message_expiry_interval(&self) -> Option<Duration> {
        self.packet
            .message_expiry_interval
            .map(u32::from)
            .map(u64::from)
            .map(Duration::from_secs)
    }

    /// Accesses correlation data.
    ///
    pub fn correlation_data(&self) -> Option<&[u8]> {
        self.packet
            .correlation_data
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
    }

    /// Accesses response topic.
    ///
    pub fn response_topic(&self) -> Option<&str> {
        self.packet
            .response_topic
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    /// Accesses content type.
    ///
    pub fn content_type(&self) -> Option<&str> {
        self.packet
            .content_type
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    /// Accesses payload.
    ///
    pub fn payload(&self) -> &[u8] {
        self.packet.payload.0.as_ref()
    }

    /// Accesses user properties.
    ///
    pub fn user_properties(&self) -> &UserProperties {
        &self.packet.user_property
    }

    pub(crate) fn subscription_identifier(&self) -> Option<u32> {
        self.packet
            .subscription_identifier
            .map(NonZero::from)
            .map(|val| val.get())
            .map(|val| val.value())
    }
}

/// Response to the publish request, with QoS==1 representing the PUBACK packet.
///
pub struct PubackRsp {
    pub(crate) packet: PubackRx,
}

impl PubackRsp {
    /// Accesses reason value.
    ///
    pub fn reason(&self) -> PubackReason {
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

impl TryFrom<PubackRx> for PubackRsp {
    type Error = PubackError;

    fn try_from(packet: PubackRx) -> Result<Self, Self::Error> {
        if packet.reason as u8 >= 0x80 {
            return Err(PubackError::from(packet));
        }

        Ok(Self { packet })
    }
}

/// Response to the publish request, with QoS==2 representing the PUBREC packet.
///
pub struct PubrecRsp {
    pub(crate) packet: PubrecRx,
}

impl PubrecRsp {
    /// Accesses reason value.
    ///
    pub fn reason(&self) -> PubrecReason {
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

impl TryFrom<PubrecRx> for PubrecRsp {
    type Error = PubrecError;

    fn try_from(packet: PubrecRx) -> Result<Self, Self::Error> {
        if packet.reason as u8 >= 0x80 {
            return Err(PubrecError::from(packet));
        }

        Ok(Self { packet })
    }
}

/// Response to the publish request, with QoS==2 representing the PUBCOMP packet.
///
pub struct PubcompRsp {
    pub(crate) packet: PubcompRx,
}

impl PubcompRsp {
    /// Accesses reason value.
    ///
    pub fn reason(&self) -> PubcompReason {
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

impl TryFrom<PubcompRx> for PubcompRsp {
    type Error = PubcompError;

    fn try_from(packet: PubcompRx) -> Result<Self, Self::Error> {
        if packet.reason as u8 >= 0x80 {
            return Err(PubcompError::from(packet));
        }

        Ok(Self { packet })
    }
}
