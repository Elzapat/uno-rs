use crate::{card::Color, prelude::*};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PlayerState {
    WaitingToPlay,
    PlayingCard,
    DrawingCard,
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
    pub score: u32,
    pub username: String,
    pub state: PlayerState,
}

impl Player {
    pub fn new(username: String) -> Player {
        Player {
            hand: Vec::new(),
            state: PlayerState::WaitingToPlay,
            score: 0,
            username,
        }
    }

    /// Check whether the player can play with his current hand
    pub fn can_play(&self, top_card: Card, current_color: Color) -> bool {
        self.hand
            .iter()
            .any(|card| card.can_be_played(top_card, current_color))
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.username == other.username
    }
}
