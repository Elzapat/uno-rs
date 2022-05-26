use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct Error {
    error: Property<String>,
}

impl Error {
    pub fn new(error: String) -> Self {
        Error::new_complete(error)
    }
}
