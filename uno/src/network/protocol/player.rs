use crate::lobby::LobbyId;
use bevy_ecs::prelude::Component;
use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct Player {
    pub in_lobby: Property<Option<LobbyId>>,
    pub username: Property<String>,
    pub hand_size: Property<usize>,
}

impl Player {
    pub fn new(in_lobby: Option<LobbyId>, username: String, hand_size: usize) -> Self {
        Player::new_complete(in_lobby, username, hand_size)
    }
}
