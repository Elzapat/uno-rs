use uno::prelude::*;
use std::net::TcpStream;

pub struct Client {
    pub socket: TcpStream,
    pub packet_state: packet::State,
    pub current_packet: Vec<u8>,
    pub player: Option<Player>,
}

impl Client {
    pub fn new(socket: TcpStream) -> Client {
        Client {
            socket,
            packet_state: packet::State::AcceptingCommand,
            current_packet: Vec::new(),
            player: None,
        }
    }
}
