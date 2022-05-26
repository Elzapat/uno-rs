use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct Username {
    pub username: Property<String>,
}

impl Username {
    pub fn new(username: String) -> Self {
        Username::new_complete(username)
    }
}
