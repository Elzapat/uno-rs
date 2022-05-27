use bevy_ecs::prelude::Component;
use uuid::Uuid;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct GameEnd {
    pub winner_id: Property<String>,
}

impl GameEnd {
    pub fn new(winner_id: Uuid) -> Self {
        GameEnd::new_complete(winner_id.to_string())
    }
}
