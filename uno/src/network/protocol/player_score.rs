use bevy_ecs::prelude::Component;
use uuid::Uuid;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct PlayerScore {
    pub score: Property<u32>,
    pub player_id: Property<String>,
}

impl PlayerScore {
    pub fn new(score: u32, player_id: Uuid) -> Self {
        PlayerScore::new_complete(score, player_id.to_string())
    }
}
