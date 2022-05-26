use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct GameEnd {
    _p: Property<u8>,
}

impl GameEnd {
    pub fn new() -> Self {
        GameEnd::new_complete(0)
    }
}
