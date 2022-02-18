use crate::packet::Args;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    Yellow = 0,
    Red = 1,
    Blue = 2,
    Green = 3,
    Black = 4,
}

impl From<u8> for Color {
    fn from(color: u8) -> Color {
        match color {
            0 => Color::Yellow,
            1 => Color::Red,
            2 => Color::Blue,
            3 => Color::Green,
            _ => Color::Black,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Value {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Zero = 10,
    DrawTwo = 11,
    Skip = 12,
    Reverse = 13,
    Wild = 14,
    WildFour = 15,
    Back = 16,
}

impl From<u8> for Value {
    fn from(value: u8) -> Value {
        match value {
            1 => Value::One,
            2 => Value::Two,
            3 => Value::Three,
            4 => Value::Four,
            5 => Value::Five,
            6 => Value::Six,
            7 => Value::Seven,
            8 => Value::Eight,
            9 => Value::Nine,
            10 => Value::Zero,
            11 => Value::DrawTwo,
            12 => Value::Skip,
            13 => Value::Reverse,
            14 => Value::Wild,
            15 => Value::WildFour,
            _ => Value::Back,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Card {
    pub color: Color,
    pub value: Value,
}

impl From<(u8, u8)> for Card {
    fn from(card: (u8, u8)) -> Card {
        Card {
            color: card.0.into(),
            value: card.1.into(),
        }
    }
}

impl From<[u8; 2]> for Card {
    fn from(arr: [u8; 2]) -> Card {
        Card::from((arr[0], arr[1]))
    }
}

impl From<&[u8]> for Card {
    fn from(arr: &[u8]) -> Card {
        Card::from((arr[0], arr[1]))
    }
}

#[allow(clippy::from_over_into)]
impl Into<[u8; 2]> for Card {
    fn into(self) -> [u8; 2] {
        [self.color as u8, self.value as u8]
    }
}

impl From<Args> for Card {
    fn from(args: Args) -> Self {
        Card::from((*args.get(0).unwrap(), *args.get(1).unwrap()))
    }
}

impl From<Card> for Args {
    fn from(card: Card) -> Self {
        Args(vec![card.color as u8, card.value as u8])
    }
}
