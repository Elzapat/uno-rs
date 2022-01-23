use std::net::TcpStream;
use uno::{
    packet::{ARG_DELIMITER, PACKET_DELIMITER},
    prelude::*,
};
use uuid::Uuid;

#[derive(Debug)]
pub struct Client {
    pub id: Uuid,
    pub socket: TcpStream,
    pub incoming_packets: Vec<packet::Packet>,
    pub in_lobby: Option<usize>,
    pub player: Option<Player>,
}

impl Client {
    pub fn new(socket: TcpStream) -> Client {
        let mut id = Uuid::new_v4();
        while id.as_bytes().contains(&ARG_DELIMITER) || id.as_bytes().contains(&PACKET_DELIMITER) {
            id = Uuid::new_v4();
        }

        Client {
            id,
            socket,
            incoming_packets: Vec::new(),
            in_lobby: None,
            player: None,
        }
    }
}

impl Eq for Client {}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
