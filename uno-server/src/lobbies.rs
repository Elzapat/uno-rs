use crate::events::{CreateLobbyEvent, SendMessageEvent};
use bevy_core::Timer;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{
    component::Component,
    event::{EventReader, EventWriter},
    system::Commands,
};
use naia_bevy_server::Server;
use uno::{
    lobby::{Lobby, LobbyId, LOBBY_DESPAWN_TIME_S, MAX_LOBBIES},
    network::{protocol, Channels, Protocol},
};

// #[derive(Deref, DerefMut)]
// pub struct Lobbies(pub Vec<Lobby>);
#[derive(Component, Deref, DerefMut)]
pub struct LobbyComponent(pub Lobby);
#[derive(Component, Deref, DerefMut)]
pub struct LobbyTimer(pub Timer);
#[derive(Deref, DerefMut)]
pub struct InLobby(pub LobbyId);

pub fn create_lobby(
    mut commands: Commands,
    mut create_lobby_events: EventReader<CreateLobbyEvent>,
) {
    for _ in create_lobby_events.iter() {
        commands
            .spawn()
            .insert(LobbyComponent(Lobby::new()))
            .insert(LobbyTimer(Timer::from_seconds(LOBBY_DESPAWN_TIME_S, false)));
    }
}
