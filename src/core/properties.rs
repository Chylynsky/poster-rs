use crate::core::{
    base_types::*,
    error::{ConversionError, InsufficientBufferSize, InvalidPropertyId, PropertyError},
    utils::{
        ByteWriter, PropertyID, SizedProperty, ToByteBuffer, TryFromBytes, TryFromIterator,
        TryToByteBuffer,
    },
};
use core::{convert::From, fmt, iter::Iterator, mem};

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

macro_rules! declare_property {
    ($property_name:ident, $property_type:ty, $property_id:literal) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $property_name($property_type);

        impl PropertyID for $property_name {
            const PROPERTY_ID: u8 = $property_id;
        }

        impl SizedProperty for $property_name {
            fn property_len(&self) -> usize {
                mem::size_of_val(&Self::PROPERTY_ID) + self.0.property_len()
            }
        }

        impl ToByteBuffer for $property_name {
            fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
                let result = &mut buf[0..self.property_len()];
                to_byte_buffer_unchecked(self, &self.0, result)
            }
        }

        impl TryToByteBuffer for $property_name {
            type Error = ConversionError;

            fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Result<&'a [u8], Self::Error> {
                let result = buf
                    .get_mut(0..self.property_len())
                    .ok_or(InsufficientBufferSize)?;
                Ok(to_byte_buffer_unchecked(self, &self.0, result))
            }
        }

        impl From<$property_type> for $property_name {
            fn from(val: $property_type) -> Self {
                Self { 0: val }
            }
        }

        impl From<$property_name> for $property_type {
            fn from(val: $property_name) -> $property_type {
                val.0
            }
        }
    };
}

declare_property!(PayloadFormatIndicator, bool, 1);
declare_property!(MessageExpiryInterval, u32, 2);
declare_property!(ContentType, String, 3);
declare_property!(ResponseTopic, String, 8);
declare_property!(CorrelationData, Binary, 9);
declare_property!(SubscriptionIdentifier, NonZero<VarSizeInt>, 11);
declare_property!(SessionExpiryInterval, u32, 17);

#[allow(clippy::derivable_impls)]
impl Default for SessionExpiryInterval {
    fn default() -> Self {
        Self(0)
    }
}

declare_property!(AssignedClientIdentifier, String, 18);
declare_property!(ServerKeepAlive, u16, 19);
declare_property!(AuthenticationMethod, String, 21);
declare_property!(AuthenticationData, Binary, 22);
declare_property!(RequestProblemInformation, bool, 23);

declare_property!(WillDelayInterval, u32, 24);

#[allow(clippy::derivable_impls)]
impl Default for WillDelayInterval {
    fn default() -> Self {
        Self(0)
    }
}

declare_property!(RequestResponseInformation, bool, 25);
declare_property!(ResponseInformation, String, 26);
declare_property!(ServerReference, String, 28);
declare_property!(ReasonString, String, 31);
declare_property!(ReceiveMaximum, NonZero<u16>, 33);

impl Default for ReceiveMaximum {
    fn default() -> Self {
        Self(NonZero::from(65535))
    }
}

declare_property!(TopicAliasMaximum, u16, 34);

#[allow(clippy::derivable_impls)]
impl Default for TopicAliasMaximum {
    fn default() -> Self {
        Self(0)
    }
}

declare_property!(TopicAlias, NonZero<u16>, 35);
declare_property!(MaximumQoS, QoS, 36);

impl Default for MaximumQoS {
    fn default() -> Self {
        Self(QoS::ExactlyOnce)
    }
}

declare_property!(RetainAvailable, bool, 37);

impl Default for RetainAvailable {
    fn default() -> Self {
        Self(true)
    }
}

declare_property!(UserProperty, StringPair, 38);
declare_property!(MaximumPacketSize, NonZero<u32>, 39);
declare_property!(WildcardSubscriptionAvailable, bool, 40);

impl Default for WildcardSubscriptionAvailable {
    fn default() -> Self {
        Self(true)
    }
}

declare_property!(SubscriptionIdentifierAvailable, bool, 41);

impl Default for SubscriptionIdentifierAvailable {
    fn default() -> Self {
        Self(true)
    }
}

declare_property!(SharedSubscriptionAvailable, bool, 42);

impl Default for SharedSubscriptionAvailable {
    fn default() -> Self {
        Self(true)
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
    type Item = Result<Property, PropertyError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Inability to read the packet ID means the iteration is over.
        let id = VarSizeInt::try_from_bytes(self.buf)
            .ok()
            .filter(|val| val.len() == 1)?;
        self.buf = self.buf.get(id.len()..)?;

        return match u8::from(id) {
            PayloadFormatIndicator::PROPERTY_ID => {
                let result = bool::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::PayloadFormatIndicator(
                    PayloadFormatIndicator(val),
                )))
            }
            RequestResponseInformation::PROPERTY_ID => {
                let result = bool::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::RequestResponseInformation(
                    RequestResponseInformation(val),
                )))
            }
            WildcardSubscriptionAvailable::PROPERTY_ID => {
                let result = bool::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::WildcardSubscriptionAvailable(
                    WildcardSubscriptionAvailable(val),
                )))
            }
            SubscriptionIdentifierAvailable::PROPERTY_ID => {
                let result = bool::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::SubscriptionIdentifierAvailable(
                    SubscriptionIdentifierAvailable(val),
                )))
            }
            SharedSubscriptionAvailable::PROPERTY_ID => {
                let result = bool::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::SharedSubscriptionAvailable(
                    SharedSubscriptionAvailable(val),
                )))
            }
            RetainAvailable::PROPERTY_ID => {
                let result = bool::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::RetainAvailable(RetainAvailable(val))))
            }
            RequestProblemInformation::PROPERTY_ID => {
                let result = bool::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::RequestProblemInformation(
                    RequestProblemInformation(val),
                )))
            }
            MaximumQoS::PROPERTY_ID => {
                let result = QoS::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::MaximumQoS(MaximumQoS(val))))
            }
            ServerKeepAlive::PROPERTY_ID => {
                let result = u16::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::ServerKeepAlive(ServerKeepAlive(val))))
            }

            TopicAliasMaximum::PROPERTY_ID => {
                let result = u16::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::TopicAliasMaximum(TopicAliasMaximum(val))))
            }
            ReceiveMaximum::PROPERTY_ID => {
                let result = NonZero::<u16>::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::ReceiveMaximum(ReceiveMaximum(val))))
            }
            TopicAlias::PROPERTY_ID => {
                let result = NonZero::<u16>::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::TopicAlias(TopicAlias(val))))
            }
            MessageExpiryInterval::PROPERTY_ID => {
                let result = u32::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::MessageExpiryInterval(MessageExpiryInterval(
                    val,
                ))))
            }
            SessionExpiryInterval::PROPERTY_ID => {
                let result = u32::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::SessionExpiryInterval(SessionExpiryInterval(
                    val,
                ))))
            }
            WillDelayInterval::PROPERTY_ID => {
                let result = u32::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::WillDelayInterval(WillDelayInterval(val))))
            }
            MaximumPacketSize::PROPERTY_ID => {
                let result = NonZero::<u32>::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::MaximumPacketSize(MaximumPacketSize(val))))
            }
            SubscriptionIdentifier::PROPERTY_ID => {
                let result = NonZero::<VarSizeInt>::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::SubscriptionIdentifier(
                    SubscriptionIdentifier(val),
                )))
            }
            CorrelationData::PROPERTY_ID => {
                let result = Binary::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::CorrelationData(CorrelationData(val))))
            }
            AuthenticationData::PROPERTY_ID => {
                let result = Binary::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::AuthenticationData(AuthenticationData(val))))
            }
            ContentType::PROPERTY_ID => {
                let result = String::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::ContentType(ContentType(val))))
            }
            ResponseTopic::PROPERTY_ID => {
                let result = String::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::ResponseTopic(ResponseTopic(val))))
            }
            AssignedClientIdentifier::PROPERTY_ID => {
                let result = String::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::AssignedClientIdentifier(
                    AssignedClientIdentifier(val),
                )))
            }
            AuthenticationMethod::PROPERTY_ID => {
                let result = String::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::AuthenticationMethod(AuthenticationMethod(
                    val,
                ))))
            }
            ResponseInformation::PROPERTY_ID => {
                let result = String::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::ResponseInformation(ResponseInformation(val))))
            }
            ServerReference::PROPERTY_ID => {
                let result = String::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::ServerReference(ServerReference(val))))
            }
            ReasonString::PROPERTY_ID => {
                let result = String::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::ReasonString(ReasonString(val))))
            }
            UserProperty::PROPERTY_ID => {
                let result = StringPair::try_from_bytes(self.buf);
                if result.is_err() {
                    return Some(Err(result.unwrap_err().into()));
                }

                let val = result.unwrap();
                self.buf = self.buf.get(val.property_len()..)?;

                Some(Ok(Property::UserProperty(UserProperty(val))))
            }
            _ => Some(Err(InvalidPropertyId.into())),
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
                    Property::RequestResponseInformation(RequestResponseInformation(
                        EXPECTED_VAL != 0,
                    )),
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
                    Property::RequestProblemInformation(RequestProblemInformation(
                        EXPECTED_VAL != 0,
                    )),
                ),
            ];

            for (id, expected) in input {
                let buf = [id, EXPECTED_VAL];
                let mut iter = PropertyIterator::from(&buf[..]);
                let property = iter.next().unwrap();
                assert_eq!(property.unwrap(), expected);
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
                    Property::ReceiveMaximum(ReceiveMaximum(NonZero::from(EXPECTED_VAL))),
                ),
                (
                    TopicAliasMaximum::PROPERTY_ID,
                    Property::TopicAliasMaximum(TopicAliasMaximum(EXPECTED_VAL)),
                ),
                (
                    TopicAlias::PROPERTY_ID,
                    Property::TopicAlias(TopicAlias(NonZero::from(EXPECTED_VAL))),
                ),
            ];

            for (id, expected) in input {
                let buf = [id, (EXPECTED_VAL >> 8) as u8, EXPECTED_VAL as u8];
                let mut iter = PropertyIterator::from(&buf[..]);
                let property = iter.next().unwrap();
                assert_eq!(property.unwrap(), expected);
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
                    Property::MaximumPacketSize(MaximumPacketSize(NonZero::from(EXPECTED_VAL))),
                ),
            ];

            for (id, expected) in input {
                let buf = [id, 0x00, 0x00, 0xae, 0x18];
                let mut iter = PropertyIterator::from(&buf[..]);
                let property = iter.next().unwrap();
                assert_eq!(property.unwrap(), expected);
            }
        }

        #[test]
        fn var_size_int() {
            const EXPECTED_VAL: u8 = 64;
            let input = [(
                SubscriptionIdentifier::PROPERTY_ID,
                Property::SubscriptionIdentifier(SubscriptionIdentifier(NonZero::from(
                    VarSizeInt::from(EXPECTED_VAL),
                ))),
            )];

            for (id, expected) in input {
                let buf = [id, EXPECTED_VAL];
                let mut iter = PropertyIterator::from(&buf[..]);
                let property = iter.next().unwrap();
                assert_eq!(property.unwrap(), expected);
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
                let property = iter.next().unwrap();
                assert_eq!(property.unwrap(), expected);
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
                let property = iter.next().unwrap();
                assert_eq!(property.unwrap(), expected);
            }
        }

        #[test]
        fn utf8_string_pair() {
            const EXPECTED_KEY: &str = "key";
            const EXPECTED_VAL: &str = "val";
            const INPUT: [u8; 11] = [
                UserProperty::PROPERTY_ID,
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
            let mut iter = PropertyIterator::from(&INPUT[..]);
            let property = iter.next().unwrap();

            match property.unwrap() {
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

        fn byte_test<T>(property: T, expected: u8)
        where
            T: SizedProperty + PropertyID + TryToByteBuffer,
            <T as TryToByteBuffer>::Error: core::fmt::Debug,
        {
            let mut buf = [0u8; 2];
            let result = property.try_to_byte_buffer(&mut buf);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), [T::PROPERTY_ID, expected]);
        }

        fn two_byte_int_test<T>(property: T, expected: u16)
        where
            T: SizedProperty + PropertyID + TryToByteBuffer,
            <T as TryToByteBuffer>::Error: core::fmt::Debug,
        {
            let mut buf = [0u8; 3];
            let result = property.try_to_byte_buffer(&mut buf);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                [&[T::PROPERTY_ID], &expected.to_be_bytes()[..]].concat()
            );
        }

        fn four_byte_int_test<T>(property: T, expected: u32)
        where
            T: SizedProperty + PropertyID + TryToByteBuffer,
            <T as TryToByteBuffer>::Error: core::fmt::Debug,
        {
            let mut buf = [0u8; 5];
            let result = property.try_to_byte_buffer(&mut buf);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                [&[T::PROPERTY_ID], &expected.to_be_bytes()[..]].concat()
            );
        }

        fn utf8_string_test<T>(property: T, expected: Vec<u8>)
        where
            T: SizedProperty + PropertyID + TryToByteBuffer,
            <T as TryToByteBuffer>::Error: core::fmt::Debug,
        {
            let mut buf = [0u8; 6];
            let result = property.try_to_byte_buffer(&mut buf);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), [&[T::PROPERTY_ID], &expected[..]].concat());
        }

        #[test]
        fn byte() {
            const EXPECTED_VAL: u8 = 1;

            byte_test(PayloadFormatIndicator(EXPECTED_VAL != 0), EXPECTED_VAL);
            byte_test(RequestResponseInformation(EXPECTED_VAL != 0), EXPECTED_VAL);
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
            byte_test(RequestProblemInformation(EXPECTED_VAL != 0), EXPECTED_VAL);
        }

        #[test]
        fn two_byte_int() {
            const EXPECTED_VAL: u16 = 0x1234;

            two_byte_int_test(ServerKeepAlive(EXPECTED_VAL), EXPECTED_VAL);
            two_byte_int_test(ReceiveMaximum(NonZero::from(EXPECTED_VAL)), EXPECTED_VAL);
            two_byte_int_test(TopicAliasMaximum(EXPECTED_VAL), EXPECTED_VAL);
            two_byte_int_test(TopicAlias(NonZero::from(EXPECTED_VAL)), EXPECTED_VAL);
        }

        #[test]
        fn four_byte_int() {
            const EXPECTED_VAL: u32 = 0x12345678;

            four_byte_int_test(MessageExpiryInterval(EXPECTED_VAL), EXPECTED_VAL);
            four_byte_int_test(SessionExpiryInterval(EXPECTED_VAL), EXPECTED_VAL);
            four_byte_int_test(WillDelayInterval(EXPECTED_VAL), EXPECTED_VAL);
            four_byte_int_test(MaximumPacketSize(NonZero::from(EXPECTED_VAL)), EXPECTED_VAL);
        }

        #[test]
        fn var_size_int() {
            const INPUT_VAL: u16 = 16383;
            const EXPECTED_BUF: &[u8] = &[0xff, 0x7f];

            let mut buf = [0u8; 5];
            let result = SubscriptionIdentifier(NonZero::from(VarSizeInt::from(INPUT_VAL)))
                .try_to_byte_buffer(&mut buf);
            assert!(result.is_ok());
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
                UserProperty::PROPERTY_ID,
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
