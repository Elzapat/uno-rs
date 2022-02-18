use crate::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PlayerState {
    WaitingToPlay,
    PlayingCard,
    ChoosingColorWild,
    ChoosingColorWildFour,
    ChoosingColorWildUno,
    ChoosingColorWildFourUno,
    Uno,
}

/// Structure to define a Uno player
#[derive(Clone, Debug)]
pub struct Player {
    pub hand: Vec<Card>,
    pub is_playing: bool,
    pub score: u32,
    pub username: String,
    pub state: PlayerState,
}

impl Player {
    pub fn new(username: String) -> Player {
        Player {
            hand: Vec::new(),
            is_playing: false,
            state: PlayerState::WaitingToPlay,
            score: 0,
            username,
        }
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.username == other.username
    }
}
