use crate::core::{
    base_types::*,
    error::{CodecError, MandatoryPropertyMissing},
    properties::*,
    utils::{ByteLen, Encode, Encoder, PacketID, SizedPacket},
};
use bytes::BytesMut;
use core::mem;
use derive_builder::Builder;

#[derive(Clone, Copy)]
pub enum RetainHandling {
    SendOnSubscribe = 0,
    SendIfNoSubscription = 1,
    NoSendOnSubscribe = 2,
}

#[derive(Copy, Clone)]
pub(crate) struct SubscriptionOptions {
    pub(crate) maximum_qos: QoS,
    pub(crate) no_local: bool,
    pub(crate) retain_as_published: bool,
    pub(crate) retain_handling: RetainHandling,
}

impl Default for SubscriptionOptions {
    fn default() -> Self {
        Self {
            maximum_qos: QoS::ExactlyOnce,
            no_local: false,
            retain_as_published: false,
            retain_handling: RetainHandling::SendOnSubscribe,
        }
    }
}

impl ByteLen for SubscriptionOptions {
    fn byte_len(&self) -> usize {
        mem::size_of::<u8>()
    }
}

impl Encode for SubscriptionOptions {
    fn encode(&self, buf: &mut BytesMut) {
        let mut encoder = Encoder::from(buf);
        let qos = self.maximum_qos;
        let val = (qos as u8)
            | ((self.no_local as u8) << 3)
            | ((self.retain_as_published as u8) << 4)
            | ((self.retain_handling as u8) << 5);
        encoder.encode(val);
    }
}

#[derive(Builder)]
#[builder(build_fn(error = "CodecError", validate = "Self::validate"))]
pub(crate) struct SubscribeTx<'a> {
    pub(crate) packet_identifier: NonZero<u16>,

    #[builder(setter(strip_option), default)]
    pub(crate) subscription_identifier: Option<SubscriptionIdentifier>,
    #[builder(setter(custom), default)]
    pub(crate) user_property: Vec<UserPropertyRef<'a>>,

    #[builder(setter(custom))]
    pub(crate) payload: Vec<(UTF8StringRef<'a>, SubscriptionOptions)>,
}

impl<'a> SubscribeTxBuilder<'a> {
    fn validate(&self) -> Result<(), CodecError> {
        if self.payload.is_none() {
            Err(MandatoryPropertyMissing.into()) // Empty payload is a protocol error
        } else {
            Ok(())
        }
    }

    pub(crate) fn user_property(&mut self, value: UserPropertyRef<'a>) {
        match self.user_property.as_mut() {
            Some(user_property) => {
                user_property.push(value);
            }
            None => {
                self.user_property = Some(Vec::new());
                self.user_property.as_mut().unwrap().push(value);
            }
        }
    }

    pub(crate) fn payload(&mut self, (topic, opts): (UTF8StringRef<'a>, SubscriptionOptions)) {
        match self.payload.as_mut() {
            Some(payload) => {
                payload.push((topic, opts));
            }
            None => {
                self.payload = Some(Vec::new());
                self.payload.as_mut().unwrap().push((topic, opts));
            }
        }
    }
}

impl<'a> SubscribeTx<'a> {
    const FIXED_HDR: u8 = (Self::PACKET_ID << 4) | 0b0010;

    fn property_len(&self) -> VarSizeInt {
        VarSizeInt::try_from(
            self.subscription_identifier
                .as_ref()
                .map(|val| val.byte_len())
                .unwrap_or(0)
                + self
                    .user_property
                    .iter()
                    .map(|val| val.byte_len())
                    .sum::<usize>(),
        )
        .unwrap()
    }

    fn remaining_len(&self) -> VarSizeInt {
        let property_len = self.property_len();
        VarSizeInt::try_from(
            self.packet_identifier.byte_len()
                + property_len.len()
                + property_len.value() as usize
                + self
                    .payload
                    .iter()
                    .map(|(topic, opts)| topic.byte_len() + opts.byte_len())
                    .sum::<usize>(),
        )
        .unwrap()
    }
}

impl<'a> PacketID for SubscribeTx<'a> {
    const PACKET_ID: u8 = 8;
}

impl<'a> SizedPacket for SubscribeTx<'a> {
    fn packet_len(&self) -> usize {
        let remaining_len = self.remaining_len();
        mem::size_of_val(&Self::FIXED_HDR) + remaining_len.len() + remaining_len.value() as usize
    }
}

impl<'a> Encode for SubscribeTx<'a> {
    fn encode(&self, buf: &mut BytesMut) {
        let mut encoder = Encoder::from(buf);

        encoder.encode(Self::FIXED_HDR);
        encoder.encode(self.remaining_len());
        encoder.encode(self.packet_identifier);

        encoder.encode(self.property_len());

        if let Some(val) = self.subscription_identifier {
            encoder.encode(val);
        }

        for val in self.user_property.iter().copied() {
            encoder.encode(val);
        }

        for (topic, opts) in self.payload.iter().copied() {
            encoder.encode(topic);
            encoder.encode(opts);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_bytes_0() {
        const EXPECTED: [u8; 11] = [
            SubscribeTx::FIXED_HDR,
            9,
            0,
            32,
            0,
            0,
            3,
            b'a',
            b'/',
            b'b',
            0b10,
        ];
        let mut builder = SubscribeTxBuilder::default();
        builder.packet_identifier(NonZero::try_from(32).unwrap());
        builder.payload((
            UTF8StringRef("a/b"),
            SubscriptionOptions {
                maximum_qos: QoS::ExactlyOnce,
                no_local: false,
                retain_as_published: false,
                retain_handling: RetainHandling::SendOnSubscribe,
            },
        ));
        let packet = builder.build().unwrap();

        let mut buf = BytesMut::new();
        packet.encode(&mut buf);

        assert_eq!(&buf.split().freeze()[..], &EXPECTED);
    }
}
