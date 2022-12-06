use crate::{
    codec::*,
    core::{base_types::QoS, utils::PacketID},
};
use std::collections::VecDeque;

pub(crate) fn tx_action_id(packet: &TxPacket) -> usize {
    match packet {
        TxPacket::Connect(_) => 0,
        TxPacket::Auth(_) => 0,
        TxPacket::Subscribe(subscribe) => {
            ((SubackRx::PACKET_ID as usize) << 24)
                | ((subscribe.packet_identifier.get() as usize) << 8)
        }
        TxPacket::Unsubscribe(unsubscribe) => {
            ((UnsubackRx::PACKET_ID as usize) << 24)
                | ((unsubscribe.packet_identifier.get() as usize) << 8)
        }
        TxPacket::Pingreq(_) => (PingrespRx::PACKET_ID as usize) << 24,
        TxPacket::Publish(publish) => match publish.qos {
            QoS::AtLeastOnce => {
                (PubackRx::PACKET_ID as usize) << 24
                    | (publish
                        .packet_identifier
                        .map(|val| -> usize { val.get() as usize })
                        .unwrap()
                        << 8)
            }
            QoS::ExactlyOnce => {
                (PubrecRx::PACKET_ID as usize) << 24
                    | (publish
                        .packet_identifier
                        .map(|val| -> usize { val.get() as usize })
                        .unwrap()
                        << 8)
            }
            _ => unreachable!("Method cannot be called for QoS 0."),
        },
        TxPacket::Pubrel(pubrel) => {
            (PubcompRx::PACKET_ID as usize) << 24 | ((pubrel.packet_identifier.get() as usize) << 8)
        }
        TxPacket::Pubrec(pubrec) => {
            (PubrelRx::PACKET_ID as usize) << 24 | ((pubrec.packet_identifier.get() as usize) << 8)
        }
        _ => unreachable!("Unexpected packet type."),
    }
}

pub(crate) fn rx_action_id(packet: &RxPacket) -> usize {
    match packet {
        RxPacket::Connack(_) => 0,
        RxPacket::Auth(_) => 0,
        RxPacket::Suback(suback) => {
            ((SubackRx::PACKET_ID as usize) << 24)
                | ((suback.packet_identifier.get() as usize) << 8)
        }
        RxPacket::Unsuback(unsuback) => {
            ((UnsubackRx::PACKET_ID as usize) << 24)
                | ((unsuback.packet_identifier.get() as usize) << 8)
        }
        RxPacket::Pingresp(_) => (PingrespRx::PACKET_ID as usize) << 24,
        RxPacket::Puback(puback) => {
            (PubackRx::PACKET_ID as usize) << 24 | ((puback.packet_identifier.get() as usize) << 8)
        }
        RxPacket::Pubrec(pubrec) => {
            (PubrecRx::PACKET_ID as usize) << 24 | ((pubrec.packet_identifier.get() as usize) << 8)
        }
        RxPacket::Pubrel(pubrel) => {
            (PubrelRx::PACKET_ID as usize) << 24 | ((pubrel.packet_identifier.get() as usize) << 8)
        }
        RxPacket::Pubcomp(pubcomp) => {
            (PubcompRx::PACKET_ID as usize) << 24
                | ((pubcomp.packet_identifier.get() as usize) << 8)
        }
        _ => unreachable!("Unexpected packet type."),
    }
}

pub(crate) fn linear_search_by_key<K, V>(deque: &VecDeque<(K, V)>, key: K) -> Option<usize>
where
    K: Copy + PartialEq,
{
    deque.iter().position(|(k, _)| *k == key)
}
