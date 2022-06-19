use crate::card::{Card, Color, Value};
use bevy_ecs::prelude::Component;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PlayerState {
    WaitingToPlay,
    PlayingCard,
    DrawingCard,
    ChoosingColorWild,
    ChoosingColorWildFour,
    /// Player has to choose a color and confirm Uno, keep state of both actions to proceed only
    /// when both are done
    ChoosingColorWildUno {
        uno_done: bool,
        color_chosen: bool,
    },
    ChoosingColorWildFourUno {
        uno_done: bool,
        color_chosen: bool,
    },
    Uno,
}

/// Structure to define a Uno player
#[derive(Component, Clone, Debug)]
pub struct Player {
    pub hand: Vec<Card>,
    pub score: u32,
    pub username: String,
    pub state: PlayerState,
    pub is_playing: bool,
}

impl Player {
    pub fn new(username: String) -> Player {
        Player {
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

    /// Compute the score of the player with their current hand
    pub fn compute_score(&mut self) -> u32 {
        let mut score = 0;

        for card in self.hand.iter() {
            score += match card.value {
                Value::Wild | Value::WildFour => 50,
                Value::Reverse | Value::DrawTwo | Value::Skip => 20,
                Value::Zero => 0,
                value => value as u32,
            }
        }

        score
    }
}

impl std::default::Default for Player {
    fn default() -> Self {
        Player::new("Unknown Player".to_owned())
    }
}
