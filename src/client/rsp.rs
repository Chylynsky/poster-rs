use crate::{
    codec::{
        Auth, AuthReason, Connack, ConnectReason, Puback, PubackReason, Publish, Suback,
        SubackReason, Unsuback, UnsubackReason,
    },
    core::base_types::{Binary, NonZero, QoS, VarSizeInt},
};

pub struct ConnectRsp {
    pub session_present: bool,
    pub reason: ConnectReason,
    pub wildcard_subscription_available: bool,
    pub subscription_identifier_available: bool,
    pub shared_subscription_available: bool,
    pub maximum_qos: QoS,
    pub retain_available: bool,
    pub server_keep_alive: Option<u16>,
    pub receive_maximum: u16,
    pub topic_alias_maximum: u16,
    pub session_expiry_interval: u32,
    pub maximum_packet_size: Option<u32>,
    pub authentication_data: Option<Vec<u8>>,
    pub assigned_client_identifier: Option<String>,
    pub reason_string: Option<String>,
    pub response_information: Option<String>,
    pub server_reference: Option<String>,
    pub authentication_method: Option<String>,
    pub user_property: Vec<(String, String)>,
}

impl From<Connack> for ConnectRsp {
    fn from(pck: Connack) -> Self {
        Self {
            session_present: pck.session_present,
            reason: pck.reason,
            wildcard_subscription_available: pck.wildcard_subscription_available.into(),
            subscription_identifier_available: pck.subscription_identifier_available.into(),
            shared_subscription_available: pck.shared_subscription_available.into(),
            maximum_qos: pck.maximum_qos.into(),
            retain_available: pck.retain_available.into(),
            server_keep_alive: pck.server_keep_alive.map(|val| val.into()),
            receive_maximum: NonZero::from(pck.receive_maximum).into(),
            topic_alias_maximum: pck.topic_alias_maximum.into(),
            session_expiry_interval: pck.session_expiry_interval.into(),
            maximum_packet_size: pck.maximum_packet_size.map(|val| NonZero::from(val).into()),
            authentication_data: pck
                .authentication_data
                .map(|val| -> Vec<u8> { Binary::from(val).into() }),
            assigned_client_identifier: pck.assigned_client_identifier.map(|val| val.into()),
            reason_string: pck.reason_string.map(|val| val.into()),
            response_information: pck.response_information.map(|val| val.into()),
            server_reference: pck.server_reference.map(|val| val.into()),
            authentication_method: pck.authentication_method.map(|val| val.into()),
            user_property: pck
                .user_property
                .into_iter()
                .map(|val| val.into())
                .collect::<Vec<(String, String)>>(),
        }
    }
}

pub struct AuthRsp {
    pub reason: AuthReason,
    pub authentication_method: Option<String>,
    pub authentication_data: Option<Vec<u8>>,
    pub reason_string: Option<String>,
    pub user_property: Vec<(String, String)>,
}

impl From<Auth> for AuthRsp {
    fn from(pck: Auth) -> Self {
        Self {
            reason: pck.reason,
            authentication_method: pck.authentication_method.map(|val| val.into()),
            authentication_data: pck.authentication_data.map(|val| Binary::from(val).into()),
            reason_string: pck.reason_string.map(|val| val.into()),
            user_property: pck
                .user_property
                .into_iter()
                .map(|val| val.into())
                .collect::<Vec<(String, String)>>(),
        }
    }
}

pub struct SubscribeRsp {
    pub reason: SubackReason,
    pub reason_string: Option<String>,
    pub user_property: Vec<(String, String)>,
}

impl From<Suback> for SubscribeRsp {
    fn from(pck: Suback) -> Self {
        Self {
            reason: pck.payload[0],
            reason_string: pck.reason_string.map(|val| val.into()),
            user_property: pck
                .user_property
                .into_iter()
                .map(|val| val.into())
                .collect::<Vec<(String, String)>>(),
        }
    }
}

pub struct PublishRsp {
    pub reason: PubackReason,
    pub reason_string: Option<String>,
    pub user_property: Vec<(String, String)>,
}

impl From<Puback> for PublishRsp {
    fn from(pck: Puback) -> Self {
        Self {
            reason: pck.reason,
            reason_string: pck.reason_string.map(|val| val.into()),
            user_property: pck
                .user_property
                .into_iter()
                .map(|val| val.into())
                .collect::<Vec<(String, String)>>(),
        }
    }
}

pub struct UnsubscribeRsp {
    pub reason: UnsubackReason,
    pub reason_string: Option<String>,
    pub user_property: Vec<(String, String)>,
}

impl From<Unsuback> for UnsubscribeRsp {
    fn from(pck: Unsuback) -> Self {
        Self {
            reason: pck.payload[0],
            reason_string: pck.reason_string.map(|val| val.into()),
            user_property: pck
                .user_property
                .into_iter()
                .map(|val| val.into())
                .collect::<Vec<(String, String)>>(),
        }
    }
}

pub struct PublishData {
    pub dup: bool,
    pub retain: bool,
    pub qos: QoS,

    pub topic_name: String,

    pub payload_format_indicator: Option<bool>,
    pub topic_alias: Option<u16>,
    pub message_expiry_interval: Option<u32>,
    pub subscription_identifier: Option<u32>,
    pub correlation_data: Option<Vec<u8>>,
    pub response_topic: Option<String>,
    pub content_type: Option<String>,
    pub user_property: Vec<(String, String)>,

    pub payload: Vec<u8>,
}

impl From<Publish> for PublishData {
    fn from(pck: Publish) -> Self {
        Self {
            dup: pck.dup,
            retain: pck.retain,
            qos: pck.qos,
            topic_name: pck.topic_name,
            payload_format_indicator: pck.payload_format_indicator.map(|val| val.into()),
            topic_alias: pck.topic_alias.map(|val| {
                let topic_alias: NonZero<u16> = val.into();
                topic_alias.value()
            }),
            message_expiry_interval: pck.message_expiry_interval.map(|val| val.into()),
            subscription_identifier: pck.subscription_identifier.map(|val| {
                let sub_id: NonZero<VarSizeInt> = val.into();
                sub_id.value().into()
            }),
            correlation_data: pck.correlation_data.map(|val| Binary::from(val).into()),
            response_topic: pck.response_topic.map(|val| val.into()),
            content_type: pck.content_type.map(|val| val.into()),
            user_property: pck
                .user_property
                .into_iter()
                .map(|val| val.into())
                .collect::<Vec<(String, String)>>(),
            payload: pck.payload,
        }
    }
}
