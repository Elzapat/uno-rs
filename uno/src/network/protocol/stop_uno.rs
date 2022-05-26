use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct StopUno {
    _p: Property<u8>,
}

impl StopUno {
    pub fn new() -> Self {
        StopUno::new_complete(0)
    }
}
