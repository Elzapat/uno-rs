use crate::player::Player;

pub type LobbyId = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lobby {
    pub id: LobbyId,
    pub players: Vec<Player>,
}

impl Lobby {
    pub fn new(id: LobbyId) -> Self {
        Lobby {
            id,
            players: vec![],
        }
    }
}
