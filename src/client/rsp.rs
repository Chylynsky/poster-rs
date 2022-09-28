use crate::{
    codec::{
        Auth, AuthReason, Connack, ConnectReason, Puback, PubackReason, Suback, SubackReason,
        Unsuback, UnsubackReason,
    },
    core::base_types::{NonZero, QoS},
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
            authentication_data: pck.authentication_data.map(|val| val.into()),
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
    reason: AuthReason,
    authentication_method: Option<String>,
    authentication_data: Option<Vec<u8>>,
    reason_string: Option<String>,
    user_property: Vec<(String, String)>,
}

impl From<Auth> for AuthRsp {
    fn from(pck: Auth) -> Self {
        Self {
            reason: pck.reason,
            authentication_method: pck.authentication_method.map(|val| val.into()),
            authentication_data: pck.authentication_data.map(|val| val.into()),
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
    reason: UnsubackReason,
    reason_string: Option<String>,
    user_property: Vec<(String, String)>,
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
