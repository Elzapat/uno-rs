use crate::lobby::LobbyId;
use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct PlayerJoinedLobby {
    lobby_id: Property<LobbyId>,
    player_id: Property<String>,
}

impl PlayerJoinedLobby {
    pub fn new(lobby_id: LobbyId, player_id: uuid::Uuid) -> Self {
        PlayerJoinedLobby::new_complete(lobby_id, player_id.to_string())
    }
}
