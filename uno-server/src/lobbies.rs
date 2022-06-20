use crate::{server::UserKeyComponent, Global};
use bevy_core::Timer;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use naia_bevy_server::{Server, UserKey};
use uno::{
    lobby::{Lobby, LobbyId},
    network::{
        protocol::{JoinLobby, LeaveLobby, Lobby as NetworkLobby, ThisPlayer},
        Channels, Protocol,
    },
};

// Events
// Lobby sent when the user wants to create a lobby
pub struct CreateLobbyEvent;
pub struct JoinLobbyEvent {
    pub lobby_id: LobbyId,
    pub user_key: UserKey,
}
pub struct LeaveLobbyEvent {
    pub lobby_id: LobbyId,
    pub user_key: UserKey,
}

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
            .insert(NetworkLobby::new(new_lobby.id, 0));
    }
}

pub fn join_lobby(
    mut commands: Commands,
    mut server: Server<Protocol, Channels>,
    mut join_lobby_events: EventReader<JoinLobbyEvent>,
    mut lobbies_query: Query<&mut NetworkLobby>,
    players_query: Query<(Entity, &UserKeyComponent), Without<InLobby>>,
    this_player_query: Query<(Entity, &ThisPlayer)>,
    global: Res<Global>,
) {
    for JoinLobbyEvent { lobby_id, user_key } in join_lobby_events.iter() {
        server
            .user_mut(user_key)
            .leave_room(&global.main_room_key)
            .enter_room(&global.lobbies_room_key[lobby_id]);

        server
            .room_mut(&global.main_room_key)
            .remove_entity(&global.user_keys_entities[user_key]);

        server
            .room_mut(&global.lobbies_room_key[lobby_id])
            .add_entity(&global.user_keys_entities[user_key]);

        for (
            entity,
            ThisPlayer {
                entity: this_player_entity,
            },
        ) in this_player_query.iter()
        {
            if Entity::from_bits(**this_player_entity) == global.user_keys_entities[user_key] {
                server
                    .room_mut(&global.main_room_key)
                    .remove_entity(&entity);

                server
                    .room_mut(&global.lobbies_room_key[lobby_id])
                    .add_entity(&entity);

                break;
            }
        }

        for (entity, &player_user_key) in players_query.iter() {
            if *player_user_key == *user_key {
                commands.entity(entity).insert(InLobby(*lobby_id));
                break;
            }
        }

        for mut network_lobby in lobbies_query.iter_mut() {
            if *network_lobby.id == *lobby_id {
                *network_lobby.number_of_players += 1;
            }
        }

        server.send_message(user_key, Channels::Uno, &JoinLobby::new(*lobby_id));
    }
}

pub fn leave_lobby(
    mut commands: Commands,
    mut server: Server<Protocol, Channels>,
    mut leave_lobby_events: EventReader<LeaveLobbyEvent>,
    mut lobbies_query: Query<&mut NetworkLobby>,
    players_query: Query<(Entity, &UserKeyComponent), Without<InLobby>>,
    this_player_query: Query<(Entity, &ThisPlayer)>,
    global: Res<Global>,
) {
    for LeaveLobbyEvent { user_key, lobby_id } in leave_lobby_events.iter() {
        server
            .user_mut(user_key)
            .leave_room(&global.lobbies_room_key[lobby_id])
            .enter_room(&global.main_room_key);

        server
            .room_mut(&global.lobbies_room_key[lobby_id])
            .remove_entity(&global.user_keys_entities[user_key]);

        server
            .room_mut(&global.main_room_key)
            .add_entity(&global.user_keys_entities[user_key]);

        for (
            entity,
            ThisPlayer {
                entity: this_player_entity,
            },
        ) in this_player_query.iter()
        {
            if Entity::from_bits(**this_player_entity) == global.user_keys_entities[user_key] {
                server
                    .room_mut(&global.lobbies_room_key[lobby_id])
                    .remove_entity(&entity);

                server.room_mut(&global.main_room_key).add_entity(&entity);

                break;
            }
        }

        for (entity, &player_user_key) in players_query.iter() {
            if *player_user_key == *user_key {
                commands.entity(entity).remove::<InLobby>();
                break;
            }
        }

        for mut network_lobby in lobbies_query.iter_mut() {
            if *network_lobby.id == *lobby_id && *network_lobby.number_of_players > 0 {
                *network_lobby.number_of_players -= 1;
            }
        }

        server.send_message(user_key, Channels::Uno, &LeaveLobby::new(*lobby_id));
    }
}
