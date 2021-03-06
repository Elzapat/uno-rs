use crate::card::Card;
use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct PlayCard {
    pub color: Property<u8>,
    pub value: Property<u8>,
}

impl PlayCard {
    pub fn new(card: Card) -> Self {
        PlayCard::new_complete(card.color as u8, card.value as u8)
    }
}
