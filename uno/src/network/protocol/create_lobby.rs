use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct CreateLobby {
    _p: Property<u8>,
}

impl CreateLobby {
    pub fn new() -> Self {
        CreateLobby::new_complete(0)
    }
}
