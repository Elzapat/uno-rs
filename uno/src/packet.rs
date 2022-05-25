use crate::{
    card::{Card, Color},
    error::UnoError,
    lobby::LobbyId,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    iter::FromIterator,
    net::TcpStream,
    ops::{Bound, RangeBounds},
};
use tungstenite::{Message, WebSocket};
use uuid::Uuid;

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

pub fn read_socket(socket: &mut WebSocket<TcpStream>) -> Result<Packet> {
    if let Message::Binary(packet) = socket.read_message()? {
        Ok(parse_packet(packet))
    } else {
        Err(anyhow::Error::new(UnoError::MessageNotBinary))
    }
}

pub fn write_socket<A>(socket: &mut WebSocket<TcpStream>, command: Command, args: A) -> Result<()>
where
    A: Into<Args>,
{
    socket.write_message(Message::Binary(
        [
            &[command as u8],
            args.into().as_slice(),
            &[PACKET_DELIMITER],
        ]
        .concat(),
    ))?;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum _Packet {
    CreateLobby,
    LobbyCreated(LobbyId),
    LobbyDestroyed(LobbyId),
    JoinLobby(LobbyId),
    PlayerJoinedLobby {
        lobby_id: LobbyId,
        player_id: Uuid,
    },
    LeaveLobby(LobbyId),
    PlayerLeftLobby {
        lobby_id: LobbyId,
        player_id: Uuid,
    },
    LobbyInfo {
        lobby_id: LobbyId,
        players: Vec<String>,
    },
    Username(String),
    StartGame,
    GameEnd,
    PlayerScore(u32),
    // In game commmands
    PlayCard(Card),
    CardPlayed(Card),
    CardValidation(bool),
    DrawCard(Card),
    HandSize(u32),
    ChooseColor,
    ColorChosen(Color),
    CurrentColor(Color),
    Uno,
    StopUno,
    CounterUno,
    StopCounterUno,
    HaveToDrawCard,
    PassTurn,
    YourPlayerId(Uuid),
    // Other
    Error(String),
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
    GameEnd = 25,
    PlayerScore = 26,
    // In game commmands
    PlayCard = 30,
    CardPlayed = 31,
    CardValidation = 32,
    DrawCard = 33,
    HandSize = 34,
    ChooseColor = 35,
    ColorChosen = 36,
    CurrentColor = 37,
    Uno = 40,
    StopUno = 41,
    CounterUno = 42,
    StopCounterUno = 43,
    HaveToDrawCard = 44,
    PassTurn = 45,
    YourPlayerId = 50,
    // Other
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
            25 => Command::GameEnd,
            26 => Command::PlayerScore,
            30 => Command::PlayCard,
            31 => Command::CardPlayed,
            32 => Command::CardValidation,
            33 => Command::DrawCard,
            34 => Command::HandSize,
            35 => Command::ChooseColor,
            36 => Command::ColorChosen,
            37 => Command::CurrentColor,
            40 => Command::Uno,
            41 => Command::StopUno,
            42 => Command::CounterUno,
            43 => Command::StopCounterUno,
            44 => Command::HaveToDrawCard,
            45 => Command::PassTurn,
            50 => Command::YourPlayerId,
            _ => Command::Unknown,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Args(pub Vec<u8>);

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
