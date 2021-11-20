use std::iter::FromIterator;

#[derive(Debug)]
pub enum PacketError {
    ZeroSizePacket,
}

/// Delimiter between packets
pub const DELIMITER: u8 = 255;

/// Represents a fully parsed packet
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
pub enum Command {
    CreateGame = 0,
    JoinGame = 1,
    NumberOfPlayersInGame = 2,
    Unknown = 254,
}

impl From<u8> for Command {
    fn from(v: u8) -> Command {
        match v {
            0 => Command::CreateGame,
            1 => Command::JoinGame,
            2 => Command::NumberOfPlayersInGame,
            _ => Command::Unknown,
        }
    }
}

impl Into<u8> for Command {
    fn into(self) -> u8 {
        self as u8
    }
}

/// Arg bytes
pub enum Arg {
    Unknown = 254,
}

pub struct Args(Vec<Arg>);

impl From<u8> for Arg {
    fn from(v: u8) -> Arg {
        match v {
            _ => Arg::Unknown,
        }
    }
}

impl Into<u8> for Arg {
    fn into(self) -> u8 {
        self as u8
    }
}

impl Args {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn add_byte(&mut self, b: u8) {
        self.0.push(b.into());
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
