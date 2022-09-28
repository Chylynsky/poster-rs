use crate::{
    codec::{
        Auth, AuthBuilder, AuthReason, Connect, ConnectBuilder, Publish, PublishBuilder,
        RetainHandling, Subscribe, SubscribeBuilder, SubscriptionOptions, Unsubscribe,
        UnsubscribeBuilder,
    },
    core::base_types::{NonZero, QoS, VarSizeInt},
};

#[derive(Default)]
pub struct ConnectOpts {
    builder: ConnectBuilder,
}

impl ConnectOpts {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mandatory field.
    pub fn client_identifier(mut self, val: String) -> Self {
        self.builder.client_identifier(val);
        self
    }

    pub fn keep_alive(mut self, val: u16) -> Self {
        self.builder.keep_alive(val);
        self
    }

    pub fn session_expiry_interval(mut self, val: u32) -> Self {
        self.builder.session_expiry_interval(val);
        self
    }

    pub fn receive_maximum(mut self, val: u16) -> Self {
        self.builder.receive_maximum(val.into());
        self
    }

    pub fn maximum_packet_size(mut self, val: u32) -> Self {
        self.builder.maximum_packet_size(val.into());
        self
    }

    pub fn topic_alias_maximum(mut self, val: u16) -> Self {
        self.builder.topic_alias_maximum(val);
        self
    }

    pub fn request_response_information(mut self, val: bool) -> Self {
        self.builder.request_response_information(val);
        self
    }

    pub fn request_problem_information(mut self, val: bool) -> Self {
        self.builder.request_problem_information(val);
        self
    }

    pub fn authentication_method(mut self, val: String) -> Self {
        self.builder.authentication_method(val);
        self
    }

    pub fn user_property(mut self, val: (String, String)) -> Self {
        self.builder.user_property(val);
        self
    }

    pub fn will_qos(mut self, val: QoS) -> Self {
        self.builder.will_qos(val);
        self
    }

    pub fn will_retain(mut self, val: bool) -> Self {
        self.builder.will_retain(val);
        self
    }

    pub fn clean_start(mut self, val: bool) -> Self {
        self.builder.clean_start(val);
        self
    }

    pub fn will_delay_interval(mut self, val: u32) -> Self {
        self.builder.will_delay_interval(val);
        self
    }

    pub fn will_payload_format_indicator(mut self, val: bool) -> Self {
        self.builder.will_payload_format_indicator(val);
        self
    }

    pub fn will_message_expiry_interval(mut self, val: u32) -> Self {
        self.builder.will_message_expiry_interval(val);
        self
    }

    pub fn will_content_type(mut self, val: String) -> Self {
        self.builder.will_content_type(val);
        self
    }

    pub fn will_reponse_topic(mut self, val: String) -> Self {
        self.builder.will_reponse_topic(val);
        self
    }

    pub fn will_correlation_data(mut self, val: Vec<u8>) -> Self {
        self.builder.will_correlation_data(val);
        self
    }

    pub fn will_user_property(mut self, val: (String, String)) -> Self {
        self.builder.will_user_property(val);
        self
    }

    pub fn will_topic(mut self, val: String) -> Self {
        self.builder.will_topic(val);
        self
    }

    pub fn will_payload(mut self, val: Vec<u8>) -> Self {
        self.builder.will_payload(val);
        self
    }

    pub fn username(mut self, val: String) -> Self {
        self.builder.username(val);
        self
    }

    pub fn password(mut self, val: Vec<u8>) -> Self {
        self.builder.password(val);
        self
    }

    pub(crate) fn build(self) -> Option<Connect> {
        self.builder.build()
    }
}

#[derive(Default)]
pub struct AuthOpts {
    builder: AuthBuilder,
}

impl AuthOpts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reason(mut self, val: AuthReason) -> Self {
        self.builder.reason(val);
        self
    }

    pub fn authentication_data(mut self, val: Vec<u8>) -> Self {
        self.builder.authentication_data(val);
        self
    }

    pub fn authentication_method(mut self, val: String) -> Self {
        self.builder.authentication_method(val);
        self
    }

    pub fn reason_string(mut self, val: String) -> Self {
        self.builder.reason_string(val);
        self
    }

    pub fn user_property(mut self, val: (String, String)) -> Self {
        self.builder.user_property(val);
        self
    }

    pub(crate) fn build(self) -> Option<Auth> {
        self.builder.build()
    }
}

#[derive(Default)]
pub struct SubscribeOpts {
    builder: SubscribeBuilder,

    topic: String,
    opts: SubscriptionOptions,
}

impl SubscribeOpts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn topic(mut self, val: String) -> Self {
        self.topic = val;
        self
    }

    pub fn no_local(mut self, val: bool) -> Self {
        self.opts.no_local = val;
        self
    }

    pub fn retain_as_published(mut self, val: bool) -> Self {
        self.opts.retain_as_published = val;
        self
    }

    pub fn retain_handling(mut self, val: RetainHandling) -> Self {
        self.opts.retain_handling = val;
        self
    }

    pub fn user_property(mut self, val: (String, String)) -> Self {
        self.builder.user_property(val);
        self
    }

    pub(crate) fn packet_identifier(mut self, val: u16) -> Self {
        self.builder.packet_identifier(NonZero::from(val));
        self
    }

    pub(crate) fn build(self) -> Option<Subscribe> {
        let mut opts = self;
        opts.builder.payload((opts.topic, opts.opts));
        opts.builder.build()
    }
}

#[derive(Default)]
pub struct PublishOpts {
    builder: PublishBuilder,
}

impl PublishOpts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn retain(mut self, val: bool) -> Self {
        self.builder.retain(val);
        self
    }

    pub fn qos(mut self, val: QoS) -> Self {
        self.builder.qos(val);
        self
    }

    pub fn topic_name(mut self, val: String) -> Self {
        self.builder.topic_name(val);
        self
    }

    pub fn packet_identifier(mut self, val: u16) -> Self {
        self.builder.packet_identifier(NonZero::from(val));
        self
    }

    pub fn payload_format_indicator(mut self, val: bool) -> Self {
        self.builder.payload_format_indicator(val);
        self
    }

    pub fn topic_alias(mut self, val: u16) -> Self {
        self.builder.topic_alias(NonZero::from(val));
        self
    }

    pub fn message_expiry_interval(mut self, val: u32) -> Self {
        self.builder.message_expiry_interval(val);
        self
    }

    pub fn subscription_identifier(mut self, val: u32) -> Self {
        self.builder
            .subscription_identifier(NonZero::from(VarSizeInt::from(val)));
        self
    }

    pub fn correlation_data(mut self, val: Vec<u8>) -> Self {
        self.builder.correlation_data(val);
        self
    }

    pub fn response_topic(mut self, val: String) -> Self {
        self.builder.response_topic(val);
        self
    }

    pub fn content_type(mut self, val: String) -> Self {
        self.builder.content_type(val);
        self
    }

    pub fn user_property(mut self, val: (String, String)) -> Self {
        self.builder.user_property(val);
        self
    }

    pub fn payload(mut self, val: Vec<u8>) -> Self {
        self.builder.payload(val);
        self
    }

    pub(crate) fn build(self) -> Option<Publish> {
        self.builder.build()
    }
}

#[derive(Default)]
pub struct UnsubscribeOpts {
    builder: UnsubscribeBuilder,
}

impl UnsubscribeOpts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn user_property(mut self, val: (String, String)) -> Self {
        self.builder.user_property(val);
        self
    }

    pub fn payload(mut self, val: String) -> Self {
        self.builder.payload(val);
        self
    }

    pub(crate) fn packet_identifier(mut self, val: u16) -> Self {
        self.builder.packet_identifier(NonZero::from(val));
        self
    }

    pub(crate) fn build(self) -> Option<Unsubscribe> {
        self.builder.build()
    }
}
