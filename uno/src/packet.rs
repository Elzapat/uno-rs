use std::{
    iter::FromIterator,
    ops::RangeBounds,
};

#[derive(Debug)]
pub enum PacketError {
    ZeroSizePacket,
}

/// Delimiter between packets
pub const DELIMITER: u8 = 255;

/// Represents a fully parsed packet
#[derive(Clone, Debug)]
pub struct Packet {
    pub command: Command,
    pub args: Args,
}

impl Packet {
    /// Parse a packet from a Vec of bytes
    pub fn parse_packet(mut packet: Vec<u8>) -> Result<Self, PacketError> {
        if packet.len() < 1 {
            return Err(PacketError::ZeroSizePacket);
        }

        Ok(Self {
            command: packet[0].into(),
            args: packet.drain(1..).collect(),
        })
    }
}

/// Current state when reading TCP stream
pub enum State {
    AcceptingCommand,
    AcceptingArgs,
}

/// Command bytes
#[derive(Clone, Debug)]
pub enum Command {
    Error = 0,
    CreateLobby = 1,
    JoinLobby = 2,
    LobbiesInfo = 3,
    Unknown = 255,
}

impl From<u8> for Command {
    fn from(v: u8) -> Command {
        match v {
            0 => Command::Error,
            1 => Command::CreateLobby,
            2 => Command::JoinLobby,
            3 => Command::LobbiesInfo,
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
