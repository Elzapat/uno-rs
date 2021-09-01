use crate::prelude::*;

/// Structure to define a Uno player
#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub hand: Vec<Card>,
    pub is_playing: bool,
    pub score: u32,
    pub username: String,
}

impl Player {
    pub fn new(username: String) -> Player {
        Player {
            hand: Vec::new(),
            is_playing: false,
            score: 0,
            username,
        }
    }
}
