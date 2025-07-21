use std::{cell::RefCell, rc::Rc};

use crate::{packet::Packet, protocol::Protocol};

pub struct Acknowledgement {
    pub acknowledgement_list: Vec<()>, // ENetListNode
    pub sent_time: u32,
    pub command: Protocol,
}

#[derive(Default, Clone)]
pub struct OutgoingCommand<'a> {
    pub outgoing_command_list: Vec<()>, // ENetListNode
    pub reliable_seq_num: u16,
    pub unreliable_seq_num: u16,
    pub sent_time: u32,
    pub roundtrip_timeout: u32,
    pub queue_time: u32,
    pub fragment_offset: u32,
    pub fragment_length: u32,
    pub send_attempts: u16,
    pub command: Protocol,
    pub packet: Option<Rc<RefCell<Packet<'a>>>>,
}

pub struct IncomingCommand<'a> {
    pub incomingcommand_list: Vec<()>, // ENetListNode
    pub reliable_seq_num: u16,
    pub unreliable_seq_num: u16,
    pub command: Protocol,
    pub fragment_count: u32,
    pub fragments_remaining: u32,
    pub fragments: Vec<u32>, // u32*
    pub packet: Packet<'a>,
}