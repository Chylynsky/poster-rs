use crate::{
    codec::*,
    core::{base_types::*, error::CodecError, properties::*},
};
use core::time::Duration;

#[derive(Default)]
pub struct ConnectOpts<'a> {
    builder: ConnectTxBuilder<'a>,
}

impl<'a> ConnectOpts<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mandatory field.
    pub fn client_identifier(mut self, val: &'a str) -> Self {
        self.builder.client_identifier(UTF8StringRef(val));
        self
    }

    pub fn keep_alive(mut self, val: u16) -> Self {
        self.builder.keep_alive(val);
        self
    }

    pub fn session_expiry_interval(mut self, val: u32) -> Self {
        self.builder
            .session_expiry_interval(SessionExpiryInterval::from(val));
        self
    }

    pub fn receive_maximum(mut self, val: u16) -> Self {
        self.builder
            .receive_maximum(ReceiveMaximum::from(NonZero::try_from(val).unwrap()));
        self
    }

    pub fn maximum_packet_size(mut self, val: u32) -> Self {
        self.builder
            .maximum_packet_size(MaximumPacketSize::from(NonZero::try_from(val).unwrap()));
        self
    }

    pub fn topic_alias_maximum(mut self, val: u16) -> Self {
        self.builder
            .topic_alias_maximum(TopicAliasMaximum::from(val));
        self
    }

    pub fn request_response_information(mut self, val: bool) -> Self {
        self.builder
            .request_response_information(RequestResponseInformation::from(val));
        self
    }

    pub fn request_problem_information(mut self, val: bool) -> Self {
        self.builder
            .request_problem_information(RequestProblemInformation::from(val));
        self
    }

    pub fn authentication_method(mut self, val: &'a str) -> Self {
        self.builder
            .authentication_method(AuthenticationMethodRef::from(UTF8StringRef(val)));
        self
    }

    pub fn user_property(mut self, (key, val): (&'a str, &'a str)) -> Self {
        self.builder
            .user_property(UserPropertyRef::from(UTF8StringPairRef(key, val)));
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
        self.builder
            .will_delay_interval(WillDelayInterval::from(val));
        self
    }

    pub fn will_payload_format_indicator(mut self, val: bool) -> Self {
        self.builder
            .will_payload_format_indicator(PayloadFormatIndicator::from(val));
        self
    }

    pub fn will_message_expiry_interval(mut self, val: u32) -> Self {
        self.builder
            .will_message_expiry_interval(MessageExpiryInterval::from(val));
        self
    }

    pub fn will_content_type(mut self, val: &'a str) -> Self {
        self.builder
            .will_content_type(ContentTypeRef::from(UTF8StringRef(val)));
        self
    }

    pub fn will_reponse_topic(mut self, val: &'a str) -> Self {
        self.builder
            .will_reponse_topic(ResponseTopicRef::from(UTF8StringRef(val)));
        self
    }

    pub fn will_correlation_data(mut self, val: &'a [u8]) -> Self {
        self.builder
            .will_correlation_data(CorrelationDataRef::from(BinaryRef(val)));
        self
    }

    pub fn will_user_property(mut self, (key, val): (&'a str, &'a str)) -> Self {
        self.builder
            .will_user_property(UserPropertyRef::from(UTF8StringPairRef(key, val)));
        self
    }

    pub fn will_topic(mut self, val: &'a str) -> Self {
        self.builder.will_topic(UTF8StringRef(val));
        self
    }

    pub fn will_payload(mut self, val: &'a [u8]) -> Self {
        self.builder.will_payload(BinaryRef(val));
        self
    }

    pub fn username(mut self, val: &'a str) -> Self {
        self.builder.username(UTF8StringRef(val));
        self
    }

    pub fn password(mut self, val: &'a [u8]) -> Self {
        self.builder.password(BinaryRef(val));
        self
    }

    pub(crate) fn build(self) -> Result<ConnectTx<'a>, CodecError> {
        self.builder.build()
    }
}

#[derive(Default)]
pub struct AuthOpts<'a> {
    builder: AuthTxBuilder<'a>,
}

impl<'a> AuthOpts<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reason(mut self, val: AuthReason) -> Self {
        self.builder.reason(val);
        self
    }

    pub fn reason_string(mut self, val: &'a str) {
        self.builder
            .reason_string(ReasonStringRef::from(UTF8StringRef(val)));
    }

    pub fn authentication_data(mut self, val: &'a [u8]) -> Self {
        self.builder
            .authentication_data(AuthenticationDataRef::from(BinaryRef(val)));
        self
    }

    pub fn authentication_method(mut self, val: &'a str) -> Self {
        self.builder
            .authentication_method(AuthenticationMethodRef::from(UTF8StringRef(val)));
        self
    }

    pub fn user_property(mut self, (key, val): (&'a str, &'a str)) -> Self {
        self.builder
            .user_property(UserPropertyRef::from(UTF8StringPairRef(key, val)));
        self
    }

    pub(crate) fn build(self) -> Result<AuthTx<'a>, CodecError> {
        self.builder.build()
    }
}

#[derive(Default)]
pub struct DisconnectOpts<'a> {
    builder: DisconnectTxBuilder<'a>,
}

impl<'a> DisconnectOpts<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reason(mut self, reason: DisconnectReason) -> Self {
        self.builder.reason(reason);
        self
    }

    pub fn session_expiry_interval(mut self, val: Duration) -> Self {
        self.builder
            .session_expiry_interval(SessionExpiryInterval::from(
                u32::try_from(val.as_secs()).unwrap(),
            ));
        self
    }

    pub fn reason_string(mut self, val: &'a str) -> Self {
        self.builder
            .reason_string(ReasonStringRef::from(UTF8StringRef(val)));
        self
    }

    pub fn user_property(mut self, (key, val): (&'a str, &'a str)) -> Self {
        self.builder
            .user_property(UserPropertyRef::from(UTF8StringPairRef(key, val)));
        self
    }

    pub(crate) fn build(self) -> Result<DisconnectTx<'a>, CodecError> {
        self.builder.build()
    }
}

#[derive(Default)]
pub struct SubscribeOpts<'a> {
    builder: SubscribeTxBuilder<'a>,
}

impl<'a> SubscribeOpts<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscription(mut self, topic: &'a str, opts: SubscriptionOptions) -> Self {
        self.builder.payload((UTF8StringRef(topic), opts));
        self
    }

    pub fn user_property(mut self, (key, val): (&'a str, &'a str)) -> Self {
        self.builder
            .user_property(UserPropertyRef::from(UTF8StringPairRef(key, val)));
        self
    }

    pub(crate) fn packet_identifier(mut self, val: u16) -> Self {
        self.builder
            .packet_identifier(NonZero::try_from(val).unwrap());
        self
    }

    pub(crate) fn subscription_identifier(mut self, val: u32) -> Self {
        self.builder.subscription_identifier(
            VarSizeInt::try_from(val)
                .and_then(NonZero::try_from)
                .map(SubscriptionIdentifier::from)
                .unwrap(),
        );
        self
    }

    pub(crate) fn build(self) -> Result<SubscribeTx<'a>, CodecError> {
        self.builder.build()
    }
}

#[derive(Default)]
pub struct PublishOpts<'a> {
    pub(crate) qos: Option<QoS>,
    builder: PublishTxBuilder<'a>,
}

impl<'a> PublishOpts<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn retain(mut self, val: bool) -> Self {
        self.builder.retain(val);
        self
    }

    pub fn qos(mut self, val: QoS) -> Self {
        self.qos = Some(val);
        self.builder.qos(val);
        self
    }

    pub fn topic(mut self, val: &'a str) -> Self {
        self.builder.topic_name(UTF8StringRef(val));
        self
    }

    pub fn payload_format_indicator(mut self, val: bool) -> Self {
        self.builder
            .payload_format_indicator(PayloadFormatIndicator::from(val));
        self
    }

    pub fn topic_alias(mut self, val: u16) -> Self {
        self.builder
            .topic_alias(TopicAlias::from(NonZero::try_from(val).unwrap()));
        self
    }

    pub fn message_expiry_interval(mut self, val: u32) -> Self {
        self.builder
            .message_expiry_interval(MessageExpiryInterval::from(val));
        self
    }

    // pub fn subscription_identifier(mut self, val: u32) -> Self {
    //     self.builder.subscription_identifier(
    //         VarSizeInt::try_from(val)
    //             .and_then(NonZero::try_from)
    //             .map(SubscriptionIdentifier::from)
    //             .unwrap(),
    //     );
    //     self
    // }

    pub fn correlation_data(mut self, val: &'a [u8]) -> Self {
        self.builder
            .correlation_data(CorrelationDataRef::from(BinaryRef(val)));
        self
    }

    pub fn response_topic(mut self, val: &'a str) -> Self {
        self.builder
            .response_topic(ResponseTopicRef::from(UTF8StringRef(val)));
        self
    }

    pub fn content_type(mut self, val: &'a str) -> Self {
        self.builder
            .content_type(ContentTypeRef::from(UTF8StringRef(val)));
        self
    }

    pub fn user_property(mut self, (key, val): (&'a str, &'a str)) -> Self {
        self.builder
            .user_property(UserPropertyRef::from(UTF8StringPairRef(key, val)));
        self
    }

    pub fn data(mut self, val: &'a [u8]) -> Self {
        self.builder.payload(PayloadRef(val));
        self
    }

    pub(crate) fn packet_identifier(mut self, val: u16) -> Self {
        self.builder
            .packet_identifier(NonZero::try_from(val).unwrap());
        self
    }

    pub(crate) fn build(self) -> Result<PublishTx<'a>, CodecError> {
        self.builder.build()
    }
}

#[derive(Default)]
pub struct UnsubscribeOpts<'a> {
    builder: UnsubscribeTxBuilder<'a>,
}

impl<'a> UnsubscribeOpts<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn topic(mut self, val: &'a str) -> Self {
        self.builder.payload(UTF8StringRef(val));
        self
    }

    pub fn user_property(mut self, (key, val): (&'a str, &'a str)) -> Self {
        self.builder
            .user_property(UserPropertyRef::from(UTF8StringPairRef(key, val)));
        self
    }

    pub(crate) fn packet_identifier(mut self, val: u16) -> Self {
        self.builder
            .packet_identifier(NonZero::try_from(val).unwrap());
        self
    }

    pub(crate) fn build(self) -> Result<UnsubscribeTx<'a>, CodecError> {
        self.builder.build()
    }
}
