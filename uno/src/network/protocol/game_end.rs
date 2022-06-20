use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct GameEnd {
    pub _p: Property<()>,
}

impl GameEnd {
    pub fn new() -> Self {
        GameEnd::new_complete(())
    }
}
