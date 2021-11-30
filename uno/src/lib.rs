pub mod card;
pub mod deck;
pub mod player;
pub mod packet;
pub mod error;

pub mod prelude {
    pub use crate::{
        card::{ self, Card },
        player::*,
        deck::*,
        packet::{ Packet, self },
        error::{ Error, Result },
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
