pub mod card;
pub mod deck;
pub mod error;
pub mod lobby;
pub mod network;
pub mod player;

pub mod prelude {
    pub use crate::{
        card::{self, Card},
        deck::*,
        player::*,
    };
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn full_deck_size() {
        assert_eq!(Deck::full().size(), 108);
    }
}
