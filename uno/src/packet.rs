use crate::error::{Error, Result, UnoError};
use std::{
    io::{Read, Write},
    iter::FromIterator,
    net::TcpStream,
    ops::{Bound, RangeBounds},
};

/// Delimiter between packets
pub const PACKET_DELIMITER: u8 = 255;
pub const ARG_DELIMITER: u8 = 254;

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

    for byte in buffer.into_iter().take(size) {
        if PACKET_DELIMITER == byte {
            packets.push(parse_packet(current_packet.drain(..).collect()));
        } else {
            current_packet.push(byte);
        }
    }

    Ok(packets)
}

pub fn write_socket<A>(socket: &mut TcpStream, command: Command, args: A) -> Result<()>
where
    A: Into<Args>,
{
    socket.write_all(
        &[
            &[command as u8],
            args.into().as_slice(),
            &[PACKET_DELIMITER],
        ]
        .concat(),
    )?;

    Ok(())
}

/// Command bytes
#[derive(Clone, Copy, Debug)]
pub enum Command {
    // Lobby commands
    Error = 0,
    CreateLobby = 1,
    LobbyCreated = 2,
    LobbyDestroyed = 3,
    JoinLobby = 4,
    PlayerJoinedLobby = 5,
    LeaveLobby = 6,
    PlayerLeftLobby = 7,
    LobbyInfo = 8,
    LobbiesInfo = 9,
    Username = 10,
    StartGame = 20,
    // In game commmands
    PlayCard = 30,
    DrawCard = 31,
    ChooseColor = 35,
    Uno = 40,
    //
    Unknown = 255,
}

impl From<u8> for Command {
    fn from(v: u8) -> Command {
        match v {
            0 => Command::Error,
            1 => Command::CreateLobby,
            2 => Command::LobbyCreated,
            3 => Command::LobbyDestroyed,
            4 => Command::JoinLobby,
            5 => Command::PlayerJoinedLobby,
            6 => Command::LeaveLobby,
            7 => Command::PlayerLeftLobby,
            8 => Command::LobbyInfo,
            9 => Command::LobbiesInfo,
            10 => Command::Username,
            20 => Command::StartGame,
            30 => Command::PlayCard,
            31 => Command::DrawCard,
            35 => Command::ChooseColor,
            40 => Command::Uno,
            _ => Command::Unknown,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Args(Vec<u8>);

impl Args {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn add_byte(&mut self, b: u8) {
        self.0.push(b);
    }

    pub fn get(&self, index: usize) -> Option<&u8> {
        self.0.get(index)
    }

    pub fn get_range<R>(&mut self, range: R) -> Vec<u8>
    where
        R: RangeBounds<usize>,
    {
        if let Bound::Included(&start) = range.start_bound() {
            if start > self.0.len() {
                return Vec::new();
            }
        }

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
