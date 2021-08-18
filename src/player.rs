use crate::prelude::*;

/// Structure to define a Uno player
#[derive(Copy, Clone, Debug, PartialEq)]
struct Player {
    hand: Vec<Card>,
    is_playing: bool,
    score: u32,
}
