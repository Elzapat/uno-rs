use crate::{
    events::{CreateLobbyEvent, SendMessageEvent},
    Global,
};
use bevy_core::Timer;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{
    component::Component,
    event::{EventReader, EventWriter},
    system::{Commands, ResMut},
};
use bevy_log::info;
use naia_bevy_server::Server;
use uno::{
    lobby::{Lobby, LobbyId, LOBBY_DESPAWN_TIME_S, MAX_LOBBIES},
    network::{protocol::Lobby as NetworkLobby, Channels, Protocol},
};

// #[derive(Deref, DerefMut)]
// pub struct Lobbies(pub Vec<Lobby>);
#[derive(Component, Deref, DerefMut)]
pub struct LobbyComponent(pub Lobby);
#[derive(Component, Deref, DerefMut)]
pub struct LobbyTimer(pub Timer);
#[derive(Component, Deref, DerefMut)]
pub struct InLobby(pub LobbyId);

pub fn create_lobby(
    mut server: Server<Protocol, Channels>,
    mut global: ResMut<Global>,
    mut create_lobby_events: EventReader<CreateLobbyEvent>,
) {
    for _ in create_lobby_events.iter() {
        info!("in create lobby event");
        let new_lobby = Lobby::new();

        global
            .lobbies_room_key
            .insert(server.make_room().key(), new_lobby.id);

        server
            .spawn()
            .enter_room(&global.main_room_key)
            .insert(NetworkLobby::new(new_lobby.id, Vec::new()));
    }
}
