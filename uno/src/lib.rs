pub mod card;
pub mod deck;
pub mod error;
pub mod lobby;
pub mod network;
pub mod player;
pub mod texts;

pub use card::Card;
pub use deck::Deck;
pub use lobby::Lobby;
pub use player::Player;

#[cfg(test)]
mod tests {
    #[test]
    fn full_deck_size() {
        assert_eq!(crate::deck::Deck::full().size(), 108);
    }
}
