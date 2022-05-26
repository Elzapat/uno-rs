use crate::lobby::LobbyId;
use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct PlayerLeftLobby {
    pub lobby_id: Property<LobbyId>,
    pub player_id: Property<String>,
}

impl PlayerLeftLobby {
    pub fn new(lobby_id: LobbyId, player_id: uuid::Uuid) -> Self {
        PlayerLeftLobby::new_complete(lobby_id, player_id.to_string())
    }
}
