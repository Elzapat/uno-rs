use std::{
    iter::FromIterator,
    ops::RangeBounds,
    net::TcpStream,
    io::{ Read, Write },
};
use crate::error::{ Result, Error, UnoError };

/// Delimiter between packets
pub const DELIMITER: u8 = 255;

/// Represents a fully parsed packet
#[derive(Clone, Debug)]
pub struct Packet {
    pub command: Command,
    pub args: Args,
}

/// Parse a packet from a Vec of bytes
pub fn parse_packet(mut packet: Vec<u8>) -> Packet {
    Packet {
        command: packet[0].into(),
        args: packet.drain(1..).collect(),
    }
}

pub fn read_socket(socket: &mut TcpStream) -> Result<Vec<Packet>> {
    let mut buffer = [0; 128];
    let size = socket.read(&mut buffer)?;

    if size < 1 {
        return Err(Error::UnoError(UnoError::Disconnected));
    }

    let mut current_packet = Vec::new();
    let mut packets = Vec::new();

    for i in 0..size {
        if DELIMITER == buffer[i] {
            packets.push(parse_packet(current_packet.drain(..).collect()));
        } else {
            current_packet.push(buffer[i]);
        }
    }

    Ok(packets)
}

pub fn write_socket<A>(socket: &mut TcpStream, command: Command, args: A) -> Result<()>
where
    A: Into<Args>,
{
    socket.write(&[&[command as u8], args.into().as_slice(), &[DELIMITER]].concat())?;

    Ok(())
}

/// Command bytes
#[derive(Clone, Copy, Debug)]
pub enum Command {
    Error = 0,
    CreateLobby = 1,
    JoinLobby = 2,
    LeaveLobby = 3,
    LobbiesInfo = 4,
    LobbyInfo = 5,
    Username = 6,
    Unknown = 255,
}

impl From<u8> for Command {
    fn from(v: u8) -> Command {
        match v {
            0 => Command::Error,
            1 => Command::CreateLobby,
            2 => Command::JoinLobby,
            3 => Command::LeaveLobby,
            4 => Command::LobbiesInfo,
            5 => Command::LobbyInfo,
            6 => Command::Username,
            _ => Command::Unknown,
        }
    }
}

impl Into<u8> for Command {
    fn into(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Debug)]
pub struct Args(Vec<u8>);

impl Args {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn add_byte(&mut self, b: u8) {
        self.0.push(b.into());
    }

    pub fn get(&self, index: usize) -> Option<&u8> {
        self.0.get(index)
    }

    pub fn get_range<R>(&mut self, range: R) -> Vec<u8>
    where
        R: RangeBounds<usize>,
    {
        self.0.drain(range).collect()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0[..]
    }
}

impl From<u8> for Args {
    fn from(v: u8) -> Args {
        Self(vec![v])
    }
}

impl From<&[u8]> for Args {
    fn from(v: &[u8]) -> Self {
        Self(v.to_vec())
    }
}

impl From<Vec<u8>> for Args {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl FromIterator<u8> for Args {
    fn from_iter<I: IntoIterator<Item = u8>>(iter: I) -> Self {
        let mut args = Self::new();

        for b in iter {
            args.add_byte(b);
        }

        args
    }
}
