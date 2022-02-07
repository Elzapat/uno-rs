use crate::prelude::*;
use card::{Color, Value};
use rand::Rng;
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq)]
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
    /// * 1 Zero of each color, and 2 of the other numbers
    /// * 2 draw two, 2 reverse and 2 skip of each color
    /// * 4 wild cards and 4 wild draw 4 cards
    pub fn full() -> Deck {
        let mut deck = Deck::empty();

        for color in (Color::Red as u8)..=(Color::Blue as u8) {
            for value in (Value::Zero as u8)..=(Value::DrawTwo as u8) {
                for i in 0..2 {
                    if i == 1 && value == Value::Zero as u8 {
                        continue;
                    }

                    deck.insert((color, value).into());
                }
            }
        }

        for value in (Value::Wild as u8)..=(Value::WildFour as u8) {
            for _ in 0..4 {
                let color = Color::Black as u8;
                deck.insert((color, value).into());
            }
        }

        deck
    }

    /// Get the number of cards remaining in the deck
    pub fn size(&self) -> usize {
        self.cards.len()
    }

    /// Shuffle the deck randomly
    pub fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        let n = self.size();

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
