//! ENet protocol definitions
//!
//! This module contains all the protocol structures and constants used by ENet
//! for network communication. All structures are packed and binary-compatible
//! with the original C implementation.

pub mod constants {
    pub const MINIMUM_MTU: u32 = 576;
    pub const MAXIMUM_MTU: u32 = 4096;
    pub const MAXIMUM_PACKET_COMMANDS: u32 = 32;
    pub const MINIMUM_WINDOW_SIZE: u32 = 4096;
    pub const MAXIMUM_WINDOW_SIZE: u32 = 65536;
    pub const MINIMUM_CHANNEL_COUNT: u32 = 1;
    pub const MAXIMUM_CHANNEL_COUNT: u32 = 255;
    pub const MAXIMUM_PEER_ID: u32 = 0xFFF;
    pub const MAXIMUM_FRAGMENT_COUNT: u32 = 1024 * 1024;
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolCommand {
    None = 0,
    Acknowledge = 1,
    Connect = 2,
    VerifyConnect = 3,
    Disconnect = 4,
    Ping = 5,
    SendReliable = 6,
    SendUnreliable = 7,
    SendFragment = 8,
    SendUnsequenced = 9,
    BandwidthLimit = 10,
    ThrottleConfigure = 11,
    SendUnreliableFragment = 12,
}

impl ProtocolCommand {
    pub const COUNT: u8 = 13;
    pub const MASK: u8 = 0x0F;

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Acknowledge),
            2 => Some(Self::Connect),
            3 => Some(Self::VerifyConnect),
            4 => Some(Self::Disconnect),
            5 => Some(Self::Ping),
            6 => Some(Self::SendReliable),
            7 => Some(Self::SendUnreliable),
            8 => Some(Self::SendFragment),
            9 => Some(Self::SendUnsequenced),
            10 => Some(Self::BandwidthLimit),
            11 => Some(Self::ThrottleConfigure),
            12 => Some(Self::SendUnreliableFragment),
            _ => None,
        }
    }
}

pub mod flags {
    pub const COMMAND_FLAG_ACKNOWLEDGE: u8 = 1 << 7;
    pub const COMMAND_FLAG_UNSEQUENCED: u8 = 1 << 6;

    pub const HEADER_FLAG_COMPRESSED: u16 = 1 << 14;
    pub const HEADER_FLAG_SENT_TIME: u16 = 1 << 15;
    pub const HEADER_FLAG_MASK: u16 = HEADER_FLAG_COMPRESSED | HEADER_FLAG_SENT_TIME;

    pub const HEADER_SESSION_MASK: u16 = 3 << 12;
    pub const HEADER_SESSION_SHIFT: u16 = 12;
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ProtocolHeader {
    pub peer_id: u16,
    pub sent_time: u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ProtocolCommandHeader {
    pub command: u8,
    pub channel_id: u8,
    pub reliable_sequence_number: u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ProtocolAcknowledge {
    pub header: ProtocolCommandHeader,
    pub received_reliable_sequence_number: u16,
    pub received_sent_time: u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ProtocolConnect {
    pub header: ProtocolCommandHeader,
    pub outgoing_peer_id: u16,
    pub incoming_session_id: u8,
    pub outgoing_session_id: u8,
    pub mtu: u32,
    pub window_size: u32,
    pub channel_count: u32,
    pub incoming_bandwidth: u32,
    pub outgoing_bandwidth: u32,
    pub packet_throttle_interval: u32,
    pub packet_throttle_acceleration: u32,
    pub packet_throttle_deceleration: u32,
    pub connect_id: u32,
    pub data: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ProtocolVerifyConnect {
    pub header: ProtocolCommandHeader,
    pub outgoing_peer_id: u16,
    pub incoming_session_id: u8,
    pub outgoing_session_id: u8,
    pub mtu: u32,
    pub window_size: u32,
    pub channel_count: u32,
    pub incoming_bandwidth: u32,
    pub outgoing_bandwidth: u32,
    pub packet_throttle_interval: u32,
    pub packet_throttle_acceleration: u32,
    pub packet_throttle_deceleration: u32,
    pub connect_id: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ProtocolBandwidthLimit {
    pub header: ProtocolCommandHeader,
    pub incoming_bandwidth: u32,
    pub outgoing_bandwidth: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ProtocolThrottleConfigure {
    pub header: ProtocolCommandHeader,
    pub packet_throttle_interval: u32,
    pub packet_throttle_acceleration: u32,
    pub packet_throttle_deceleration: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ProtocolDisconnect {
    pub header: ProtocolCommandHeader,
    pub data: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ProtocolPing {
    pub header: ProtocolCommandHeader,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ProtocolSendReliable {
    pub header: ProtocolCommandHeader,
    pub data_length: u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ProtocolSendUnreliable {
    pub header: ProtocolCommandHeader,
    pub unreliable_sequence_number: u16,
    pub data_length: u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ProtocolSendUnsequenced {
    pub header: ProtocolCommandHeader,
    pub unsequenced_group: u16,
    pub data_length: u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ProtocolSendFragment {
    pub header: ProtocolCommandHeader,
    pub start_sequence_number: u16,
    pub data_length: u16,
    pub fragment_count: u32,
    pub fragment_number: u32,
    pub total_length: u32,
    pub fragment_offset: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    Header(ProtocolCommandHeader),
    Acknowledge(ProtocolAcknowledge),
    Connect(ProtocolConnect),
    VerifyConnect(ProtocolVerifyConnect),
    Disconnect(ProtocolDisconnect),
    Ping(ProtocolPing),
    SendReliable(ProtocolSendReliable),
    SendUnreliable(ProtocolSendUnreliable),
    SendUnsequenced(ProtocolSendUnsequenced),
    SendFragment(ProtocolSendFragment),
    BandwidthLimit(ProtocolBandwidthLimit),
    ThrottleConfigure(ProtocolThrottleConfigure),
}

impl Default for Protocol {
    fn default() -> Self {
        Self::Header(ProtocolCommandHeader {
            command: 0, channel_id: 0, reliable_sequence_number: 0
        })
    }
}

impl Protocol {
    pub fn header(&self) -> &ProtocolCommandHeader {
        match self {
            Protocol::Header(header) => header,
            Protocol::Acknowledge(ack) => &ack.header,
            Protocol::Connect(conn) => &conn.header,
            Protocol::VerifyConnect(verify) => &verify.header,
            Protocol::Disconnect(disc) => &disc.header,
            Protocol::Ping(ping) => &ping.header,
            Protocol::SendReliable(reliable) => &reliable.header,
            Protocol::SendUnreliable(unreliable) => &unreliable.header,
            Protocol::SendUnsequenced(unsequenced) => &unsequenced.header,
            Protocol::SendFragment(fragment) => &fragment.header,
            Protocol::BandwidthLimit(bandwidth) => &bandwidth.header,
            Protocol::ThrottleConfigure(throttle) => &throttle.header,
        }
    }

    pub fn header_mut(&mut self) -> &mut ProtocolCommandHeader {
        match self {
            Protocol::Header(header) => header,
            Protocol::Acknowledge(ack) => &mut ack.header,
            Protocol::Connect(conn) => &mut conn.header,
            Protocol::VerifyConnect(verify) => &mut verify.header,
            Protocol::Disconnect(disc) => &mut disc.header,
            Protocol::Ping(ping) => &mut ping.header,
            Protocol::SendReliable(reliable) => &mut reliable.header,
            Protocol::SendUnreliable(unreliable) => &mut unreliable.header,
            Protocol::SendUnsequenced(unsequenced) => &mut unsequenced.header,
            Protocol::SendFragment(fragment) => &mut fragment.header,
            Protocol::BandwidthLimit(bandwidth) => &mut bandwidth.header,
            Protocol::ThrottleConfigure(throttle) => &mut throttle.header,
        }
    }

    pub fn command(&self) -> Option<ProtocolCommand> {
        ProtocolCommand::from_u8(self.header().command & ProtocolCommand::MASK)
    }
}

pub fn command_size(command_number: u8) -> usize {
    use std::mem;

    match command_number {
        0 => 0,
        1 => mem::size_of::<ProtocolAcknowledge>(),
        2 => mem::size_of::<ProtocolConnect>(),
        3 => mem::size_of::<ProtocolVerifyConnect>(),
        4 => mem::size_of::<ProtocolDisconnect>(),
        5 => mem::size_of::<ProtocolPing>(),
        6 => mem::size_of::<ProtocolSendReliable>(),
        7 => mem::size_of::<ProtocolSendUnreliable>(),
        8 => mem::size_of::<ProtocolSendFragment>(),
        9 => mem::size_of::<ProtocolSendUnsequenced>(),
        10 => mem::size_of::<ProtocolBandwidthLimit>(),
        11 => mem::size_of::<ProtocolThrottleConfigure>(),
        12 => mem::size_of::<ProtocolSendFragment>(),
        _ => 0,
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_struct_sizes() {
        assert_eq!(mem::size_of::<ProtocolHeader>(), 4);
        assert_eq!(mem::size_of::<ProtocolCommandHeader>(), 4);
        assert_eq!(mem::size_of::<ProtocolPing>(), 4);
        assert_eq!(mem::size_of::<ProtocolConnect>(), 48);
        assert_eq!(mem::size_of::<ProtocolVerifyConnect>(), 44);
    }

    #[test]
    fn test_protocol_command_conversion() {
        assert_eq!(ProtocolCommand::from_u8(0), Some(ProtocolCommand::None));
        assert_eq!(
            ProtocolCommand::from_u8(1),
            Some(ProtocolCommand::Acknowledge)
        );
        assert_eq!(ProtocolCommand::from_u8(255), None);
    }

    #[test]
    fn test_flags() {
        assert_eq!(flags::COMMAND_FLAG_ACKNOWLEDGE, 128);
        assert_eq!(flags::COMMAND_FLAG_UNSEQUENCED, 64);
        assert_eq!(flags::HEADER_FLAG_COMPRESSED, 16384);
        assert_eq!(flags::HEADER_FLAG_SENT_TIME, 32768);
    }
}
