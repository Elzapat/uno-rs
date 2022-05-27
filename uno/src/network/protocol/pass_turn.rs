use bevy_ecs::prelude::Component;
use uuid::Uuid;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct PassTurn {
    pub playing_id: Property<String>,
}

impl PassTurn {
    pub fn new(playing_id: Uuid) -> Self {
        PassTurn::new_complete(playing_id.to_string())
    }
}
