use crate::{
    codec::*,
    core::base_types::{Binary, NonZero, QoS, UTF8String},
};
use std::str;

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

    pub fn user_property(&self) {
        todo!()
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

    pub fn reason_string(&self) -> Option<&str> {
        self.packet
            .reason_string
            .as_ref()
            .map(|val| &val.0)
            .map(|val| val.0.as_ref())
            .map(str::from_utf8)
            .and_then(Result::ok)
    }

    pub fn user_property(&self) {
        todo!()
    }
}

pub struct SubscribeRsp {
    packet: SubackRx,
}

impl From<SubackRx> for SubscribeRsp {
    fn from(packet: SubackRx) -> Self {
        Self { packet }
    }
}

impl SubscribeRsp {
    pub fn reason(&self) -> SubackReason {
        self.packet.payload.first().copied().unwrap()
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

    pub fn user_property(&self) {
        todo!()
    }
}

pub struct PublishRsp {
    packet: PubackRx,
}

impl From<PubackRx> for PublishRsp {
    fn from(packet: PubackRx) -> Self {
        Self { packet }
    }
}

impl PublishRsp {
    pub fn reason(&self) -> PubackReason {
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

    pub fn user_property(&self) {
        todo!()
    }
}

pub struct UnsubscribeRsp {
    packet: UnsubackRx,
}

impl From<UnsubackRx> for UnsubscribeRsp {
    fn from(packet: UnsubackRx) -> Self {
        Self { packet }
    }
}

impl UnsubscribeRsp {
    pub fn reason(&self) -> UnsubackReason {
        self.packet.payload.first().copied().unwrap()
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

    pub fn user_property(&self) {
        todo!()
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

    pub fn user_property(&self) {
        todo!()
    }

    pub fn payload(&self) -> &[u8] {
        self.packet.payload.0.as_ref()
    }
}
