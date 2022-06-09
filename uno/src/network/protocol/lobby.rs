use crate::lobby::LobbyId;
use bevy_ecs::prelude::Component;
use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct Lobby {
    pub id: Property<LobbyId>,
    pub players: Property<Vec<String>>,
}

impl Lobby {
    pub fn new(id: LobbyId, players: Vec<String>) -> Self {
        Lobby::new_complete(id, players)
    }
}
