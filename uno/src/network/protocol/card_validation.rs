use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct CardValidation {
    valid: Property<bool>,
}

impl CardValidation {
    pub fn new(valid: bool) -> Self {
        CardValidation::new_complete(valid)
    }
}
