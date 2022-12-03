use crate::{
    codec::*,
    core::base_types::{NonZero, QoS},
};
use std::{error::Error, fmt::Display, str};

use super::error::MqttError;

pub struct ConnectRsp {
    packet: ConnackRx,
}

impl From<ConnackRx> for ConnectRsp {
    fn from(packet: ConnackRx) -> Self {
        Self { packet }
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
}

pub struct AuthRsp {
    packet: AuthRx,
}

impl From<AuthRx> for AuthRsp {
    fn from(packet: AuthRx) -> Self {
        Self { packet }
    }
}

impl AuthRsp {
    pub fn reason(&self) -> AuthReason {
        self.packet.reason
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
}
