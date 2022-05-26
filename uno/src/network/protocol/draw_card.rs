use crate::card::Card;
use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct DrawCard {
    color: Property<u8>,
    value: Property<u8>,
}

impl DrawCard {
    pub fn new(card: Card) -> Self {
        DrawCard::new_complete(card.color as u8, card.value as u8)
    }
}
