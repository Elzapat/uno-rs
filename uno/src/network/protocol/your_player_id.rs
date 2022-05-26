use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct YourPlayerId {
    player_id: Property<String>,
}

impl YourPlayerId {
    pub fn new(player_id: uuid::Uuid) -> Self {
        YourPlayerId::new_complete(player_id.to_string())
    }
}
