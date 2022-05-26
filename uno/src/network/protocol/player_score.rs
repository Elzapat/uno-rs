use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct PlayerScore {
    pub color: Property<u32>,
}

impl PlayerScore {
    pub fn new(score: u32) -> Self {
        PlayerScore::new_complete(score)
    }
}
