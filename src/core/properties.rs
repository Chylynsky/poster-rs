use crate::{
    core::base_types::*,
    core::utils::{
        ByteWriter, PropertyID, SizedProperty, ToByteBuffer, TryFromBytes, TryFromIterator,
        TryToByteBuffer,
    },
};
use std::{convert::From, iter::Iterator, mem};

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
        pub(crate) struct $property_name(pub(crate) $property_type);

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
            fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
                let result = buf.get_mut(0..self.property_len())?;
                Some(to_byte_buffer_unchecked(self, &self.0, result))
            }
        }
    };
}

declare_property!(PayloadFormatIndicator, Boolean, 1);
declare_property!(MessageExpiryInterval, FourByteInteger, 2);
declare_property!(ContentType, UTF8String, 3);
declare_property!(ResponseTopic, UTF8String, 8);
declare_property!(CorrelationData, Binary, 9);
declare_property!(SubscriptionIdentifier, NonZero<VarSizeInt>, 11);
declare_property!(SessionExpiryInterval, FourByteInteger, 17);

#[allow(clippy::derivable_impls)]
impl Default for SessionExpiryInterval {
    fn default() -> Self {
        Self(0)
    }
}

declare_property!(AssignedClientIdentifier, UTF8String, 18);
declare_property!(ServerKeepAlive, TwoByteInteger, 19);
declare_property!(AuthenticationMethod, UTF8String, 21);
declare_property!(AuthenticationData, Binary, 22);
declare_property!(RequestProblemInformation, Boolean, 23);

declare_property!(WillDelayInterval, FourByteInteger, 24);

#[allow(clippy::derivable_impls)]
impl Default for WillDelayInterval {
    fn default() -> Self {
        Self(0)
    }
}

declare_property!(RequestResponseInformation, Boolean, 25);
declare_property!(ResponseInformation, UTF8String, 26);
declare_property!(ServerReference, UTF8String, 28);
declare_property!(ReasonString, UTF8String, 31);
declare_property!(ReceiveMaximum, NonZero<TwoByteInteger>, 33);

impl Default for ReceiveMaximum {
    fn default() -> Self {
        Self(NonZero::from(65535))
    }
}

declare_property!(TopicAliasMaximum, TwoByteInteger, 34);

#[allow(clippy::derivable_impls)]
impl Default for TopicAliasMaximum {
    fn default() -> Self {
        Self(0)
    }
}

declare_property!(TopicAlias, NonZero<TwoByteInteger>, 35);
declare_property!(MaximumQoS, QoS, 36);

impl Default for MaximumQoS {
    fn default() -> Self {
        Self(QoS::ExactlyOnce)
    }
}

declare_property!(RetainAvailable, Boolean, 37);

impl Default for RetainAvailable {
    fn default() -> Self {
        Self(true)
    }
}

declare_property!(UserProperty, UTF8StringPair, 38);
declare_property!(MaximumPacketSize, NonZero<FourByteInteger>, 39);
declare_property!(WildcardSubscriptionAvailable, Boolean, 40);

impl Default for WildcardSubscriptionAvailable {
    fn default() -> Self {
        Self(true)
    }
}

declare_property!(SubscriptionIdentifierAvailable, Boolean, 41);

impl Default for SubscriptionIdentifierAvailable {
    fn default() -> Self {
        Self(true)
    }
}

declare_property!(SharedSubscriptionAvailable, Boolean, 42);

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
    type Item = Property;

    fn next(&mut self) -> Option<Self::Item> {
        let id_var = VarSizeInt::try_from_iter(self.buf.iter().copied())?;
        if id_var.len() != 1 {
            return None;
        }

        let (_, remaining) = self.buf.split_at(id_var.len());
        self.buf = remaining;

        return match u8::from(id_var) {
            PayloadFormatIndicator::PROPERTY_ID => {
                let property = Boolean::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::PayloadFormatIndicator(PayloadFormatIndicator(
                    property,
                )))
            }
            RequestResponseInformation::PROPERTY_ID => {
                let property = Boolean::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::RequestResponseInformation(
                    RequestResponseInformation(property),
                ))
            }
            WildcardSubscriptionAvailable::PROPERTY_ID => {
                let property = Boolean::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::WildcardSubscriptionAvailable(
                    WildcardSubscriptionAvailable(property),
                ))
            }
            SubscriptionIdentifierAvailable::PROPERTY_ID => {
                let property = Boolean::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::SubscriptionIdentifierAvailable(
                    SubscriptionIdentifierAvailable(property),
                ))
            }
            SharedSubscriptionAvailable::PROPERTY_ID => {
                let property = Boolean::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::SharedSubscriptionAvailable(
                    SharedSubscriptionAvailable(property),
                ))
            }
            MaximumQoS::PROPERTY_ID => {
                let property = QoS::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::MaximumQoS(MaximumQoS(property)))
            }
            RetainAvailable::PROPERTY_ID => {
                let property = Boolean::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::RetainAvailable(RetainAvailable(property)))
            }
            RequestProblemInformation::PROPERTY_ID => {
                let property = Boolean::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::RequestProblemInformation(
                    RequestProblemInformation(property),
                ))
            }
            ServerKeepAlive::PROPERTY_ID => {
                let property = TwoByteInteger::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::ServerKeepAlive(ServerKeepAlive(property)))
            }
            ReceiveMaximum::PROPERTY_ID => {
                let property = NonZero::<TwoByteInteger>::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::ReceiveMaximum(ReceiveMaximum(property)))
            }
            TopicAliasMaximum::PROPERTY_ID => {
                let property = TwoByteInteger::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::TopicAliasMaximum(TopicAliasMaximum(property)))
            }
            TopicAlias::PROPERTY_ID => {
                let property = NonZero::<TwoByteInteger>::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::TopicAlias(TopicAlias(property)))
            }
            MessageExpiryInterval::PROPERTY_ID => {
                let property = FourByteInteger::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::MessageExpiryInterval(MessageExpiryInterval(
                    property,
                )))
            }
            SessionExpiryInterval::PROPERTY_ID => {
                let property = FourByteInteger::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::SessionExpiryInterval(SessionExpiryInterval(
                    property,
                )))
            }
            WillDelayInterval::PROPERTY_ID => {
                let property = FourByteInteger::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::WillDelayInterval(WillDelayInterval(property)))
            }
            MaximumPacketSize::PROPERTY_ID => {
                let property = NonZero::<FourByteInteger>::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::MaximumPacketSize(MaximumPacketSize(property)))
            }
            SubscriptionIdentifier::PROPERTY_ID => {
                let property = NonZero::<VarSizeInt>::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::SubscriptionIdentifier(SubscriptionIdentifier(
                    property,
                )))
            }
            CorrelationData::PROPERTY_ID => {
                let property = Binary::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::CorrelationData(CorrelationData(property)))
            }
            ContentType::PROPERTY_ID => {
                let property = UTF8String::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::ContentType(ContentType(property)))
            }
            ResponseTopic::PROPERTY_ID => {
                let property = UTF8String::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::ResponseTopic(ResponseTopic(property)))
            }
            AssignedClientIdentifier::PROPERTY_ID => {
                let property = UTF8String::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::AssignedClientIdentifier(
                    AssignedClientIdentifier(property),
                ))
            }
            AuthenticationMethod::PROPERTY_ID => {
                let property = UTF8String::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::AuthenticationMethod(AuthenticationMethod(
                    property,
                )))
            }
            AuthenticationData::PROPERTY_ID => {
                let property = Binary::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::AuthenticationData(AuthenticationData(property)))
            }
            ResponseInformation::PROPERTY_ID => {
                let property = UTF8String::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::ResponseInformation(ResponseInformation(property)))
            }
            ServerReference::PROPERTY_ID => {
                let property = UTF8String::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::ServerReference(ServerReference(property)))
            }
            ReasonString::PROPERTY_ID => {
                let property = UTF8String::try_from_bytes(self.buf)?;

                let (_, remaining) = self.buf.split_at(property.property_len());
                self.buf = remaining;

                Some(Property::ReasonString(ReasonString(property)))
            }
            UserProperty::PROPERTY_ID => {
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
                    Property::MaximumPacketSize(MaximumPacketSize(NonZero::from(EXPECTED_VAL))),
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
                Property::SubscriptionIdentifier(SubscriptionIdentifier(NonZero::from(
                    VarSizeInt::from(EXPECTED_VAL),
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
