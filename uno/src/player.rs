use crate::card::{Card, Color};
use uuid::Uuid;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PlayerState {
    WaitingToPlay,
    PlayingCard,
    DrawingCard,
    ChoosingColorWild,
    ChoosingColorWildFour,
    /// Player has to choose a color and confirm Uno, keep state of both actions to proceed only
    /// when both are done
    ChoosingColorWildUno([bool; 2]),
    ChoosingColorWildFourUno([bool; 2]),
    Uno,
}

/// Structure to define a Uno player
#[derive(Clone, Debug, Eq)]
pub struct Player {
    pub id: Uuid,
    pub hand: Vec<Card>,
    pub score: u32,
    pub username: String,
    pub state: PlayerState,
    pub is_playing: bool,
}

impl Player {
    pub fn new(id: Uuid, username: String) -> Player {
        Player {
            id,
            hand: Vec::new(),
            state: PlayerState::WaitingToPlay,
            score: 0,
            username,
            is_playing: false,
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
        self.id == other.id
    }
}
