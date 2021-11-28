use uno::prelude::*;
use crate::ServerResult;
use std::{
    net::TcpStream,
    io::{ Read, Write },
};

pub struct Client {
    pub socket: TcpStream,
    pub incoming_packets: Vec<packet::Packet>,
    pub in_lobby: Option<usize>,
    pub player: Option<Player>,
}

impl Client {
    pub fn new(socket: TcpStream) -> Client {
        Client {
            socket,
            incoming_packets: Vec::new(),
            in_lobby: None,
            player: None,
        }
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.player == other.player
    }
}
