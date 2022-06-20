use crate::lobby::LobbyId;
use bevy_ecs::prelude::Component;
use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct Lobby {
    pub id: Property<LobbyId>,
    pub number_of_players: Property<usize>,
}

impl Lobby {
    pub fn new(id: LobbyId, number_of_players: usize) -> Self {
        Lobby::new_complete(id, number_of_players)
    }
}
