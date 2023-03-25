use crate::{
    codec::*,
    core::{base_types::*, error::CodecError, properties::*},
};
use core::time::Duration;

/// Connection options, represented as a consuming builder.
/// Used during [connection request](crate::Context::connect), translated to the CONNECT packet.
///
#[derive(Default)]
pub struct ConnectOpts<'a> {
    builder: ConnectTxBuilder<'a>,
}

impl<'a> ConnectOpts<'a> {
    /// Creates a new [ConnectOpts] instance.
    ///
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the client identifier.
    ///
    pub fn client_identifier(mut self, val: &'a str) -> Self {
        self.builder.client_identifier(UTF8StringRef(val));
        self
    }

    /// Sets the session keep alive.
    ///
    /// # Arguments
    /// `val` - [Duration] value less than [u16::MAX] in seconds.
    ///
    /// # Panics
    /// When the duration in seconds is greater than [u16::MAX].
    ///
    pub fn keep_alive(mut self, val: Duration) -> Self {
        self.builder
            .keep_alive(u16::try_from(val.as_secs()).unwrap());
        self
    }

    /// Sets the session expiry interval.
    ///
    /// # Arguments
    /// `val` - [Duration] value less than [u32::MAX] in seconds.
    ///
    /// # Panics
    /// When the duration in seconds is greater than [u32::MAX].
    ///
    pub fn session_expiry_interval(mut self, val: Duration) -> Self {
        self.builder
            .session_expiry_interval(SessionExpiryInterval::from(
                u32::try_from(val.as_secs()).unwrap(),
            ));
        self
    }

    /// Sets the maximum incoming QoS>0 publish messages handled at once.
    ///
    /// # Arguments
    /// `val` - value greater than 0
    ///
    /// # Panics
    /// When `val` equals 0.
    ///
    pub fn receive_maximum(mut self, val: u16) -> Self {
        self.builder
            .receive_maximum(ReceiveMaximum::from(NonZero::try_from(val).unwrap()));
        self
    }

    /// Sets the maximum packet size (in bytes).
    ///
    /// # Arguments
    /// `val` - value greater than 0
    ///
    /// # Panics
    /// When `val` equals 0.
    ///
    pub fn maximum_packet_size(mut self, val: u32) -> Self {
        self.builder
            .maximum_packet_size(MaximumPacketSize::from(NonZero::try_from(val).unwrap()));
        self
    }

    /// Sets the maximum accepted value of topic alias.
    ///
    pub fn topic_alias_maximum(mut self, val: u16) -> Self {
        self.builder
            .topic_alias_maximum(TopicAliasMaximum::from(val));
        self
    }

    /// Requests the broker to return response information in [ConnectRsp](super::rsp::ConnectRsp).
    ///
    pub fn request_response_information(mut self, val: bool) -> Self {
        self.builder
            .request_response_information(RequestResponseInformation::from(val));
        self
    }

    /// Requests the broker to return additional diagnostic data ([reason string](super::rsp::ConnectRsp::reason_string),
    /// [user properties](super::rsp::ConnectRsp::user_properties)) in [ConnectRsp](super::rsp::ConnectRsp).
    ///
    pub fn request_problem_information(mut self, val: bool) -> Self {
        self.builder
            .request_problem_information(RequestProblemInformation::from(val));
        self
    }

    /// Sets the name of the authentication method used for extended authorization.
    ///
    pub fn authentication_method(mut self, val: &'a str) -> Self {
        self.builder
            .authentication_method(AuthenticationMethodRef::from(UTF8StringRef(val)));
        self
    }

    /// Sets the binary authentication data. Note that setting authentication data without
    /// [authentication_method][ConnectOpts::authentication_method] set will result in an error.
    ///
    pub fn authentication_data(mut self, val: &'a [u8]) -> Self {
        self.builder
            .authentication_data(AuthenticationDataRef::from(BinaryRef(val)));
        self
    }

    /// Sets user properties as key-value pairs. Multiple user properties may be set.
    ///
    pub fn user_property(mut self, (key, val): (&'a str, &'a str)) -> Self {
        self.builder
            .user_property(UserPropertyRef::from(UTF8StringPairRef(key, val)));
        self
    }

    /// [QoS] used for will messages.
    ///
    pub fn will_qos(mut self, val: QoS) -> Self {
        self.builder.will_qos(val);
        self
    }

    /// Retain for will messages.
    ///
    pub fn will_retain(mut self, val: bool) -> Self {
        self.builder.will_retain(val);
        self
    }

    /// Clears the session upon connection.
    ///
    pub fn clean_start(mut self, val: bool) -> Self {
        self.builder.clean_start(val);
        self
    }

    /// Sets delay before publishing will messages.
    ///
    /// # Arguments
    /// `val` - [Duration] value less than [u32::MAX] in seconds.
    ///
    /// # Panics
    /// When the duration in seconds is greater than [u32::MAX].
    ///
    pub fn will_delay_interval(mut self, val: Duration) -> Self {
        self.builder.will_delay_interval(WillDelayInterval::from(
            u32::try_from(val.as_secs()).unwrap(),
        ));
        self
    }

    /// Sets payload format indicator for will messages.
    /// Value `false` indicates that the will payload is in unspecified format.
    /// Value `true` indicates that the payload is UTF8 encoded character data.
    ///
    pub fn will_payload_format_indicator(mut self, val: bool) -> Self {
        self.builder
            .will_payload_format_indicator(PayloadFormatIndicator::from(val));
        self
    }

    /// Sets the expiry interval of the will messages.
    ///
    /// # Arguments
    /// `val` - [Duration] value less than [u32::MAX] in seconds.
    ///
    /// # Panics
    /// When the duration in seconds is greater than [u32::MAX].
    ///
    pub fn will_message_expiry_interval(mut self, val: Duration) -> Self {
        self.builder
            .will_message_expiry_interval(MessageExpiryInterval::from(
                u32::try_from(val.as_secs()).unwrap(),
            ));
        self
    }

    /// Sets the content type of will messages.
    ///
    pub fn will_content_type(mut self, val: &'a str) -> Self {
        self.builder
            .will_content_type(ContentTypeRef::from(UTF8StringRef(val)));
        self
    }

    /// Sets the response topic for will messages.
    ///
    pub fn will_response_topic(mut self, val: &'a str) -> Self {
        self.builder
            .will_response_topic(ResponseTopicRef::from(UTF8StringRef(val)));
        self
    }

    /// Sets the correlation data for will messages.
    ///
    pub fn will_correlation_data(mut self, val: &'a [u8]) -> Self {
        self.builder
            .will_correlation_data(CorrelationDataRef::from(BinaryRef(val)));
        self
    }

    /// Sets user properties for will messages as key-value pairs. Multiple user properties may be set.
    ///
    pub fn will_user_property(mut self, (key, val): (&'a str, &'a str)) -> Self {
        self.builder
            .will_user_property(UserPropertyRef::from(UTF8StringPairRef(key, val)));
        self
    }

    /// Sets the topic for will messages.
    ///
    pub fn will_topic(mut self, val: &'a str) -> Self {
        self.builder.will_topic(UTF8StringRef(val));
        self
    }

    /// Sets the binary payload for will messages.
    ///
    pub fn will_payload(mut self, val: &'a [u8]) -> Self {
        self.builder.will_payload(BinaryRef(val));
        self
    }

    /// Sets the username for normal authorization.
    ///
    pub fn username(mut self, val: &'a str) -> Self {
        self.builder.username(UTF8StringRef(val));
        self
    }

    /// Sets the password for normal authorization.
    ///
    pub fn password(mut self, val: &'a [u8]) -> Self {
        self.builder.password(BinaryRef(val));
        self
    }

    pub(crate) fn build(self) -> Result<ConnectTx<'a>, CodecError> {
        self.builder.build()
    }
}

/// Authorization options, represented as a consuming builder.
/// Used during [extended authorization](crate::Context::authorize), translated to the AUTH packet.
///
#[derive(Default)]
pub struct AuthOpts<'a> {
    builder: AuthTxBuilder<'a>,
}

impl<'a> AuthOpts<'a> {
    /// Creates a new [AuthOpts] instance.
    ///
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a reason value.
    ///
    pub fn reason(mut self, val: AuthReason) -> Self {
        self.builder.reason(val);
        self
    }

    /// Sets a reason string property.
    ///
    pub fn reason_string(mut self, val: &'a str) {
        self.builder
            .reason_string(ReasonStringRef::from(UTF8StringRef(val)));
    }

    /// Sets the name of the authentication method used for extended authorization.
    ///
    pub fn authentication_method(mut self, val: &'a str) -> Self {
        self.builder
            .authentication_method(AuthenticationMethodRef::from(UTF8StringRef(val)));
        self
    }

    /// Sets the binary authentication data. Note that setting authentication data without
    /// [authentication_method][ConnectOpts::authentication_method] set will result in an error.
    ///
    pub fn authentication_data(mut self, val: &'a [u8]) -> Self {
        self.builder
            .authentication_data(AuthenticationDataRef::from(BinaryRef(val)));
        self
    }

    /// Sets user properties as key-value pairs. Multiple user properties may be set.
    ///
    pub fn user_property(mut self, (key, val): (&'a str, &'a str)) -> Self {
        self.builder
            .user_property(UserPropertyRef::from(UTF8StringPairRef(key, val)));
        self
    }

    pub(crate) fn build(self) -> Result<AuthTx<'a>, CodecError> {
        self.builder.build()
    }
}

/// Disconnection options, represented as a consuming builder.
/// Used during [disconnection request](super::handle::ContextHandle::disconnect), translated to the DISCONNECT packet.
///
#[derive(Default)]
pub struct DisconnectOpts<'a> {
    builder: DisconnectTxBuilder<'a>,
}

impl<'a> DisconnectOpts<'a> {
    /// Creates a new [DisconnectOpts] instance.
    ///
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a reason for disconnection.
    ///
    pub fn reason(mut self, reason: DisconnectReason) -> Self {
        self.builder.reason(reason);
        self
    }

    /// Sets session expiration interval.
    ///
    /// # Arguments
    /// `val` - [Duration] value less than [u32::MAX] in seconds.
    ///
    /// # Panics
    /// When the duration in seconds is greater than [u32::MAX].
    ///
    pub fn session_expiry_interval(mut self, val: Duration) -> Self {
        self.builder
            .session_expiry_interval(SessionExpiryInterval::from(
                u32::try_from(val.as_secs()).unwrap(),
            ));
        self
    }

    /// Sets a reason string property.
    ///
    pub fn reason_string(mut self, val: &'a str) -> Self {
        self.builder
            .reason_string(ReasonStringRef::from(UTF8StringRef(val)));
        self
    }

    /// Sets user properties. Multiple user properties may be set.
    ///
    pub fn user_property(mut self, (key, val): (&'a str, &'a str)) -> Self {
        self.builder
            .user_property(UserPropertyRef::from(UTF8StringPairRef(key, val)));
        self
    }

    pub(crate) fn build(self) -> Result<DisconnectTx<'a>, CodecError> {
        self.builder.build()
    }
}

/// Subscription options set for the topic filter.
///
#[derive(Copy, Clone, Default)]
pub struct SubscriptionOpts {
    opts: SubscriptionOptions,
}

impl SubscriptionOpts {
    /// Creates a new [SubscriptionOpts] instance.
    ///
    pub fn new() -> Self {
        Self::default()
    }

    /// Maximum Quality of Service for the topic.
    ///
    pub fn maximum_qos(mut self, val: QoS) -> Self {
        self.opts.maximum_qos = val;
        self
    }

    /// No local option.
    ///
    pub fn no_local(mut self, val: bool) -> Self {
        self.opts.no_local = val;
        self
    }

    /// Retain as published flag. Setting to `true` keeps the RETAIN flag from
    /// incoming PUBLISH packets untouched.
    ///
    pub fn retain_as_published(mut self, val: bool) -> Self {
        self.opts.retain_as_published = val;
        self
    }

    /// Retain handling options, see [RetainHandling].
    ///
    pub fn retain_handling(mut self, val: RetainHandling) -> Self {
        self.opts.retain_handling = val;
        self
    }

    pub(crate) fn build(self) -> SubscriptionOptions {
        self.opts
    }
}

/// Subscription options, represented as a consuming builder.
/// Used during [subscription request](super::handle::ContextHandle::subscribe), translated to the SUBSCRIBE packet.
/// Note that multiple topic filters may be supplied.
///
#[derive(Default)]
pub struct SubscribeOpts<'a> {
    builder: SubscribeTxBuilder<'a>,
}

impl<'a> SubscribeOpts<'a> {
    /// Creates a new [SubscribeOpts] instance.
    ///
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a new subscription with the given topic filter and options.
    /// Multiple subscriptions may be created.
    ///
    pub fn subscription(mut self, topic: &'a str, opts: SubscriptionOpts) -> Self {
        self.builder.payload((UTF8StringRef(topic), opts.build()));
        self
    }

    /// Sets user properties as key-value pairs. Multiple user properties may be set.
    ///
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

/// Publish options, represented as a consuming builder.
/// Used during [publish request](super::handle::ContextHandle::publish), translated to the PUBLISH packet.
///
#[derive(Default)]
pub struct PublishOpts<'a> {
    pub(crate) qos: Option<QoS>,
    builder: PublishTxBuilder<'a>,
}

impl<'a> PublishOpts<'a> {
    /// Creates a new [PublishOpts] instance.
    ///
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a retain flag.
    ///
    pub fn retain(mut self, val: bool) -> Self {
        self.builder.retain(val);
        self
    }

    /// Sets QoS level.
    ///
    pub fn qos(mut self, val: QoS) -> Self {
        self.qos = Some(val);
        self.builder.qos(val);
        self
    }

    /// Sets topic.
    ///
    pub fn topic_name(mut self, val: &'a str) -> Self {
        self.builder.topic_name(UTF8StringRef(val));
        self
    }

    /// Sets payload format indicator.
    /// Value `false` indicates that the will payload is in unspecified format.
    /// Value `true` indicates that the payload is UTF8 encoded character data.
    ///
    pub fn payload_format_indicator(mut self, val: bool) -> Self {
        self.builder
            .payload_format_indicator(PayloadFormatIndicator::from(val));
        self
    }

    /// Sets the topic alias.
    ///
    /// # Arguments
    /// `val` - value greater than 0
    ///
    /// # Panics
    /// When `val` equals 0.
    ///
    pub fn topic_alias(mut self, val: u16) -> Self {
        self.builder
            .topic_alias(TopicAlias::from(NonZero::try_from(val).unwrap()));
        self
    }

    /// Sets the expiry interval of the message.
    ///
    /// # Arguments
    /// `val` - [Duration] value less than [u32::MAX] in seconds.
    ///
    /// # Panics
    /// When the duration in seconds is greater than [u32::MAX].
    ///
    pub fn message_expiry_interval(mut self, val: Duration) -> Self {
        self.builder
            .message_expiry_interval(MessageExpiryInterval::from(
                u32::try_from(val.as_secs()).unwrap(),
            ));
        self
    }

    /// Sets correlation data.
    ///
    pub fn correlation_data(mut self, val: &'a [u8]) -> Self {
        self.builder
            .correlation_data(CorrelationDataRef::from(BinaryRef(val)));
        self
    }

    /// Sets response topic.
    ///
    pub fn response_topic(mut self, val: &'a str) -> Self {
        self.builder
            .response_topic(ResponseTopicRef::from(UTF8StringRef(val)));
        self
    }

    /// Sets message content type.
    ///
    pub fn content_type(mut self, val: &'a str) -> Self {
        self.builder
            .content_type(ContentTypeRef::from(UTF8StringRef(val)));
        self
    }

    /// Sets user properties as key-value pairs. Multiple user properties may be set.
    ///
    pub fn user_property(mut self, (key, val): (&'a str, &'a str)) -> Self {
        self.builder
            .user_property(UserPropertyRef::from(UTF8StringPairRef(key, val)));
        self
    }

    /// Sets message payload.
    ///
    pub fn payload(mut self, val: &'a [u8]) -> Self {
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

/// Unsubscribe options, represented as a consuming builder.
/// Used during [unsubscribe request](super::handle::ContextHandle::unsubscribe), translated to the UNSUBSCRIBE packet.
///
#[derive(Default)]
pub struct UnsubscribeOpts<'a> {
    builder: UnsubscribeTxBuilder<'a>,
}

impl<'a> UnsubscribeOpts<'a> {
    /// Creates a new [UnsubscribeOpts] instance.
    ///
    pub fn new() -> Self {
        Self::default()
    }

    /// Topic filter to unsubscribe from.
    pub fn topic_filter(mut self, val: &'a str) -> Self {
        self.builder.payload(UTF8StringRef(val));
        self
    }

    /// Sets user properties as key-value pairs. Multiple user properties may be set.
    ///
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
