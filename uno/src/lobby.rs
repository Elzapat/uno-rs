use crate::player::Player;

pub type LobbyId = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lobby {
    pub id: LobbyId,
    pub players: Vec<Player>,
}
