use std::{cell::RefCell, net::Ipv4Addr, rc::Rc};
use crate::{channel::Channel, command::OutgoingCommand, packet::Packet, peer::constants::{PEER_STATE_ACKNOWLEDGING_DISCONNECT, PEER_STATE_CONNECTED, PEER_STATE_DISCONNECTED, PEER_STATE_DISCONNECTING, PEER_STATE_DISCONNECT_LATER, PEER_STATE_ZOMBIE}, protocol::{command_size, constants::MAXIMUM_PEER_ID, flags::{COMMAND_FLAG_ACKNOWLEDGE, COMMAND_FLAG_UNSEQUENCED}, Protocol, ProtocolCommand, ProtocolCommandHeader, ProtocolThrottleConfigure}};

pub mod constants {
    use crate::protocol::constants::MAXIMUM_PACKET_COMMANDS;

    pub const PEER_STATE_DISCONNECTED: u32                = 0;
    pub const PEER_STATE_CONNECTING: u32                  = 1;
    pub const PEER_STATE_ACKNOWLEDGING_CONNECT: u32       = 2;
    pub const PEER_STATE_CONNECTION_PENDING: u32          = 3;
    pub const PEER_STATE_CONNECTION_SUCCEEDED: u32        = 4;
    pub const PEER_STATE_CONNECTED: u32                   = 5;
    pub const PEER_STATE_DISCONNECT_LATER: u32            = 6;
    pub const PEER_STATE_DISCONNECTING: u32               = 7;
    pub const PEER_STATE_ACKNOWLEDGING_DISCONNECT: u32    = 8;
    pub const PEER_STATE_ZOMBIE: u32                      = 9;

    pub const BUFFER_MAXIMUM: u32 = 1 + 2 * MAXIMUM_PACKET_COMMANDS;

    pub const PEER_DEFAULT_ROUND_TRIP_TIME: u32      = 500;
    pub const PEER_DEFAULT_PACKET_THROTTLE: u32      = 32;
    pub const PEER_PACKET_THROTTLE_SCALE: u32        = 32;
    pub const PEER_PACKET_THROTTLE_COUNTER: u32      = 7; 
    pub const PEER_PACKET_THROTTLE_ACCELERATION: u32 = 2;
    pub const PEER_PACKET_THROTTLE_DECELERATION: u32 = 2;
    pub const PEER_PACKET_THROTTLE_INTERVAL: u32     = 5000;
    pub const PEER_PACKET_LOSS_SCALE: u32            = 1 << 16;
    pub const PEER_PACKET_LOSS_INTERVAL: u32         = 10000;
    pub const PEER_WINDOW_SIZE_SCALE: u32            = 64 * 1024;
    pub const PEER_TIMEOUT_LIMIT: u32                = 32;
    pub const PEER_TIMEOUT_MINIMUM: u32              = 5000;
    pub const PEER_TIMEOUT_MAXIMUM: u32              = 30000;
    pub const PEER_PING_INTERVAL: u32                = 500;
    pub const PEER_UNSEQUENCED_WINDOWS: u32          = 64;
    pub const PEER_UNSEQUENCED_WINDOW_SIZE: u32      = 1024;
    pub const PEER_FREE_UNSEQUENCED_WINDOWS: u32     = 32;
    pub const PEER_RELIABLE_WINDOWS: u32             = 16;
    pub const PEER_RELIABLE_WINDOW_SIZE: u32         = 0x1000;
    pub const PEER_FREE_RELIABLE_WINDOWS: u32        = 8;
    
    pub const PEER_FLAG_NEEDS_DISPATCH: u32        = 1 << 0;
    pub const PEER_FLAG_CONTINUE_SENDING: u32        = 1 << 1;
}

pub struct Peer<'a> {
    pub dispatch_list: Option<()>,
    pub host: Option<()>,
    
    pub outgoing_peer_id: u16,
    pub incoming_peer_id: u16,
    pub connect_id: u32,
    
    pub outgoing_session_id: u8,
    pub incoming_session_id: u8,
    
    pub address: Ipv4Addr,
    pub data: Option<()>, // void ptr
    
    pub state: u32,
    
    pub channels: Vec<Channel>, // ENetChannel*
    pub channel_count: usize,

    pub incoming_bandwidth: u32,
    pub outgoing_bandwidth: u32,
    
    pub incoming_bandwidth_throttle_epoch: u32,
    pub outgoing_bandwidth_throttle_epoch: u32,
    
    pub incoming_data_total: u32,
    pub outgoing_data_total: u32,

    pub last_send_time: u32,
    pub last_receive_time: u32,
    pub next_timeout: u32,
    pub earliest_timeout: u32,

    pub packet_loss_epoch: u32,
    pub packets_sent: u32,
    pub packets_lost: u32,
    pub packet_loss: u32,
    pub packet_loss_variance: u32,
    pub packet_throttle: u32,
    pub packet_throttle_limit: u32,
    pub packet_throttle_counter: u32,
    pub packet_throttle_epoch: u32,
    pub packet_throttle_accel: u32,
    pub packet_throttle_decel: u32,
    pub packet_throttle_interval: u32,
    
    pub ping_interval: u32,
    pub timeout_limit: u32,
    pub timeout_minimum: u32,
    pub timeout_maximum: u32,
    
    pub last_roundtrip_time: u32,
    pub lowest_roundtrip_time: u32,
    
    pub last_roundtrip_time_variance: u32,
    pub highest_roundtrip_time_variance: u32,
    
    pub roundtrip_time: u32,
    pub roundtrip_time_variance: u32,

    pub mtu: u32,
    pub window_size: u32,
    pub reliable_data_in_transit: u32,
    pub outgoing_reliable_seq_num: u16,

    pub acknowledgements: Vec<()>, // ENetList
    pub sent_reliable_commands: Vec<()>, // ENetList
    pub outgoing_send_reliable_commands: Vec<OutgoingCommand<'a>>, // ENetList
    pub outgoing_commands: Vec<OutgoingCommand<'a>>, // ENetList
    pub dispatched_commands: Vec<()>, // ENetList

    pub flags: u16,
    pub reserved: u16,
    pub incoming_unsequenced_group: u16,
    pub outgoing_unsequenced_group: u16,

    pub unsequenced_window: Box<[u32]>, // size constants::PEER_UNSEQUENCED_WINDOW_SIZE / 32
    pub event_data: u32,
    pub total_waiting_data: usize
}

impl<'a> Peer<'a> {
    pub fn disconnect(&mut self, data: u32) {
        if self.state == PEER_STATE_DISCONNECTING || self.state == PEER_STATE_DISCONNECTED ||
           self.state == PEER_STATE_ACKNOWLEDGING_DISCONNECT || self.state == PEER_STATE_ZOMBIE {
            return
        }


    }

    pub fn throttle(&mut self, rtt: u32) -> i32 {
        if self.last_roundtrip_time <= self.last_roundtrip_time_variance {
            self.packet_throttle = self.packet_throttle_limit;
        } else if rtt <= self.last_roundtrip_time {
            self.packet_throttle += self.packet_throttle_accel;

            if self.packet_throttle > self.packet_throttle_limit {
                self.packet_throttle = self.packet_throttle_limit;
            }

            return 1;
        } else if rtt > self.last_roundtrip_time + 2 * self.last_roundtrip_time_variance {
            if self.packet_throttle > self.packet_throttle_decel {
                self.packet_throttle -= self.packet_throttle_decel;
            } else {
                self.packet_throttle = 0;
            }

            return -1;
        } 

        return 0;
    }

    pub fn receive(&mut self, channel_id: Option<u8>) -> Option<Packet<'a>> {
        if self.dispatched_commands.is_empty() {
            return None;
        }

        let ch_id = 0;
        if channel_id.is_none() {
            
        }

        None
    }

    pub fn on_connect(&mut self) {
        if self.state == PEER_STATE_CONNECTED && self.state != PEER_STATE_DISCONNECT_LATER {
            if self.incoming_bandwidth != 0 {
                // self.host.bandwidth_limited_peers += 1;
            }
            // self.host.connected_peers += 1;
        }
    }

    pub fn on_disconnect(&mut self) {
        if self.state == PEER_STATE_CONNECTED || self.state == PEER_STATE_DISCONNECT_LATER {
            if self.incoming_bandwidth != 0 {
                // self.host.bandwidth_limited_peers -= 1;
            }
            // self.host.connected_peers -= 1;
        }
    }

    pub fn reset(&mut self) {
        self.on_disconnect();

        self.outgoing_peer_id = MAXIMUM_PEER_ID as u16;
        self.connect_id = 0;

        self.state = PEER_STATE_DISCONNECTED;

        self.incoming_bandwidth = 0;
        self.outgoing_bandwidth = 0;
    }

    pub fn has_outgoing_commands(&self) -> bool {
        !(self.outgoing_commands.is_empty() && self.outgoing_send_reliable_commands.is_empty() && self.sent_reliable_commands.is_empty())
    }

    // pub fn queue_outgoing_command<'a>(&mut self, command: Protocol, packet: &mut Packet<'a>, offset: u32, length: u16) -> OutgoingCommand {
    //     if true /* maybe packet should be an option :p */ {
    //         packet.ref_count += 1;
    //     }


    // }

    pub fn throttle_configure(&'a mut self, interval: u32, accel: u32, decel: u32) {
        self.packet_throttle_interval = interval;
        self.packet_throttle_accel = accel;
        self.packet_throttle_decel = decel;

        let command = Protocol::ThrottleConfigure(ProtocolThrottleConfigure 
        { 
            header: ProtocolCommandHeader { command: ProtocolCommand::ThrottleConfigure as u8 | ProtocolCommand::Acknowledge as u8, channel_id: 0xFF, reliable_sequence_number: 0 },
            packet_throttle_interval: interval,
            packet_throttle_acceleration: accel,
            packet_throttle_deceleration: decel,
        });

        // possible impl
        self.queue_outgoing_command(command, None, 0, 0);
    }

    pub fn queue_outgoing_command(&'a mut self, command: Protocol, packet: Option<Rc<RefCell<Packet<'a>>>>, offset: u32, length: u16) -> OutgoingCommand<'a> {
        let mut cmd = OutgoingCommand::default();

        cmd.command = command;
        cmd.fragment_offset = offset;
        cmd.fragment_length = length as u32;
        
        cmd.packet = packet;
        if let Some(pck) = &mut cmd.packet {
            pck.borrow_mut().ref_count += 1;
        }
        
        self.setup_outgoing_command(&mut cmd);
        cmd
    }

    pub fn setup_outgoing_command(&mut self, cmd: &mut OutgoingCommand<'a>) {
        self.outgoing_data_total = command_size(cmd.command.header().command) as u32 + cmd.fragment_length;

        if cmd.command.header().channel_id == 0xFF {
            self.outgoing_reliable_seq_num += 1;

            cmd.reliable_seq_num = self.outgoing_reliable_seq_num;
            cmd.unreliable_seq_num = 0;
        } else {
            let channel = self.channels.get_mut(cmd.command.header().channel_id as usize).expect("failed to get channel");

            if cmd.command.header().command & COMMAND_FLAG_ACKNOWLEDGE != 0 {
                channel.outgoing_reliable_seq_num += 1;
                channel.outgoing_unreliable_seq_num = 0;

                cmd.reliable_seq_num = channel.outgoing_reliable_seq_num;
                cmd.unreliable_seq_num = 0;
            } else if cmd.command.header().command & COMMAND_FLAG_UNSEQUENCED != 0 {
                self.outgoing_unsequenced_group += 1;

                cmd.reliable_seq_num = 0;
                cmd.unreliable_seq_num = 0;
            } else {
                if cmd.fragment_offset == 0 {
                    channel.outgoing_unreliable_seq_num += 1;
                }

                cmd.reliable_seq_num = channel.outgoing_reliable_seq_num;
                cmd.unreliable_seq_num = channel.outgoing_unreliable_seq_num;
            }
        }

        cmd.send_attempts = 0;
        cmd.sent_time = 0;
        cmd.roundtrip_timeout = 0;
        cmd.command.header_mut().reliable_sequence_number = cmd.reliable_seq_num.to_be();
        // cmd.queue_time = self.host.total_queued + 1;
        // self.host.total_queued += 1;

        match cmd.command.header().command & ProtocolCommand::MASK {
            val if val == ProtocolCommand::SendUnreliable as u8 => {
                // cmd.command.header_mut().
            },

            val if val == ProtocolCommand::SendUnsequenced as u8 => {

            },

            _ => {}
        }

        if cmd.command.header().command & COMMAND_FLAG_ACKNOWLEDGE != 0 &&
           cmd.packet.is_some() {
            // push cmd
            self.outgoing_send_reliable_commands.push(cmd.clone());
        } else {
            // push cmd
            self.outgoing_commands.push(cmd.clone());
        }
    }
}