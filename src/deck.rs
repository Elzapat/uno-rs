use crate::prelude::*;
use std::collections::VecDeque;
use rand::Rng;

pub struct Deck {
    cards: VecDeque<Card>,
}

impl Deck {
    /// Create an empty deck
    pub fn empty() -> Deck {
        Deck {
            cards: VecDeque::new(),
        }
    }

    /// Fill the deck with the cards of a base Uno game:
    ///   - 1 Zero of each color, and 2 of the other numbers
    ///   - 2 draw two, 2 reverse and 2 skip of each color
    ///   - 4 wild cards and 4 wild draw 4 cards
    pub fn full() -> Deck {
        let mut deck = Deck::empty();

        for color in (Color::Red as i32)..=(Color::Blue as i32) {
            for value in (Value::Zero as i32)..=(Value::DrawTwo as i32) {
                for i in 0..2 {
                    if i == 1 && value == Value::Zero as i32 { continue; }
                    deck.insert((color, value).into());
                }
            }
        }

        for value in (Value::Wild as i32)..=(Value::WildFour as i32) {
            for _ in 0..4 {
                let color = Color::Black as i32;
                deck.insert((color, value).into());
            }
        }

        deck
    }

    /// Get the number of cards remaining in the deck
    pub fn number_of_cards(&self) -> usize {
        self.cards.len()
    }

    /// Shuffle the deck randomly
    pub fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        let n = self.number_of_cards();

        for i in 0..n {
            let r = i + (rng.gen::<usize>() % (n - i));
            self.cards.swap(i, r);
        }
    }

    /// Insert a card at the bottom of the deck
    pub fn insert(&mut self, card: Card) {
        self.cards.push_back(card);
    }

    /// Draw a card at the top of the deck
    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop_front()
    }
}
