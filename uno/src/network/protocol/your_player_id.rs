use bevy_ecs::prelude::Component;
use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct YourPlayerId {
    pub id: Property<u64>,
}

impl YourPlayerId {
    pub fn new(id: u64) -> Self {
        YourPlayerId::new_complete(id)
    }
}
