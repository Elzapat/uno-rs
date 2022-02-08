#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Color {
    Red,
    Yellow,
    Green,
    Blue,
    Black,
}

impl From<u8> for Color {
    fn from(color: u8) -> Color {
        match color {
            0 => Color::Red,
            1 => Color::Yellow,
            2 => Color::Green,
            3 => Color::Blue,
            _ => Color::Black,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Value {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Zero,
    Skip,
    Reverse,
    DrawTwo,
    Wild,
    WildFour,
    Back,
}

impl From<u8> for Value {
    fn from(value: u8) -> Value {
        match value {
            0 => Value::Zero,
            1 => Value::One,
            2 => Value::Two,
            3 => Value::Three,
            4 => Value::Four,
            5 => Value::Five,
            6 => Value::Six,
            7 => Value::Seven,
            8 => Value::Eight,
            9 => Value::Nine,
            10 => Value::Skip,
            11 => Value::Reverse,
            12 => Value::DrawTwo,
            13 => Value::Wild,
            14 => Value::WildFour,
            _ => Value::Back,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
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

impl Into<[u8; 2]> for Card {
    fn into(self) -> [u8; 2] {
        [self.color as u8, self.value as u8]
    }
}
