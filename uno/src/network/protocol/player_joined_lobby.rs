use crate::{lobby::LobbyId, Player};
use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct PlayerJoinedLobby {
    pub lobby_id: Property<LobbyId>,
    pub player_id: Property<String>,
    pub player_name: Property<String>,
}

impl PlayerJoinedLobby {
    pub fn new(lobby_id: LobbyId, player: Player) -> Self {
        PlayerJoinedLobby::new_complete(lobby_id, player.id.to_string(), player.username)
    }
}
