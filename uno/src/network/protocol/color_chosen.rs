use crate::card::Color;
use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct ColorChosen {
    color: Property<u8>,
}

impl ColorChosen {
    pub fn new(color: Color) -> Self {
        ColorChosen::new_complete(color as u8)
    }
}
