use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct HandSize {
    size: Property<u8>,
}

impl HandSize {
    pub fn new(size: u8) -> Self {
        HandSize::new_complete(size)
    }
}
