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

    pub fn read(&mut self) -> ServerResult<()> {
        let mut buffer = [0; 32];
        let size = self.socket.read(&mut buffer)?;
        let mut current_packet = Vec::new();

        for i in 0..size {
            if uno::packet::DELIMITER == buffer[i] {
                let packet = Packet::parse_packet(current_packet.clone())?;
                self.incoming_packets.push(packet);
                current_packet.drain(..);
            } else {
                current_packet.push(buffer[i]);
            }
        }

        Ok(())
    }

    pub fn send<A>(&mut self, command: packet::Command, args: A) -> ServerResult<()>
    where
        A: Into<packet::Args>
    {
        self.socket.write(&[&[command as u8], args.into().as_slice(), &[packet::DELIMITER]].concat())?;

        Ok(())
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.player == other.player
    }
}
