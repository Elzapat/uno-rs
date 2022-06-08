use bevy_ecs::prelude::Component;
use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct Player {
    username: Property<String>,
    hand_size: Property<u32>,
}

impl Player {
    pub fn new(username: String, hand_size: u32) -> Self {
        Player::new_complete(username, hand_size)
    }
}
