use crate::lobby::{Lobby, LobbyId};
use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct LobbyInfo {
    pub lobby_id: Property<LobbyId>,
    pub players: Property<Vec<(String, String)>>,
}

impl LobbyInfo {
    pub fn new(lobby: &Lobby) -> Self {
        LobbyInfo::new_complete(
            lobby.id,
            lobby
                .players
                .clone()
                .iter()
                .map(|player| (player.id.to_string(), player.username.clone()))
                .collect(),
        )
    }
}
