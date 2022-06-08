use crate::lobby::LobbyId;
use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct JoinLobby {
    pub id: Property<LobbyId>,
}

impl JoinLobby {
    pub fn new(id: LobbyId) -> Self {
        JoinLobby::new_complete(id)
    }
}
