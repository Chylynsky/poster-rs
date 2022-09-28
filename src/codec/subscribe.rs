use crate::core::{
    base_types::*,
    properties::*,
    utils::{ByteWriter, PacketID, SizedPacket, SizedProperty, ToByteBuffer, TryToByteBuffer},
};
use core::mem;

#[derive(Clone, Copy)]
pub enum RetainHandling {
    SendOnSubscribe = 0,
    SendIfNoSubscription = 1,
    NoSendOnSubscribe = 2,
}

pub(crate) struct SubscriptionOptions {
    pub(crate) maximum_qos: MaximumQoS,
    pub(crate) no_local: bool,
    pub(crate) retain_as_published: bool,
    pub(crate) retain_handling: RetainHandling,
}

impl Default for SubscriptionOptions {
    fn default() -> Self {
        Self {
            maximum_qos: MaximumQoS::from(QoS::ExactlyOnce),
            no_local: false,
            retain_as_published: false,
            retain_handling: RetainHandling::SendOnSubscribe,
        }
    }
}

impl SizedProperty for SubscriptionOptions {
    fn property_len(&self) -> usize {
        mem::size_of::<u8>()
    }
}

impl ToByteBuffer for SubscriptionOptions {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let property_len = self.property_len();

        debug_assert!(property_len <= buf.len());

        let result = &mut buf[0..property_len];
        let mut writer = ByteWriter::from(result);

        {
            let qos: QoS = self.maximum_qos.clone().into();
            let val = (qos as u8)
                | ((self.no_local as u8) << 3)
                | ((self.retain_as_published as u8) << 4)
                | ((self.retain_handling as u8) << 5);
            writer.write(&val);
        }

        result
    }
}

pub(crate) struct SubscribeProperties {
    pub(crate) subscription_identifier: Option<SubscriptionIdentifier>,
    pub(crate) user_property: Vec<UserProperty>,
}

impl SizedProperty for SubscribeProperties {
    fn property_len(&self) -> usize {
        self.subscription_identifier
            .as_ref()
            .map(|val| val.property_len())
            .unwrap_or(0)
            + self
                .user_property
                .iter()
                .map(|val| val.property_len())
                .sum::<usize>()
    }
}

impl ToByteBuffer for SubscribeProperties {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let property_len = VarSizeInt::from(self.property_len());
        let len = property_len.len() + property_len.value() as usize;

        debug_assert!(len <= buf.len());

        let result = &mut buf[0..len];
        let mut writer = ByteWriter::from(result);

        writer.write(&property_len);

        if let Some(val) = self.subscription_identifier.as_ref() {
            writer.write(val);
        }

        for val in self.user_property.iter() {
            writer.write(val);
        }

        result
    }
}

pub(crate) struct Subscribe {
    pub(crate) packet_identifier: NonZero<u16>,
    pub(crate) properties: SubscribeProperties,
    pub(crate) payload: Vec<(String, SubscriptionOptions)>,
}

impl Subscribe {
    const FIXED_HDR: u8 = (Self::PACKET_ID << 4) | 0b0010;

    fn remaining_len(&self) -> VarSizeInt {
        let property_len = VarSizeInt::from(self.properties.property_len());
        VarSizeInt::from(
            self.packet_identifier.property_len()
                + property_len.len()
                + property_len.value() as usize
                + self
                    .payload
                    .iter()
                    .map(|(topic, opts)| topic.property_len() + opts.property_len())
                    .sum::<usize>(),
        )
    }
}

impl PacketID for Subscribe {
    const PACKET_ID: u8 = 8;
}

impl SizedPacket for Subscribe {
    fn packet_len(&self) -> usize {
        let remaining_len = self.remaining_len();
        mem::size_of_val(&Self::FIXED_HDR) + remaining_len.len() + remaining_len.value() as usize
    }
}

impl TryToByteBuffer for Subscribe {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let packet_len = self.packet_len();

        let result = buf.get_mut(0..packet_len)?;
        let mut writer = ByteWriter::from(result);

        writer.write(&Self::FIXED_HDR);

        let remaining_len = self.remaining_len();
        debug_assert!(remaining_len.value() as usize <= writer.remaining());
        writer.write(&remaining_len);

        writer.write(&self.packet_identifier);
        writer.write(&self.properties);

        for (topic, opts) in self.payload.iter() {
            writer.write(topic);
            writer.write(opts);
        }

        Some(result)
    }
}

#[derive(Default)]
pub(crate) struct SubscribeBuilder {
    packet_identifier: Option<NonZero<u16>>,
    subscription_identifier: Option<SubscriptionIdentifier>,
    user_property: Vec<UserProperty>,
    payload: Vec<(String, SubscriptionOptions)>,
}

impl SubscribeBuilder {
    pub(crate) fn packet_identifier(&mut self, packet_identifier: NonZero<u16>) -> &mut Self {
        self.packet_identifier = Some(packet_identifier);
        self
    }

    pub(crate) fn subscription_identifier(&mut self, val: NonZero<VarSizeInt>) -> &mut Self {
        self.subscription_identifier = Some(val.into());
        self
    }

    pub(crate) fn user_property(&mut self, val: StringPair) -> &mut Self {
        self.user_property.push(val.into());
        self
    }

    pub(crate) fn payload(&mut self, payload: (String, SubscriptionOptions)) -> &mut Self {
        self.payload.push(payload);
        self
    }

    pub(crate) fn build(self) -> Option<Subscribe> {
        if self.payload.is_empty() {
            return None; // Subscribe packet with no payload is a Protocol Error
        }

        let properties = SubscribeProperties {
            subscription_identifier: self.subscription_identifier,
            user_property: self.user_property,
        };

        Some(Subscribe {
            packet_identifier: self.packet_identifier?,
            properties,
            payload: self.payload,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_bytes_0() {
        const EXPECTED: [u8; 11] = [
            Subscribe::FIXED_HDR,
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
        let mut builder = SubscribeBuilder::default();
        builder.packet_identifier(NonZero::from(32));
        builder.payload((
            String::from("a/b"),
            SubscriptionOptions {
                maximum_qos: MaximumQoS::from(QoS::ExactlyOnce),
                no_local: false,
                retain_as_published: false,
                retain_handling: RetainHandling::SendOnSubscribe,
            },
        ));
        let packet = builder.build().unwrap();

        let mut buf = [0u8; 11];
        let result = packet.try_to_byte_buffer(&mut buf).unwrap();

        assert_eq!(result, EXPECTED);
    }
}
