use crate::{packet::Packet, peer::Peer};

pub enum EventType {
    None = 0,
    Connect = 1,
    Disconnect = 2,
    Receive = 3
}

pub struct Event<'a> {
    pub event_type: EventType,
    pub peer: Peer<'a>,
    pub channel_id: u8,
    pub data: u32,
    pub packet: Packet<'a>
}
