use crate::lobby::LobbyId;
use bevy_ecs::prelude::Component;
use uuid::Uuid;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct JoinLobby {
    pub lobby_id: Property<LobbyId>,
    pub players: Property<Vec<(String, String)>>,
}

impl JoinLobby {
    pub fn new(lobby_id: LobbyId, players: Vec<(Uuid, String)>) -> Self {
        JoinLobby::new_complete(
            lobby_id,
            players
                .iter()
                .map(|(id, name)| (id.to_string(), name.clone()))
                .collect(),
        )
    }
}
