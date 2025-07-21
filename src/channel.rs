pub struct Channel {
    pub outgoing_reliable_seq_num: u16,
    pub outgoing_unreliable_seq_num: u16,
    pub used_reliable_windows: u16,
    pub reliable_windows: Box<[u16]>, // u16 list with length peer::constants::ENET_PEER_RELIABLE_WINDOWS
    pub incoming_reliable_seq_num: u16,
    pub incoming_unreliable_seq_num: u16,

    pub incoming_reliable_commands: Vec<()>, // ENetList
    pub incoming_unreliable_commands: Vec<()>, // ENetList
}