use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct Uno {
    _p: Property<u8>,
}

impl Uno {
    pub fn new() -> Self {
        Uno::new_complete(0)
    }
}
