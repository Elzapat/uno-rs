use crate::lobby::LobbyId;
use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct LobbyInfo {
    lobby_id: Property<LobbyId>,
    players: Property<Vec<String>>,
}

impl LobbyInfo {
    pub fn new(lobby_id: LobbyId, players: Vec<String>) -> Self {
        LobbyInfo::new_complete(lobby_id, players)
    }
}
