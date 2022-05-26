use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct StopCounterUno {
    _p: Property<u8>,
}

impl StopCounterUno {
    pub fn new() -> Self {
        StopCounterUno::new_complete(0)
    }
}
