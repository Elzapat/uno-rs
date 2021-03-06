use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct StartGame {
    _p: Property<()>,
}

impl StartGame {
    pub fn new() -> Self {
        StartGame::new_complete(())
    }
}
