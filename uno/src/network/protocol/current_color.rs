use crate::card::Color;
use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct CurrentColor {
    color: Property<u8>,
}

impl CurrentColor {
    pub fn new(color: Color) -> Self {
        CurrentColor::new_complete(color as u8)
    }
}
