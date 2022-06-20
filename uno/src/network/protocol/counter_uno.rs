use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct CounterUno {
    _p: Property<()>,
}

impl CounterUno {
    pub fn new() -> Self {
        CounterUno::new_complete(())
    }
}
