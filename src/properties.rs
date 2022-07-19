use crate::{
    base_types::*,
    utils::{
        ByteWriter, SizedProperty, ToByteBuffer, TryFromBytes, TryFromIterator, TryToByteBuffer,
    },
};
use std::{convert::From, iter::Iterator, mem};

pub(crate) trait PropertyID {
    const PROPERTY_ID: u8;
}

const PAYLOAD_FORMAT_INDICATOR: u8 = 0x01;
const MESSAGE_EXPIRY_INTERVAL: u8 = 0x02;
const CONTENT_TYPE: u8 = 0x03;
const RESPONSE_TOPIC: u8 = 0x08;
const CORRELATION_DATA: u8 = 0x09;
const SUBSCRIPTION_IDENTIFIER: u8 = 0x0b;
const SESSION_EXPIRY_INTERVAL: u8 = 0x11;
const ASSIGNED_CLIENT_IDENTIFIER: u8 = 0x12;
const SERVER_KEEP_ALIVE: u8 = 0x13;
const AUTHENTICATION_METHOD: u8 = 0x15;
const AUTHENTICATION_DATA: u8 = 0x16;
const REQUEST_PROBLEM_INFORMATION: u8 = 0x17;
const WILL_DELAY_INTERVAL: u8 = 0x18;
const REQUEST_RESPONSE_INFORMATION: u8 = 0x19;
const RESPONSE_INFORMATION: u8 = 0x1a;
const SERVER_REFERENCE: u8 = 0x1c;
const REASON_STRING: u8 = 0x1f;
const RECEIVE_MAXIMUM: u8 = 0x21;
const TOPIC_ALIAS_MAXIMUM: u8 = 0x22;
const TOPIC_ALIAS: u8 = 0x23;
const MAXIMUM_QOS: u8 = 0x24;
const RETAIN_AVAILABLE: u8 = 0x25;
const USER_PROPERTY: u8 = 0x26;
const MAXIMUM_PACKET_SIZE: u8 = 0x27;
const WILDCARD_SUBSCRIPTION_AVAILABLE: u8 = 0x28;
const SUBSCRIPTION_IDENTIFIER_AVAILABLE: u8 = 0x29;
const SHARED_SUBSCRIPTION_AVAILABLE: u8 = 0x2a;

fn to_byte_buffer_unchecked<'a, PropertyT, UnderlyingT>(
    _: &PropertyT,
    underlying: &UnderlyingT,
    buf: &'a mut [u8],
) -> &'a [u8]
where
    PropertyT: PropertyID,
    UnderlyingT: ToByteBuffer,
{
    let mut writer = ByteWriter::from(buf);

    writer.write(&PropertyT::PROPERTY_ID);
    writer.write(underlying);

    buf
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PayloadFormatIndicator(pub(crate) Boolean);

impl PropertyID for PayloadFormatIndicator {
    const PROPERTY_ID: u8 = PAYLOAD_FORMAT_INDICATOR;
}

impl SizedProperty for PayloadFormatIndicator {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for PayloadFormatIndicator {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for PayloadFormatIndicator {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MessageExpiryInterval(pub(crate) FourByteInteger);

impl PropertyID for MessageExpiryInterval {
    const PROPERTY_ID: u8 = MESSAGE_EXPIRY_INTERVAL;
}

impl SizedProperty for MessageExpiryInterval {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for MessageExpiryInterval {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for MessageExpiryInterval {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ContentType(pub(crate) UTF8String);

impl PropertyID for ContentType {
    const PROPERTY_ID: u8 = CONTENT_TYPE;
}

impl SizedProperty for ContentType {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for ContentType {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for ContentType {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResponseTopic(pub(crate) UTF8String);

impl PropertyID for ResponseTopic {
    const PROPERTY_ID: u8 = RESPONSE_TOPIC;
}

impl SizedProperty for ResponseTopic {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for ResponseTopic {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for ResponseTopic {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CorrelationData(pub(crate) Binary);

impl PropertyID for CorrelationData {
    const PROPERTY_ID: u8 = CORRELATION_DATA;
}

impl SizedProperty for CorrelationData {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for CorrelationData {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for CorrelationData {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SubscriptionIdentifier(pub(crate) VarSizeInt);

impl PropertyID for SubscriptionIdentifier {
    const PROPERTY_ID: u8 = SUBSCRIPTION_IDENTIFIER;
}

impl SizedProperty for SubscriptionIdentifier {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for SubscriptionIdentifier {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for SubscriptionIdentifier {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SessionExpiryInterval(pub(crate) FourByteInteger);

impl PropertyID for SessionExpiryInterval {
    const PROPERTY_ID: u8 = SESSION_EXPIRY_INTERVAL;
}

impl SizedProperty for SessionExpiryInterval {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for SessionExpiryInterval {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for SessionExpiryInterval {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AssignedClientIdentifier(pub(crate) UTF8String);

impl PropertyID for AssignedClientIdentifier {
    const PROPERTY_ID: u8 = ASSIGNED_CLIENT_IDENTIFIER;
}

impl SizedProperty for AssignedClientIdentifier {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for AssignedClientIdentifier {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for AssignedClientIdentifier {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ServerKeepAlive(pub(crate) TwoByteInteger);

impl PropertyID for ServerKeepAlive {
    const PROPERTY_ID: u8 = SERVER_KEEP_ALIVE;
}

impl SizedProperty for ServerKeepAlive {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for ServerKeepAlive {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for ServerKeepAlive {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AuthenticationMethod(pub(crate) UTF8String);

impl PropertyID for AuthenticationMethod {
    const PROPERTY_ID: u8 = AUTHENTICATION_METHOD;
}

impl SizedProperty for AuthenticationMethod {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for AuthenticationMethod {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for AuthenticationMethod {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AuthenticationData(pub(crate) Binary);

impl PropertyID for AuthenticationData {
    const PROPERTY_ID: u8 = AUTHENTICATION_DATA;
}

impl SizedProperty for AuthenticationData {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for AuthenticationData {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for AuthenticationData {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RequestProblemInformation(pub(crate) Byte);

impl PropertyID for RequestProblemInformation {
    const PROPERTY_ID: u8 = REQUEST_PROBLEM_INFORMATION;
}

impl SizedProperty for RequestProblemInformation {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for RequestProblemInformation {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for RequestProblemInformation {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WillDelayInterval(pub(crate) FourByteInteger);

impl PropertyID for WillDelayInterval {
    const PROPERTY_ID: u8 = WILL_DELAY_INTERVAL;
}

impl SizedProperty for WillDelayInterval {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for WillDelayInterval {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for WillDelayInterval {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RequestResponseInformation(pub(crate) Byte);

impl PropertyID for RequestResponseInformation {
    const PROPERTY_ID: u8 = REQUEST_RESPONSE_INFORMATION;
}

impl SizedProperty for RequestResponseInformation {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for RequestResponseInformation {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for RequestResponseInformation {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResponseInformation(pub(crate) UTF8String);

impl PropertyID for ResponseInformation {
    const PROPERTY_ID: u8 = RESPONSE_INFORMATION;
}

impl SizedProperty for ResponseInformation {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for ResponseInformation {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for ResponseInformation {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ServerReference(pub(crate) UTF8String);

impl PropertyID for ServerReference {
    const PROPERTY_ID: u8 = SERVER_REFERENCE;
}

impl SizedProperty for ServerReference {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for ServerReference {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for ServerReference {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ReasonString(pub(crate) UTF8String);

impl PropertyID for ReasonString {
    const PROPERTY_ID: u8 = REASON_STRING;
}

impl SizedProperty for ReasonString {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for ReasonString {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for ReasonString {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ReceiveMaximum(pub(crate) TwoByteInteger);

impl PropertyID for ReceiveMaximum {
    const PROPERTY_ID: u8 = RECEIVE_MAXIMUM;
}

impl SizedProperty for ReceiveMaximum {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for ReceiveMaximum {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for ReceiveMaximum {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TopicAliasMaximum(pub(crate) TwoByteInteger);

impl PropertyID for TopicAliasMaximum {
    const PROPERTY_ID: u8 = TOPIC_ALIAS_MAXIMUM;
}

impl SizedProperty for TopicAliasMaximum {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for TopicAliasMaximum {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for TopicAliasMaximum {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TopicAlias(pub(crate) TwoByteInteger);

impl PropertyID for TopicAlias {
    const PROPERTY_ID: u8 = TOPIC_ALIAS;
}

impl SizedProperty for TopicAlias {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for TopicAlias {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for TopicAlias {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MaximumQoS(pub(crate) QoS);

impl PropertyID for MaximumQoS {
    const PROPERTY_ID: u8 = MAXIMUM_QOS;
}

impl SizedProperty for MaximumQoS {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for MaximumQoS {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for MaximumQoS {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RetainAvailable(pub(crate) Boolean);

impl PropertyID for RetainAvailable {
    const PROPERTY_ID: u8 = RETAIN_AVAILABLE;
}

impl SizedProperty for RetainAvailable {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for RetainAvailable {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for RetainAvailable {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UserProperty(pub(crate) UTF8StringPair);

impl PropertyID for UserProperty {
    const PROPERTY_ID: u8 = USER_PROPERTY;
}

impl SizedProperty for UserProperty {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for UserProperty {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for UserProperty {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MaximumPacketSize(pub(crate) FourByteInteger);

impl PropertyID for MaximumPacketSize {
    const PROPERTY_ID: u8 = MAXIMUM_PACKET_SIZE;
}

impl SizedProperty for MaximumPacketSize {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for MaximumPacketSize {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for MaximumPacketSize {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WildcardSubscriptionAvailable(pub(crate) Boolean);

impl PropertyID for WildcardSubscriptionAvailable {
    const PROPERTY_ID: u8 = WILDCARD_SUBSCRIPTION_AVAILABLE;
}

impl SizedProperty for WildcardSubscriptionAvailable {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for WildcardSubscriptionAvailable {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for WildcardSubscriptionAvailable {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SubscriptionIdentifierAvailable(pub(crate) Boolean);

impl PropertyID for SubscriptionIdentifierAvailable {
    const PROPERTY_ID: u8 = SUBSCRIPTION_IDENTIFIER_AVAILABLE;
}

impl SizedProperty for SubscriptionIdentifierAvailable {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for SubscriptionIdentifierAvailable {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for SubscriptionIdentifierAvailable {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SharedSubscriptionAvailable(pub(crate) Boolean);

impl PropertyID for SharedSubscriptionAvailable {
    const PROPERTY_ID: u8 = SHARED_SUBSCRIPTION_AVAILABLE;
}

impl SizedProperty for SharedSubscriptionAvailable {
    fn property_len(&self) -> usize {
        mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
    }
}

impl ToByteBuffer for SharedSubscriptionAvailable {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        to_byte_buffer_unchecked(self, &self.0, result)
    }
}

impl TryToByteBuffer for SharedSubscriptionAvailable {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        Some(to_byte_buffer_unchecked(self, &self.0, result))
    }
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum Property {
    PayloadFormatIndicator(PayloadFormatIndicator),
    MessageExpiryInterval(MessageExpiryInterval),
    ContentType(ContentType),
    ResponseTopic(ResponseTopic),
    CorrelationData(CorrelationData),
    SubscriptionIdentifier(SubscriptionIdentifier),
    SessionExpiryInterval(SessionExpiryInterval),
    AssignedClientIdentifier(AssignedClientIdentifier),
    ServerKeepAlive(ServerKeepAlive),
    AuthenticationMethod(AuthenticationMethod),
    AuthenticationData(AuthenticationData),
    RequestProblemInformation(RequestProblemInformation),
    WillDelayInterval(WillDelayInterval),
    RequestResponseInformation(RequestResponseInformation),
    ResponseInformation(ResponseInformation),
    ServerReference(ServerReference),
    ReasonString(ReasonString),
    ReceiveMaximum(ReceiveMaximum),
    TopicAliasMaximum(TopicAliasMaximum),
    TopicAlias(TopicAlias),
    MaximumQoS(MaximumQoS),
    RetainAvailable(RetainAvailable),
    UserProperty(UserProperty),
    MaximumPacketSize(MaximumPacketSize),
    WildcardSubscriptionAvailable(WildcardSubscriptionAvailable),
    SubscriptionIdentifierAvailable(SubscriptionIdentifierAvailable),
    SharedSubscriptionAvailable(SharedSubscriptionAvailable),
}

pub(crate) struct PropertyIterator<'a> {
    buf: &'a [u8],
}

impl<'a> From<&'a [u8]> for PropertyIterator<'a> {
    fn from(buf: &'a [u8]) -> Self {
        Self { buf }
    }
}

impl<'a> Iterator for PropertyIterator<'a> {
    type Item = Property;

    fn next(&mut self) -> Option<Self::Item> {
        let id_var = VarSizeInt::try_from_iter(self.buf.iter().copied())?;
        if id_var.len() != 1 {
            return None;
        }

        let (_, remaining) = self.buf.split_at(id_var.len());
        self.buf = remaining;

        return match u8::from(id_var) {
            PAYLOAD_FORMAT_INDICATOR => {
                let property = Boolean::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::PayloadFormatIndicator(PayloadFormatIndicator(
                    property,
                )))
            }
            REQUEST_RESPONSE_INFORMATION => {
                let property = Byte::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::RequestResponseInformation(
                    RequestResponseInformation(property),
                ))
            }
            WILDCARD_SUBSCRIPTION_AVAILABLE => {
                let property = Boolean::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::WildcardSubscriptionAvailable(
                    WildcardSubscriptionAvailable(property),
                ))
            }
            SUBSCRIPTION_IDENTIFIER_AVAILABLE => {
                let property = Boolean::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::SubscriptionIdentifierAvailable(
                    SubscriptionIdentifierAvailable(property),
                ))
            }
            SHARED_SUBSCRIPTION_AVAILABLE => {
                let property = Boolean::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::SharedSubscriptionAvailable(
                    SharedSubscriptionAvailable(property),
                ))
            }
            MAXIMUM_QOS => {
                let property = QoS::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::MaximumQoS(MaximumQoS(property)))
            }
            RETAIN_AVAILABLE => {
                let property = Boolean::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::RetainAvailable(RetainAvailable(property)))
            }
            REQUEST_PROBLEM_INFORMATION => {
                let property = Byte::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::RequestProblemInformation(
                    RequestProblemInformation(property),
                ))
            }
            SERVER_KEEP_ALIVE => {
                let property = TwoByteInteger::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::ServerKeepAlive(ServerKeepAlive(property)))
            }
            RECEIVE_MAXIMUM => {
                let property = TwoByteInteger::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::ReceiveMaximum(ReceiveMaximum(property)))
            }
            TOPIC_ALIAS_MAXIMUM => {
                let property = TwoByteInteger::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::TopicAliasMaximum(TopicAliasMaximum(property)))
            }
            TOPIC_ALIAS => {
                let property = TwoByteInteger::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::TopicAlias(TopicAlias(property)))
            }
            MESSAGE_EXPIRY_INTERVAL => {
                let property = FourByteInteger::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::MessageExpiryInterval(MessageExpiryInterval(
                    property,
                )))
            }
            SESSION_EXPIRY_INTERVAL => {
                let property = FourByteInteger::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::SessionExpiryInterval(SessionExpiryInterval(
                    property,
                )))
            }
            WILL_DELAY_INTERVAL => {
                let property = FourByteInteger::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::WillDelayInterval(WillDelayInterval(property)))
            }
            MAXIMUM_PACKET_SIZE => {
                let property = FourByteInteger::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::MaximumPacketSize(MaximumPacketSize(property)))
            }
            SUBSCRIPTION_IDENTIFIER => {
                let property = VarSizeInt::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::SubscriptionIdentifier(SubscriptionIdentifier(
                    property,
                )))
            }
            CORRELATION_DATA => {
                let property = Binary::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::CorrelationData(CorrelationData(property)))
            }
            CONTENT_TYPE => {
                let property = UTF8String::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::ContentType(ContentType(property)))
            }
            RESPONSE_TOPIC => {
                let property = UTF8String::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::ResponseTopic(ResponseTopic(property)))
            }
            ASSIGNED_CLIENT_IDENTIFIER => {
                let property = UTF8String::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::AssignedClientIdentifier(
                    AssignedClientIdentifier(property),
                ))
            }
            AUTHENTICATION_METHOD => {
                let property = UTF8String::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::AuthenticationMethod(AuthenticationMethod(
                    property,
                )))
            }
            AUTHENTICATION_DATA => {
                let property = Binary::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::AuthenticationData(AuthenticationData(property)))
            }
            RESPONSE_INFORMATION => {
                let property = UTF8String::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::ResponseInformation(ResponseInformation(property)))
            }
            SERVER_REFERENCE => {
                let property = UTF8String::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::ServerReference(ServerReference(property)))
            }
            REASON_STRING => {
                let property = UTF8String::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::ReasonString(ReasonString(property)))
            }
            USER_PROPERTY => {
                let property = UTF8StringPair::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::UserProperty(UserProperty(property)))
            }
            _ => None,
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod property_iterator {
        use super::*;

        #[test]
        fn single_byte() {
            const EXPECTED_VAL: u8 = 1;
            let input = [
                (
                    PayloadFormatIndicator::PROPERTY_ID,
                    Property::PayloadFormatIndicator(PayloadFormatIndicator(EXPECTED_VAL != 0)),
                ),
                (
                    RequestResponseInformation::PROPERTY_ID,
                    Property::RequestResponseInformation(RequestResponseInformation(EXPECTED_VAL)),
                ),
                (
                    WildcardSubscriptionAvailable::PROPERTY_ID,
                    Property::WildcardSubscriptionAvailable(WildcardSubscriptionAvailable(
                        EXPECTED_VAL != 0,
                    )),
                ),
                (
                    SubscriptionIdentifierAvailable::PROPERTY_ID,
                    Property::SubscriptionIdentifierAvailable(SubscriptionIdentifierAvailable(
                        EXPECTED_VAL != 0,
                    )),
                ),
                (
                    SharedSubscriptionAvailable::PROPERTY_ID,
                    Property::SharedSubscriptionAvailable(SharedSubscriptionAvailable(
                        EXPECTED_VAL != 0,
                    )),
                ),
                (
                    MaximumQoS::PROPERTY_ID,
                    Property::MaximumQoS(MaximumQoS(QoS::try_from(EXPECTED_VAL).unwrap())),
                ),
                (
                    RetainAvailable::PROPERTY_ID,
                    Property::RetainAvailable(RetainAvailable(EXPECTED_VAL != 0)),
                ),
                (
                    RequestProblemInformation::PROPERTY_ID,
                    Property::RequestProblemInformation(RequestProblemInformation(EXPECTED_VAL)),
                ),
            ];

            for (id, expected) in input {
                let buf = [id, EXPECTED_VAL];
                let mut iter = PropertyIterator::from(&buf[..]);
                let property = iter.next().unwrap();
                assert_eq!(property, expected);
            }
        }

        #[test]
        fn two_byte_int() {
            const EXPECTED_VAL: u16 = 965;
            let input = [
                (
                    ServerKeepAlive::PROPERTY_ID,
                    Property::ServerKeepAlive(ServerKeepAlive(EXPECTED_VAL)),
                ),
                (
                    ReceiveMaximum::PROPERTY_ID,
                    Property::ReceiveMaximum(ReceiveMaximum(EXPECTED_VAL)),
                ),
                (
                    TopicAliasMaximum::PROPERTY_ID,
                    Property::TopicAliasMaximum(TopicAliasMaximum(EXPECTED_VAL)),
                ),
                (
                    TopicAlias::PROPERTY_ID,
                    Property::TopicAlias(TopicAlias(EXPECTED_VAL)),
                ),
            ];

            for (id, expected) in input {
                let buf = [id, (EXPECTED_VAL >> 8) as u8, EXPECTED_VAL as u8];
                let mut iter = PropertyIterator::from(&buf[..]);
                assert_eq!(iter.next().unwrap(), expected);
            }
        }

        #[test]
        fn four_byte_int() {
            const EXPECTED_VAL: u32 = 44568;
            let input = [
                (
                    MessageExpiryInterval::PROPERTY_ID,
                    Property::MessageExpiryInterval(MessageExpiryInterval(EXPECTED_VAL)),
                ),
                (
                    SessionExpiryInterval::PROPERTY_ID,
                    Property::SessionExpiryInterval(SessionExpiryInterval(EXPECTED_VAL)),
                ),
                (
                    WillDelayInterval::PROPERTY_ID,
                    Property::WillDelayInterval(WillDelayInterval(EXPECTED_VAL)),
                ),
                (
                    MaximumPacketSize::PROPERTY_ID,
                    Property::MaximumPacketSize(MaximumPacketSize(EXPECTED_VAL)),
                ),
            ];

            for (id, expected) in input {
                let buf = [id, 0x00, 0x00, 0xae, 0x18];
                let mut iter = PropertyIterator::from(&buf[..]);
                assert_eq!(iter.next().unwrap(), expected);
            }
        }

        #[test]
        fn var_size_int() {
            const EXPECTED_VAL: u8 = 64;
            let input = [(
                SubscriptionIdentifier::PROPERTY_ID,
                Property::SubscriptionIdentifier(SubscriptionIdentifier(VarSizeInt::from(
                    EXPECTED_VAL,
                ))),
            )];

            for (id, expected) in input {
                let buf = [id, EXPECTED_VAL];
                let mut iter = PropertyIterator::from(&buf[..]);
                assert_eq!(iter.next().unwrap(), expected);
            }
        }

        #[test]
        fn binary() {
            const EXPECTED_VAL: [u8; 3] = [0x02u8, 0xae, 0x18];
            let input = [
                (
                    &[
                        &((EXPECTED_VAL.len() as u16).to_be_bytes()),
                        &EXPECTED_VAL[..],
                    ]
                    .concat(),
                    CorrelationData::PROPERTY_ID,
                    Property::CorrelationData(CorrelationData(Vec::from(EXPECTED_VAL))),
                ),
                (
                    &[
                        &((EXPECTED_VAL.len() as u16).to_be_bytes()),
                        &EXPECTED_VAL[..],
                    ]
                    .concat(),
                    AuthenticationData::PROPERTY_ID,
                    Property::AuthenticationData(AuthenticationData(Vec::from(EXPECTED_VAL))),
                ),
            ];

            for (buf, id, expected) in input {
                let buf = [&[id], &buf[..]].concat();
                let mut iter = PropertyIterator::from(&buf[..]);
                assert_eq!(iter.next().unwrap(), expected);
            }
        }

        #[test]
        fn utf8_string() {
            const EXPECTED_VAL: [u8; 3] = [b'v', b'a', b'l'];
            let input = [
                (
                    &[
                        &((EXPECTED_VAL.len() as u16).to_be_bytes()),
                        &EXPECTED_VAL[..],
                    ]
                    .concat(),
                    ContentType::PROPERTY_ID,
                    Property::ContentType(ContentType(
                        String::from_utf8(Vec::from(EXPECTED_VAL)).unwrap(),
                    )),
                ),
                (
                    &[
                        &((EXPECTED_VAL.len() as u16).to_be_bytes()),
                        &EXPECTED_VAL[..],
                    ]
                    .concat(),
                    ResponseTopic::PROPERTY_ID,
                    Property::ResponseTopic(ResponseTopic(
                        String::from_utf8(Vec::from(EXPECTED_VAL)).unwrap(),
                    )),
                ),
                (
                    &[
                        &((EXPECTED_VAL.len() as u16).to_be_bytes()),
                        &EXPECTED_VAL[..],
                    ]
                    .concat(),
                    AssignedClientIdentifier::PROPERTY_ID,
                    Property::AssignedClientIdentifier(AssignedClientIdentifier(
                        String::from_utf8(Vec::from(EXPECTED_VAL)).unwrap(),
                    )),
                ),
                (
                    &[
                        &((EXPECTED_VAL.len() as u16).to_be_bytes()),
                        &EXPECTED_VAL[..],
                    ]
                    .concat(),
                    AuthenticationMethod::PROPERTY_ID,
                    Property::AuthenticationMethod(AuthenticationMethod(
                        String::from_utf8(Vec::from(EXPECTED_VAL)).unwrap(),
                    )),
                ),
                (
                    &[
                        &((EXPECTED_VAL.len() as u16).to_be_bytes()),
                        &EXPECTED_VAL[..],
                    ]
                    .concat(),
                    ResponseInformation::PROPERTY_ID,
                    Property::ResponseInformation(ResponseInformation(
                        String::from_utf8(Vec::from(EXPECTED_VAL)).unwrap(),
                    )),
                ),
                (
                    &[
                        &((EXPECTED_VAL.len() as u16).to_be_bytes()),
                        &EXPECTED_VAL[..],
                    ]
                    .concat(),
                    ServerReference::PROPERTY_ID,
                    Property::ServerReference(ServerReference(
                        String::from_utf8(Vec::from(EXPECTED_VAL)).unwrap(),
                    )),
                ),
                (
                    &[
                        &((EXPECTED_VAL.len() as u16).to_be_bytes()),
                        &EXPECTED_VAL[..],
                    ]
                    .concat(),
                    ReasonString::PROPERTY_ID,
                    Property::ReasonString(ReasonString(
                        String::from_utf8(Vec::from(EXPECTED_VAL)).unwrap(),
                    )),
                ),
            ];

            for (buf, id, expected) in input {
                let buf = [&[id], &buf[..]].concat();
                let mut iter = PropertyIterator::from(&buf[..]);
                assert_eq!(iter.next().unwrap(), expected);
            }
        }

        #[test]
        fn utf8_string_pair() {
            const EXPECTED_KEY: &str = "key";
            const EXPECTED_VAL: &str = "val";
            const ID: u8 = USER_PROPERTY;
            const INPUT: [u8; 11] = [ID, 0, 3, b'k', b'e', b'y', 0, 3, b'v', b'a', b'l'];
            let mut iter = PropertyIterator::from(&INPUT[..]);
            let property = iter.next().unwrap();

            match property {
                Property::UserProperty(result) => {
                    assert_eq!(result.property_len(), INPUT.len());
                    let (key, val) = result.0;
                    assert_eq!(key, EXPECTED_KEY);
                    assert_eq!(val, EXPECTED_VAL);
                }
                _ => panic!(),
            }
        }
    }

    mod to_bytes {
        use super::*;

        fn byte_test<T>(property: T, expected: Byte)
        where
            T: SizedProperty + PropertyID + TryToByteBuffer,
        {
            let mut buf = [0u8; 2];
            let result = property.try_to_byte_buffer(&mut buf);
            assert!(result.is_some());
            assert_eq!(result.unwrap(), [T::PROPERTY_ID, expected]);
        }

        fn two_byte_int_test<T>(property: T, expected: TwoByteInteger)
        where
            T: SizedProperty + PropertyID + TryToByteBuffer,
        {
            let mut buf = [0u8; 3];
            let result = property.try_to_byte_buffer(&mut buf);
            assert!(result.is_some());
            assert_eq!(
                result.unwrap(),
                [&[T::PROPERTY_ID], &expected.to_be_bytes()[..]].concat()
            );
        }

        fn four_byte_int_test<T>(property: T, expected: FourByteInteger)
        where
            T: SizedProperty + PropertyID + TryToByteBuffer,
        {
            let mut buf = [0u8; 5];
            let result = property.try_to_byte_buffer(&mut buf);
            assert!(result.is_some());
            assert_eq!(
                result.unwrap(),
                [&[T::PROPERTY_ID], &expected.to_be_bytes()[..]].concat()
            );
        }

        fn utf8_string_test<T>(property: T, expected: Vec<u8>)
        where
            T: SizedProperty + PropertyID + TryToByteBuffer,
        {
            let mut buf = [0u8; 6];
            let result = property.try_to_byte_buffer(&mut buf);
            assert!(result.is_some());
            assert_eq!(result.unwrap(), [&[T::PROPERTY_ID], &expected[..]].concat());
        }

        #[test]
        fn byte() {
            const EXPECTED_VAL: u8 = 1;

            byte_test(PayloadFormatIndicator(EXPECTED_VAL != 0), EXPECTED_VAL);
            byte_test(RequestResponseInformation(EXPECTED_VAL), EXPECTED_VAL);
            byte_test(
                WildcardSubscriptionAvailable(EXPECTED_VAL != 0),
                EXPECTED_VAL,
            );
            byte_test(
                SubscriptionIdentifierAvailable(EXPECTED_VAL != 0),
                EXPECTED_VAL,
            );
            byte_test(SharedSubscriptionAvailable(EXPECTED_VAL != 0), EXPECTED_VAL);
            byte_test(
                MaximumQoS(QoS::try_from(EXPECTED_VAL).unwrap()),
                EXPECTED_VAL,
            );
            byte_test(RetainAvailable(EXPECTED_VAL != 0), EXPECTED_VAL);
            byte_test(RequestProblemInformation(EXPECTED_VAL), EXPECTED_VAL);
        }

        #[test]
        fn two_byte_int() {
            const EXPECTED_VAL: u16 = 0x1234;

            two_byte_int_test(ServerKeepAlive(EXPECTED_VAL), EXPECTED_VAL);
            two_byte_int_test(ReceiveMaximum(EXPECTED_VAL), EXPECTED_VAL);
            two_byte_int_test(TopicAliasMaximum(EXPECTED_VAL), EXPECTED_VAL);
            two_byte_int_test(TopicAlias(EXPECTED_VAL), EXPECTED_VAL);
        }

        #[test]
        fn four_byte_int() {
            const EXPECTED_VAL: u32 = 0x12345678;

            four_byte_int_test(MessageExpiryInterval(EXPECTED_VAL), EXPECTED_VAL);
            four_byte_int_test(SessionExpiryInterval(EXPECTED_VAL), EXPECTED_VAL);
            four_byte_int_test(WillDelayInterval(EXPECTED_VAL), EXPECTED_VAL);
            four_byte_int_test(MaximumPacketSize(EXPECTED_VAL), EXPECTED_VAL);
        }

        #[test]
        fn var_size_int() {
            const INPUT_VAL: u16 = 16383;
            const EXPECTED_BUF: &[u8] = &[0xff, 0x7f];

            let mut buf = [0u8; 5];
            let result =
                SubscriptionIdentifier(VarSizeInt::from(INPUT_VAL)).try_to_byte_buffer(&mut buf);
            assert!(result.is_some());
            assert_eq!(
                result.unwrap(),
                [&[SubscriptionIdentifier::PROPERTY_ID], EXPECTED_BUF].concat()
            );
        }

        #[test]
        fn utf8_string() {
            const INPUT_VAL: &str = "val";
            const EXPECTED_BUF: [u8; 5] = [0, 3, b'v', b'a', b'l'];

            utf8_string_test(
                ContentType(String::from(INPUT_VAL)),
                Vec::from(EXPECTED_BUF),
            );
            utf8_string_test(
                ResponseTopic(String::from(INPUT_VAL)),
                Vec::from(EXPECTED_BUF),
            );
            utf8_string_test(
                AssignedClientIdentifier(String::from(INPUT_VAL)),
                Vec::from(EXPECTED_BUF),
            );
            utf8_string_test(
                AuthenticationMethod(String::from(INPUT_VAL)),
                Vec::from(EXPECTED_BUF),
            );
            utf8_string_test(
                ResponseInformation(String::from(INPUT_VAL)),
                Vec::from(EXPECTED_BUF),
            );
            utf8_string_test(
                ServerReference(String::from(INPUT_VAL)),
                Vec::from(EXPECTED_BUF),
            );
            utf8_string_test(
                ReasonString(String::from(INPUT_VAL)),
                Vec::from(EXPECTED_BUF),
            );
        }

        #[test]
        fn utf8_string_pair() {
            const INPUT_KEY: &str = "key";
            const INPUT_VAL: &str = "val";
            const EXPECTED_BUF: [u8; 11] = [
                USER_PROPERTY,
                0,
                3,
                b'k',
                b'e',
                b'y',
                0,
                3,
                b'v',
                b'a',
                b'l',
            ];

            let mut buf = [0u8; 11];
            let property = UserProperty((String::from(INPUT_KEY), String::from(INPUT_VAL)));

            assert_eq!(property.try_to_byte_buffer(&mut buf).unwrap(), EXPECTED_BUF);
        }
    }
}
