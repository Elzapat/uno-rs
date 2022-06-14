use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use uno::{lobby::LobbyId, Card};

#[derive(Deref, DerefMut)]
pub struct StartGameEvent(pub LobbyId);
#[derive(Deref, DerefMut)]
pub struct CardPlayedEvent(pub Card);

pub fn setup_game(mut start_game_event: EventReader<StartGameEvent>) {
    for StartGameEvent(lobby_id) in start_game_event.iter() {}
}
