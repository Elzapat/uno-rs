pub mod card;
pub mod deck;
pub mod error;
pub mod packet;
pub mod player;

pub mod prelude {
    pub use crate::{
        card::{self, Card},
        deck::*,
        packet::{self, Packet},
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
