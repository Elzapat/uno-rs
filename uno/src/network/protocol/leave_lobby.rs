use crate::lobby::LobbyId;
use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct LeaveLobby {
    lobby_id: Property<LobbyId>,
}

impl LeaveLobby {
    pub fn new(lobby_id: LobbyId) -> Self {
        LeaveLobby::new_complete(lobby_id)
    }
}
