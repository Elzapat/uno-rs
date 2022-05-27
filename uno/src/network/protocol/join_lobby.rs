use crate::{lobby::LobbyId, Player};
use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct JoinLobby {
    pub lobby_id: Property<LobbyId>,
    pub players: Property<Vec<(String, String)>>,
}

impl JoinLobby {
    pub fn new(lobby_id: LobbyId, players: Vec<Player>) -> Self {
        JoinLobby::new_complete(
            lobby_id,
            players
                .iter()
                .map(|player| (player.id.to_string(), player.username.clone()))
                .collect(),
        )
    }
}
