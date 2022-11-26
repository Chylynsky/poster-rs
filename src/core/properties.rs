use crate::core::{
    base_types::*,
    error::{InvalidPropertyId, PropertyError},
    utils::{ByteLen, Decoder, Encode, PropertyID, TryDecode},
};
use bytes::{Bytes, BytesMut};
use core::{convert::From, mem};

macro_rules! declare_property {
    ($property_name:ident, $property_type:ty, $property_id:literal) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $property_name(pub(crate) $property_type);

        impl PropertyID for $property_name {
            const PROPERTY_ID: u8 = $property_id;
        }

        impl ByteLen for $property_name {
            fn byte_len(&self) -> usize {
                mem::size_of_val(&Self::PROPERTY_ID) + self.0.byte_len()
            }
        }

        impl Encode for $property_name {
            fn encode(&self, buf: &mut BytesMut) {
                Self::PROPERTY_ID.encode(buf);
                self.0.encode(buf);
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

macro_rules! declare_property_ref {
    ($property_name:ident, $property_type:ident, $property_id:literal) => {
        #[derive(Clone, Debug, PartialEq, Copy)]
        pub(crate) struct $property_name<'a>($property_type<'a>);

        impl<'a> PropertyID for $property_name<'a> {
            const PROPERTY_ID: u8 = $property_id;
        }

        impl<'a> ByteLen for $property_name<'a> {
            fn byte_len(&self) -> usize {
                mem::size_of_val(&Self::PROPERTY_ID) + self.0.byte_len()
            }
        }

        impl<'a> Encode for $property_name<'a> {
            fn encode(&self, buf: &mut BytesMut) {
                Self::PROPERTY_ID.encode(buf);
                self.0.encode(buf);
            }
        }

        impl<'a> From<$property_type<'a>> for $property_name<'a> {
            fn from(val: $property_type<'a>) -> Self {
                Self { 0: val }
            }
        }

        impl<'a> From<$property_name<'a>> for $property_type<'a> {
            fn from(val: $property_name) -> $property_type {
                val.0
            }
        }
    };
}

declare_property!(PayloadFormatIndicator, bool, 1);

impl Copy for PayloadFormatIndicator {}

declare_property!(MessageExpiryInterval, u32, 2);

impl Copy for MessageExpiryInterval {}

declare_property!(ContentType, UTF8String, 3);
declare_property_ref!(ContentTypeRef, UTF8StringRef, 3);
declare_property!(ResponseTopic, UTF8String, 8);
declare_property_ref!(ResponseTopicRef, UTF8StringRef, 8);
declare_property!(CorrelationData, Binary, 9);
declare_property_ref!(CorrelationDataRef, BinaryRef, 9);
declare_property!(SubscriptionIdentifier, NonZero<VarSizeInt>, 11);

impl Copy for SubscriptionIdentifier {}

declare_property!(SessionExpiryInterval, u32, 17);

#[allow(clippy::derivable_impls)]
impl Default for SessionExpiryInterval {
    fn default() -> Self {
        Self(0)
    }
}

impl Copy for SessionExpiryInterval {}

declare_property!(AssignedClientIdentifier, UTF8String, 18);
declare_property_ref!(AssignedClientIdentifierRef, UTF8StringRef, 18);
declare_property!(ServerKeepAlive, u16, 19);

impl Copy for ServerKeepAlive {}

declare_property!(AuthenticationMethod, UTF8String, 21);
declare_property_ref!(AuthenticationMethodRef, UTF8StringRef, 21);
declare_property!(AuthenticationData, Binary, 22);
declare_property_ref!(AuthenticationDataRef, BinaryRef, 22);
declare_property!(RequestProblemInformation, bool, 23);

impl Copy for RequestProblemInformation {}

declare_property!(WillDelayInterval, u32, 24);

#[allow(clippy::derivable_impls)]
impl Default for WillDelayInterval {
    fn default() -> Self {
        Self(0)
    }
}

impl Copy for WillDelayInterval {}

declare_property!(RequestResponseInformation, bool, 25);

impl Copy for RequestResponseInformation {}

declare_property!(ResponseInformation, UTF8String, 26);
declare_property_ref!(ResponseInformationRef, UTF8StringRef, 26);
declare_property!(ServerReference, UTF8String, 28);
declare_property_ref!(ServerReferenceRef, UTF8StringRef, 28);
declare_property!(ReasonString, UTF8String, 31);
declare_property_ref!(ReasonStringRef, UTF8StringRef, 31);
declare_property!(ReceiveMaximum, NonZero<u16>, 33);

impl Default for ReceiveMaximum {
    fn default() -> Self {
        Self(NonZero::try_from(65535).unwrap())
    }
}

impl Copy for ReceiveMaximum {}

declare_property!(TopicAliasMaximum, u16, 34);

#[allow(clippy::derivable_impls)]
impl Default for TopicAliasMaximum {
    fn default() -> Self {
        Self(0)
    }
}

impl Copy for TopicAliasMaximum {}

declare_property!(TopicAlias, NonZero<u16>, 35);

impl Copy for TopicAlias {}

declare_property!(MaximumQoS, QoS, 36);

impl Default for MaximumQoS {
    fn default() -> Self {
        Self(QoS::ExactlyOnce)
    }
}

impl Copy for MaximumQoS {}

declare_property!(RetainAvailable, bool, 37);

impl Default for RetainAvailable {
    fn default() -> Self {
        Self(true)
    }
}

impl Copy for RetainAvailable {}

declare_property!(UserProperty, UTF8StringPair, 38);
declare_property_ref!(UserPropertyRef, UTF8StringPairRef, 38);
declare_property!(MaximumPacketSize, NonZero<u32>, 39);

impl Copy for MaximumPacketSize {}

declare_property!(WildcardSubscriptionAvailable, bool, 40);

impl Default for WildcardSubscriptionAvailable {
    fn default() -> Self {
        Self(true)
    }
}

impl Copy for WildcardSubscriptionAvailable {}

declare_property!(SubscriptionIdentifierAvailable, bool, 41);

impl Default for SubscriptionIdentifierAvailable {
    fn default() -> Self {
        Self(true)
    }
}

impl Copy for SubscriptionIdentifierAvailable {}

declare_property!(SharedSubscriptionAvailable, bool, 42);

impl Default for SharedSubscriptionAvailable {
    fn default() -> Self {
        Self(true)
    }
}

impl Copy for SharedSubscriptionAvailable {}

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

impl ByteLen for Property {
    fn byte_len(&self) -> usize {
        match self {
            Self::PayloadFormatIndicator(property) => property.byte_len(),
            Self::MessageExpiryInterval(property) => property.byte_len(),
            Self::ContentType(property) => property.byte_len(),
            Self::ResponseTopic(property) => property.byte_len(),
            Self::CorrelationData(property) => property.byte_len(),
            Self::SubscriptionIdentifier(property) => property.byte_len(),
            Self::SessionExpiryInterval(property) => property.byte_len(),
            Self::AssignedClientIdentifier(property) => property.byte_len(),
            Self::ServerKeepAlive(property) => property.byte_len(),
            Self::AuthenticationMethod(property) => property.byte_len(),
            Self::AuthenticationData(property) => property.byte_len(),
            Self::RequestProblemInformation(property) => property.byte_len(),
            Self::WillDelayInterval(property) => property.byte_len(),
            Self::RequestResponseInformation(property) => property.byte_len(),
            Self::ResponseInformation(property) => property.byte_len(),
            Self::ServerReference(property) => property.byte_len(),
            Self::ReasonString(property) => property.byte_len(),
            Self::ReceiveMaximum(property) => property.byte_len(),
            Self::TopicAliasMaximum(property) => property.byte_len(),
            Self::TopicAlias(property) => property.byte_len(),
            Self::MaximumQoS(property) => property.byte_len(),
            Self::RetainAvailable(property) => property.byte_len(),
            Self::UserProperty(property) => property.byte_len(),
            Self::MaximumPacketSize(property) => property.byte_len(),
            Self::WildcardSubscriptionAvailable(property) => property.byte_len(),
            Self::SubscriptionIdentifierAvailable(property) => property.byte_len(),
            Self::SharedSubscriptionAvailable(property) => property.byte_len(),
        }
    }
}

impl TryDecode for Property {
    type Error = PropertyError;

    fn try_decode(buf: Bytes) -> Result<Self, Self::Error> {
        let mut decoder = Decoder::from(buf);
        let id = decoder.try_decode::<u8>()?; // Technically, the ID is Variable Byte Integer

        match u8::from(id) {
            PayloadFormatIndicator::PROPERTY_ID => decoder
                .try_decode::<bool>()
                .map(|val| Property::PayloadFormatIndicator(PayloadFormatIndicator(val)))
                .map_err(PropertyError::from),

            RequestResponseInformation::PROPERTY_ID => decoder
                .try_decode::<bool>()
                .map(|val| Property::RequestResponseInformation(RequestResponseInformation(val)))
                .map_err(PropertyError::from),

            WildcardSubscriptionAvailable::PROPERTY_ID => decoder
                .try_decode::<bool>()
                .map(|val| {
                    Property::WildcardSubscriptionAvailable(WildcardSubscriptionAvailable(val))
                })
                .map_err(PropertyError::from),

            SubscriptionIdentifierAvailable::PROPERTY_ID => decoder
                .try_decode::<bool>()
                .map(|val| {
                    Property::SubscriptionIdentifierAvailable(SubscriptionIdentifierAvailable(val))
                })
                .map_err(PropertyError::from),

            SharedSubscriptionAvailable::PROPERTY_ID => decoder
                .try_decode::<bool>()
                .map(|val| Property::SharedSubscriptionAvailable(SharedSubscriptionAvailable(val)))
                .map_err(PropertyError::from),

            RetainAvailable::PROPERTY_ID => decoder
                .try_decode::<bool>()
                .map(|val| Property::RetainAvailable(RetainAvailable(val)))
                .map_err(PropertyError::from),

            RequestProblemInformation::PROPERTY_ID => decoder
                .try_decode::<bool>()
                .map(|val| Property::RequestProblemInformation(RequestProblemInformation(val)))
                .map_err(PropertyError::from),

            MaximumQoS::PROPERTY_ID => decoder
                .try_decode::<QoS>()
                .map(|val| Property::MaximumQoS(MaximumQoS(val)))
                .map_err(PropertyError::from),

            ServerKeepAlive::PROPERTY_ID => decoder
                .try_decode::<u16>()
                .map(|val| Property::ServerKeepAlive(ServerKeepAlive(val)))
                .map_err(PropertyError::from),

            TopicAliasMaximum::PROPERTY_ID => decoder
                .try_decode::<u16>()
                .map(|val| Property::TopicAliasMaximum(TopicAliasMaximum(val)))
                .map_err(PropertyError::from),

            ReceiveMaximum::PROPERTY_ID => decoder
                .try_decode::<NonZero<u16>>()
                .map(|val| Property::ReceiveMaximum(ReceiveMaximum(val)))
                .map_err(PropertyError::from),

            TopicAlias::PROPERTY_ID => decoder
                .try_decode::<NonZero<u16>>()
                .map(|val| Property::TopicAlias(TopicAlias(val)))
                .map_err(PropertyError::from),

            MessageExpiryInterval::PROPERTY_ID => decoder
                .try_decode::<u32>()
                .map(|val| Property::MessageExpiryInterval(MessageExpiryInterval(val)))
                .map_err(PropertyError::from),

            SessionExpiryInterval::PROPERTY_ID => decoder
                .try_decode::<u32>()
                .map(|val| Property::SessionExpiryInterval(SessionExpiryInterval(val)))
                .map_err(PropertyError::from),

            WillDelayInterval::PROPERTY_ID => decoder
                .try_decode::<u32>()
                .map(|val| Property::WillDelayInterval(WillDelayInterval(val)))
                .map_err(PropertyError::from),

            MaximumPacketSize::PROPERTY_ID => decoder
                .try_decode::<NonZero<u32>>()
                .map(|val| Property::MaximumPacketSize(MaximumPacketSize(val)))
                .map_err(PropertyError::from),

            SubscriptionIdentifier::PROPERTY_ID => decoder
                .try_decode::<NonZero<VarSizeInt>>()
                .map(|val| Property::SubscriptionIdentifier(SubscriptionIdentifier(val)))
                .map_err(PropertyError::from),

            CorrelationData::PROPERTY_ID => decoder
                .try_decode::<Binary>()
                .map(|val| Property::CorrelationData(CorrelationData(val)))
                .map_err(PropertyError::from),

            AuthenticationData::PROPERTY_ID => decoder
                .try_decode::<Binary>()
                .map(|val| Property::AuthenticationData(AuthenticationData(val)))
                .map_err(PropertyError::from),

            ContentType::PROPERTY_ID => decoder
                .try_decode::<UTF8String>()
                .map(|val| Property::ContentType(ContentType(val)))
                .map_err(PropertyError::from),

            ResponseTopic::PROPERTY_ID => decoder
                .try_decode::<UTF8String>()
                .map(|val| Property::ResponseTopic(ResponseTopic(val)))
                .map_err(PropertyError::from),

            AssignedClientIdentifier::PROPERTY_ID => decoder
                .try_decode::<UTF8String>()
                .map(|val| Property::AssignedClientIdentifier(AssignedClientIdentifier(val)))
                .map_err(PropertyError::from),

            AuthenticationMethod::PROPERTY_ID => decoder
                .try_decode::<UTF8String>()
                .map(|val| Property::AuthenticationMethod(AuthenticationMethod(val)))
                .map_err(PropertyError::from),

            ResponseInformation::PROPERTY_ID => decoder
                .try_decode::<UTF8String>()
                .map(|val| Property::ResponseInformation(ResponseInformation(val)))
                .map_err(PropertyError::from),

            ServerReference::PROPERTY_ID => decoder
                .try_decode::<UTF8String>()
                .map(|val| Property::ServerReference(ServerReference(val)))
                .map_err(PropertyError::from),

            ReasonString::PROPERTY_ID => decoder
                .try_decode::<UTF8String>()
                .map(|val| Property::ReasonString(ReasonString(val)))
                .map_err(PropertyError::from),

            UserProperty::PROPERTY_ID => decoder
                .try_decode::<UTF8StringPair>()
                .map(|val| Property::UserProperty(UserProperty(val)))
                .map_err(PropertyError::from),

            _ => Err(InvalidPropertyId.into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod try_decode {
        use super::*;

        #[test]
        fn u8() {
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
                let property = Property::try_decode(Bytes::copy_from_slice(&buf)).unwrap();
                assert_eq!(property, expected);
            }
        }

        #[test]
        fn u16() {
            const EXPECTED_VAL: u16 = 965;
            let input = [
                (
                    ServerKeepAlive::PROPERTY_ID,
                    Property::ServerKeepAlive(ServerKeepAlive(EXPECTED_VAL)),
                ),
                (
                    ReceiveMaximum::PROPERTY_ID,
                    Property::ReceiveMaximum(ReceiveMaximum(
                        NonZero::try_from(EXPECTED_VAL).unwrap(),
                    )),
                ),
                (
                    TopicAliasMaximum::PROPERTY_ID,
                    Property::TopicAliasMaximum(TopicAliasMaximum(EXPECTED_VAL)),
                ),
                (
                    TopicAlias::PROPERTY_ID,
                    Property::TopicAlias(TopicAlias(NonZero::try_from(EXPECTED_VAL).unwrap())),
                ),
            ];

            for (id, expected) in input {
                let buf = [id, (EXPECTED_VAL >> 8) as u8, EXPECTED_VAL as u8];
                let property = Property::try_decode(Bytes::copy_from_slice(&buf)).unwrap();
                assert_eq!(property, expected);
            }
        }

        #[test]
        fn u32() {
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
                    Property::MaximumPacketSize(MaximumPacketSize(
                        NonZero::try_from(EXPECTED_VAL).unwrap(),
                    )),
                ),
            ];

            for (id, expected) in input {
                let buf = [id, 0x00, 0x00, 0xae, 0x18];
                let property = Property::try_decode(Bytes::copy_from_slice(&buf)).unwrap();
                assert_eq!(property, expected);
            }
        }

        #[test]
        fn var_size_int() {
            const EXPECTED_VAL: u8 = 64;
            let input = [(
                SubscriptionIdentifier::PROPERTY_ID,
                Property::SubscriptionIdentifier(SubscriptionIdentifier(
                    NonZero::try_from(VarSizeInt::from(EXPECTED_VAL)).unwrap(),
                )),
            )];

            for (id, expected) in input {
                let buf = [id, EXPECTED_VAL];
                let property = Property::try_decode(Bytes::copy_from_slice(&buf)).unwrap();
                assert_eq!(property, expected);
            }
        }

        #[test]
        fn binary() {
            const EXPECTED_VAL: [u8; 3] = [0x02u8, 0xae, 0x18];
            let input_bytes = [
                &((EXPECTED_VAL.len() as u16).to_be_bytes()),
                &EXPECTED_VAL[..],
            ]
            .concat();
            let expected_binary = Binary(Bytes::from_static(&EXPECTED_VAL));

            let input = [
                (
                    &input_bytes,
                    CorrelationData::PROPERTY_ID,
                    Property::CorrelationData(CorrelationData(expected_binary.clone())),
                ),
                (
                    &input_bytes,
                    AuthenticationData::PROPERTY_ID,
                    Property::AuthenticationData(AuthenticationData(expected_binary.clone())),
                ),
            ];

            for (buf, id, expected) in input {
                let buf = [&[id], &buf[..]].concat();
                let property = Property::try_decode(Bytes::from(buf)).unwrap();
                assert_eq!(property, expected);
            }
        }

        #[test]
        fn utf8_string() {
            const EXPECTED_VAL: [u8; 3] = [b'v', b'a', b'l'];
            let input_bytes = [
                &((EXPECTED_VAL.len() as u16).to_be_bytes()),
                &EXPECTED_VAL[..],
            ]
            .concat();
            let expected_str = UTF8String(Bytes::from_static(&EXPECTED_VAL));

            let input = [
                (
                    &input_bytes,
                    ContentType::PROPERTY_ID,
                    Property::ContentType(ContentType(expected_str.clone())),
                ),
                (
                    &input_bytes,
                    ResponseTopic::PROPERTY_ID,
                    Property::ResponseTopic(ResponseTopic(expected_str.clone())),
                ),
                (
                    &input_bytes,
                    AssignedClientIdentifier::PROPERTY_ID,
                    Property::AssignedClientIdentifier(AssignedClientIdentifier(
                        expected_str.clone(),
                    )),
                ),
                (
                    &input_bytes,
                    AuthenticationMethod::PROPERTY_ID,
                    Property::AuthenticationMethod(AuthenticationMethod(expected_str.clone())),
                ),
                (
                    &input_bytes,
                    ResponseInformation::PROPERTY_ID,
                    Property::ResponseInformation(ResponseInformation(expected_str.clone())),
                ),
                (
                    &input_bytes,
                    ServerReference::PROPERTY_ID,
                    Property::ServerReference(ServerReference(expected_str.clone())),
                ),
                (
                    &input_bytes,
                    ReasonString::PROPERTY_ID,
                    Property::ReasonString(ReasonString(expected_str.clone())),
                ),
            ];

            for (buf, id, expected) in input {
                let buf = [&[id], &buf[..]].concat();
                let property = Property::try_decode(Bytes::from(buf)).unwrap();
                assert_eq!(property, expected);
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
            let property = Property::try_decode(Bytes::from_static(&INPUT)).unwrap();

            match property {
                Property::UserProperty(result) => {
                    assert_eq!(result.byte_len(), INPUT.len());

                    let pair = result.0;

                    assert_eq!(&pair.0, EXPECTED_KEY.as_bytes());
                    assert_eq!(&pair.1, EXPECTED_VAL.as_bytes());
                }
                _ => panic!(),
            }
        }
    }

    mod encode {
        use super::*;

        fn byte_test<T>(property: T, expected: u8)
        where
            T: ByteLen + PropertyID + Encode,
        {
            let mut buf = BytesMut::new();
            property.encode(&mut buf);
            assert_eq!(&[T::PROPERTY_ID, expected][..], &buf.split().freeze());
        }

        fn two_byte_int_test<T>(property: T, expected: u16)
        where
            T: ByteLen + PropertyID + Encode,
        {
            let mut buf = BytesMut::new();
            property.encode(&mut buf);
            assert_eq!(
                &[&[T::PROPERTY_ID], &expected.to_be_bytes()[..]].concat(),
                &buf.split().freeze()
            );
        }

        fn four_byte_int_test<T>(property: T, expected: u32)
        where
            T: ByteLen + PropertyID + Encode,
        {
            let mut buf = BytesMut::new();
            property.encode(&mut buf);
            assert_eq!(
                &[&[T::PROPERTY_ID], &expected.to_be_bytes()[..]].concat(),
                &buf.split().freeze()
            );
        }

        fn utf8_string_test<T>(property: T, expected: Vec<u8>)
        where
            T: ByteLen + PropertyID + Encode,
        {
            let mut buf = BytesMut::new();
            property.encode(&mut buf);
            assert_eq!(
                &[&[T::PROPERTY_ID], &expected[..]].concat(),
                &buf.split().freeze()
            );
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
            two_byte_int_test(
                ReceiveMaximum(NonZero::try_from(EXPECTED_VAL).unwrap()),
                EXPECTED_VAL,
            );
            two_byte_int_test(TopicAliasMaximum(EXPECTED_VAL), EXPECTED_VAL);
            two_byte_int_test(
                TopicAlias(NonZero::try_from(EXPECTED_VAL).unwrap()),
                EXPECTED_VAL,
            );
        }

        #[test]
        fn four_byte_int() {
            const EXPECTED_VAL: u32 = 0x12345678;

            four_byte_int_test(MessageExpiryInterval(EXPECTED_VAL), EXPECTED_VAL);
            four_byte_int_test(SessionExpiryInterval(EXPECTED_VAL), EXPECTED_VAL);
            four_byte_int_test(WillDelayInterval(EXPECTED_VAL), EXPECTED_VAL);
            four_byte_int_test(
                MaximumPacketSize(NonZero::try_from(EXPECTED_VAL).unwrap()),
                EXPECTED_VAL,
            );
        }

        #[test]
        fn var_size_int() {
            const INPUT_VAL: u16 = 16383;
            const EXPECTED_BUF: &[u8] = &[0xff, 0x7f];

            let mut buf = BytesMut::new();
            SubscriptionIdentifier(NonZero::try_from(VarSizeInt::from(INPUT_VAL)).unwrap())
                .encode(&mut buf);
            assert_eq!(
                &[&[SubscriptionIdentifier::PROPERTY_ID], EXPECTED_BUF].concat(),
                &buf.split().freeze()
            );
        }

        #[test]
        fn utf8_string() {
            const INPUT_VAL: &str = "val";
            const EXPECTED_BUF: [u8; 5] = [0, 3, b'v', b'a', b'l'];
            let input_str = UTF8String(Bytes::from_static(INPUT_VAL.as_bytes()));

            utf8_string_test(ContentType(input_str.clone()), Vec::from(EXPECTED_BUF));
            utf8_string_test(ResponseTopic(input_str.clone()), Vec::from(EXPECTED_BUF));
            utf8_string_test(
                AssignedClientIdentifier(input_str.clone()),
                Vec::from(EXPECTED_BUF),
            );
            utf8_string_test(
                AuthenticationMethod(input_str.clone()),
                Vec::from(EXPECTED_BUF),
            );
            utf8_string_test(
                ResponseInformation(input_str.clone()),
                Vec::from(EXPECTED_BUF),
            );
            utf8_string_test(ServerReference(input_str.clone()), Vec::from(EXPECTED_BUF));
            utf8_string_test(ReasonString(input_str.clone()), Vec::from(EXPECTED_BUF));
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

            let input_pair = UTF8StringPair(
                Bytes::from_static(INPUT_KEY.as_bytes()),
                Bytes::from_static(INPUT_VAL.as_bytes()),
            );
            let mut buf = BytesMut::new();
            let property = UserProperty(input_pair);
            property.encode(&mut buf);

            assert_eq!(&EXPECTED_BUF[..], buf.split().freeze());
        }
    }
}
