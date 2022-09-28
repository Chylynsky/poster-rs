use crate::core::{
    base_types::*,
    properties::*,
    utils::{
        ByteReader, ByteWriter, PacketID, SizedPacket, SizedProperty, ToByteBuffer, TryFromBytes,
        TryToByteBuffer,
    },
};
use core::mem;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum DisconnectReason {
    Success = 0x00,
    DisconnectWithWillMessage = 0x04,
    UnspecifiedError = 0x80,
    MalformedPacket = 0x81,
    ProtocolError = 0x82,
    ImplementationSpecificError = 0x83,
    NotAuthorized = 0x87,
    ServerBusy = 0x89,
    ServerShuttingDown = 0x8b,
    KeepAliveTimeout = 0x8d,
    SessionTakenOver = 0x8e,
    TopicFilterInvalid = 0x8f,
    TopicNameInvalid = 0x90,
    ReceiveMaximumExcceeded = 0x93,
    TopicAliasInvalid = 0x94,
    PacketTooLarge = 0x95,
    MessageRateTooHigh = 0x96,
    QuotaExceeded = 0x97,
    AdministrativeAction = 0x98,
    PayloadFormatInvalid = 0x99,
    RetainNotSupported = 0x9a,
    QoSNotSupported = 0x9b,
    UseAnotherServer = 0x9c,
    ServerMoved = 0x9d,
    SharedSubscriptionsNotSupported = 0x9e,
    ConnectionRateExceeded = 0x9f,
    MaximumConnectTime = 0xa0,
    SubscriptionIdentifiersNotSupported = 0xa1,
    WildcardSubscriptionsNotSupported = 0xa2,
}

impl DisconnectReason {
    fn try_from(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(DisconnectReason::Success),
            0x04 => Some(DisconnectReason::DisconnectWithWillMessage),
            0x80 => Some(DisconnectReason::UnspecifiedError),
            0x81 => Some(DisconnectReason::MalformedPacket),
            0x82 => Some(DisconnectReason::ProtocolError),
            0x83 => Some(DisconnectReason::ImplementationSpecificError),
            0x87 => Some(DisconnectReason::NotAuthorized),
            0x89 => Some(DisconnectReason::ServerBusy),
            0x8b => Some(DisconnectReason::ServerShuttingDown),
            0x8d => Some(DisconnectReason::KeepAliveTimeout),
            0x8e => Some(DisconnectReason::SessionTakenOver),
            0x8f => Some(DisconnectReason::TopicFilterInvalid),
            0x90 => Some(DisconnectReason::TopicNameInvalid),
            0x93 => Some(DisconnectReason::ReceiveMaximumExcceeded),
            0x94 => Some(DisconnectReason::TopicAliasInvalid),
            0x95 => Some(DisconnectReason::PacketTooLarge),
            0x96 => Some(DisconnectReason::MessageRateTooHigh),
            0x97 => Some(DisconnectReason::QuotaExceeded),
            0x98 => Some(DisconnectReason::AdministrativeAction),
            0x99 => Some(DisconnectReason::PayloadFormatInvalid),
            0x9a => Some(DisconnectReason::RetainNotSupported),
            0x9b => Some(DisconnectReason::QoSNotSupported),
            0x9c => Some(DisconnectReason::UseAnotherServer),
            0x9d => Some(DisconnectReason::ServerMoved),
            0x9e => Some(DisconnectReason::SharedSubscriptionsNotSupported),
            0x9f => Some(DisconnectReason::ConnectionRateExceeded),
            0xa0 => Some(DisconnectReason::MaximumConnectTime),
            0xa1 => Some(DisconnectReason::SubscriptionIdentifiersNotSupported),
            0xa2 => Some(DisconnectReason::WildcardSubscriptionsNotSupported),
            _ => None,
        }
    }
}

impl SizedProperty for DisconnectReason {
    fn property_len(&self) -> usize {
        mem::size_of::<u8>()
    }
}

impl Default for DisconnectReason {
    fn default() -> Self {
        Self::Success
    }
}

impl TryFromBytes for DisconnectReason {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        Self::try_from(u8::try_from_bytes(bytes)?)
    }
}

impl ToByteBuffer for DisconnectReason {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        (*self as u8).to_byte_buffer(buf)
    }
}

struct DisconnectProperties {
    session_expiry_interval: SessionExpiryInterval,
    reason_string: Option<ReasonString>,
    server_reference: Option<ServerReference>,
    user_property: Vec<UserProperty>,
}

impl SizedProperty for DisconnectProperties {
    fn property_len(&self) -> usize {
        let session_expiry_interval_len = Some(&self.session_expiry_interval)
            .map(|val| {
                if *val == SessionExpiryInterval::default() {
                    return 0;
                }

                return val.property_len();
            })
            .unwrap();

        let reason_string_len = self
            .reason_string
            .as_ref()
            .map(|val| val.property_len())
            .unwrap_or(0);

        let server_reference_len = self
            .server_reference
            .as_ref()
            .map(|val| val.property_len())
            .unwrap_or(0);

        let user_property_len = self
            .user_property
            .iter()
            .map(|val| val.property_len())
            .sum::<usize>();

        session_expiry_interval_len + reason_string_len + server_reference_len + user_property_len
    }
}

impl ToByteBuffer for DisconnectProperties {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let property_len = VarSizeInt::from(self.property_len());
        let len = property_len.len() + property_len.value() as usize;

        let result = &mut buf[0..len];
        let mut writer = ByteWriter::from(result);

        writer.write(&property_len);

        if self.session_expiry_interval != SessionExpiryInterval::default() {
            writer.write(&self.session_expiry_interval);
        }

        if let Some(val) = self.reason_string.as_ref() {
            writer.write(val);
        }

        if let Some(val) = self.server_reference.as_ref() {
            writer.write(val);
        }

        for val in self.user_property.iter() {
            writer.write(val)
        }

        result
    }
}

pub(crate) struct Disconnect {
    reason: DisconnectReason,
    properties: DisconnectProperties,
}

impl Disconnect {
    const FIXED_HDR: u8 = Self::PACKET_ID << 4;

    fn remaining_len(&self) -> VarSizeInt {
        let property_len = VarSizeInt::from(self.properties.property_len());
        VarSizeInt::from(
            mem::size_of::<DisconnectReason>() + property_len.len() + property_len.value() as usize,
        )
    }
}

impl PacketID for Disconnect {
    const PACKET_ID: u8 = 14;
}

impl SizedPacket for Disconnect {
    fn packet_len(&self) -> usize {
        let remaining_len = self.remaining_len();
        mem::size_of::<u8>() + remaining_len.len() + remaining_len.value() as usize
    }
}

impl TryFromBytes for Disconnect {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut builder = DisconnectBuilder::default();
        let mut reader = ByteReader::from(bytes);

        let fixed_hdr = reader.try_read::<u8>()?;
        if fixed_hdr != Self::FIXED_HDR {
            return None; // Invalid header
        }

        let remaining_len = reader.try_read::<VarSizeInt>()?;
        let packet_size =
            mem::size_of_val(&fixed_hdr) + remaining_len.len() + remaining_len.value() as usize;
        if packet_size > bytes.len() {
            return None; // Invalid packet size
        }

        let reason = reader.try_read::<DisconnectReason>()?;
        builder.reason(reason);

        let property_len = reader.try_read::<VarSizeInt>()?;
        if property_len.value() as usize > reader.remaining() {
            return None; // Invalid property length
        }

        for property in PropertyIterator::from(reader.get_buf()) {
            match property {
                Property::SessionExpiryInterval(_) => {
                    // The Session Expiry Interval MUST NOT be sent on a DISCONNECT by the Server
                    return None;
                }
                Property::ReasonString(val) => {
                    builder.reason_string(val.into());
                }
                Property::ServerReference(val) => {
                    builder.server_reference(val.into());
                }
                Property::UserProperty(val) => {
                    builder.user_property(val.into());
                }
                _ => {
                    return None;
                }
            }
        }

        builder.build()
    }
}

impl TryToByteBuffer for Disconnect {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let packet_len = self.packet_len();

        let result = buf.get_mut(0..packet_len)?;
        let mut writer = ByteWriter::from(result);

        writer.write(&Self::FIXED_HDR);

        let remaining_len = self.remaining_len();
        debug_assert!(remaining_len.value() as usize <= writer.remaining());
        writer.write(&remaining_len);

        writer.write(&self.reason);
        writer.write(&self.properties);

        Some(result)
    }
}

#[derive(Default)]
pub(crate) struct DisconnectBuilder {
    reason: Option<DisconnectReason>,
    session_expiry_interval: SessionExpiryInterval,
    reason_string: Option<ReasonString>,
    server_reference: Option<ServerReference>,
    user_property: Vec<UserProperty>,
}

impl DisconnectBuilder {
    pub(crate) fn reason(&mut self, val: DisconnectReason) -> &mut Self {
        self.reason = Some(val);
        self
    }

    pub(crate) fn session_expiry_interval(&mut self, val: u32) -> &mut Self {
        self.session_expiry_interval = val.into();
        self
    }

    pub(crate) fn reason_string(&mut self, val: String) -> &mut Self {
        self.reason_string = Some(val.into());
        self
    }

    pub(crate) fn server_reference(&mut self, val: String) -> &mut Self {
        self.server_reference = Some(val.into());
        self
    }

    pub(crate) fn user_property(&mut self, val: StringPair) -> &mut Self {
        self.user_property.push(val.into());
        self
    }

    pub(crate) fn build(self) -> Option<Disconnect> {
        let properties = DisconnectProperties {
            session_expiry_interval: self.session_expiry_interval,
            reason_string: self.reason_string,
            user_property: self.user_property,
            server_reference: self.server_reference,
        };

        Some(Disconnect {
            reason: self.reason?,
            properties,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::utils::PropertyID;

    const PACKET: [u8; 25] = [
        ((Disconnect::PACKET_ID as u8) << 4) as u8,
        23, // Remaining length
        (DisconnectReason::Success as u8),
        21, // Property length
        (ReasonString::PROPERTY_ID),
        0, // Reason string size
        7,
        b'S',
        b'u',
        b'c',
        b'c',
        b'e',
        b's',
        b's',
        (UserProperty::PROPERTY_ID),
        0, // User property key size
        3,
        b'k',
        b'e',
        b'y',
        0, // User property value size
        3,
        b'v',
        b'a',
        b'l',
    ];

    #[test]
    fn from_bytes_0() {
        let packet = Disconnect::try_from_bytes(&PACKET).unwrap();

        assert_eq!(packet.reason, DisconnectReason::Success);
        assert_eq!(
            String::from(packet.properties.reason_string.unwrap()),
            String::from("Success")
        );
        assert_eq!(packet.properties.user_property.len(), 1);
        assert_eq!(
            packet.properties.user_property[0],
            UserProperty::from((String::from("key"), String::from("val")))
        );
    }

    #[test]
    fn to_bytes_0() {
        let mut builder = DisconnectBuilder::default();

        builder.reason(DisconnectReason::Success);
        builder.reason_string(String::from("Success"));
        builder.user_property((String::from("key"), String::from("val")));

        let packet = builder.build().unwrap();

        let mut buf = [0u8; PACKET.len()];
        let result = packet.try_to_byte_buffer(&mut buf).unwrap();

        assert_eq!(result, PACKET);
    }
}
