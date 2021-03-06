use crate::lobby::LobbyId;
use bevy_ecs::prelude::Component;
use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct Player {
    pub id: Property<u64>,
    pub in_lobby: Property<Option<LobbyId>>,
    pub username: Property<String>,
    pub hand_size: Property<usize>,
    pub score: Property<u32>,
    pub is_playing: Property<bool>,
}

impl Player {
    pub fn new(id: u64, in_lobby: Option<LobbyId>, username: String, hand_size: usize) -> Self {
        Player::new_complete(id, in_lobby, username, hand_size, 0, false)
    }
}
