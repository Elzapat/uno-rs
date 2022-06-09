use crate::{server::UserKeyComponent, Global};
use bevy_core::Timer;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_log::info;
use naia_bevy_server::{Server, UserKey};
use uno::{
    lobby::{Lobby, LobbyId, LOBBY_DESPAWN_TIME_S, MAX_LOBBIES},
    network::{protocol::Lobby as NetworkLobby, Channels, Protocol},
};

// Events
// Lobby sent when the user wants to create a lobby
pub struct CreateLobbyEvent;
pub struct JoinLobbyEvent {
    pub lobby_id: LobbyId,
    pub user_key: UserKey,
}
pub struct LeaveLobbyEvent;

// Components
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
        let new_lobby = Lobby::new();

        global
            .lobbies_room_key
            .insert(new_lobby.id, server.make_room().key());

        server
            .spawn()
            .enter_room(&global.main_room_key)
            .insert(NetworkLobby::new(new_lobby.id, Vec::new()));
    }
}

pub fn join_lobby(
    mut commands: Commands,
    mut server: Server<Protocol, Channels>,
    mut join_lobby_events: EventReader<JoinLobbyEvent>,
    global: Res<Global>,
    players_query: Query<(Entity, &UserKeyComponent), Without<InLobby>>,
) {
    for JoinLobbyEvent { lobby_id, user_key } in join_lobby_events.iter() {
        dbg!("join lobby event!!");
        if let Some(room_key) = global.lobbies_room_key.get(lobby_id) {
            server
                .user_mut(user_key)
                .leave_room(&global.main_room_key)
                .enter_room(room_key);

            for (entity, &player_user_key) in players_query.iter() {
                if *player_user_key == *user_key {
                    commands.entity(entity).insert(InLobby(*lobby_id));
                    break;
                }
            }

            dbg!("HERE");
            server.send_message(
                user_key,
                Channels::Uno,
                &uno::network::protocol::JoinLobby::new(*lobby_id),
            );
        } else {
            dbg!("couldn't find room key");
        }
    }
}
