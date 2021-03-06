use bevy_ecs::prelude::Component;
use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct GameExit {
    _p: Property<()>,
}

impl GameExit {
    pub fn new() -> Self {
        GameExit::new_complete(())
    }
}
