use crate::{lobbies::InLobby, Global};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_log::info;
use naia_bevy_server::{Server, ServerAddrs, UserKey};
use std::collections::HashMap;
use uno::{
    network::{
        protocol::{Lobby as NetworkLobby, Player as NetworkPlayer},
        Channels, Protocol,
    },
    Lobby, Player,
};

pub struct UsernameChangedEvent {
    pub user_key: UserKey,
    pub username: String,
}

#[derive(Component, Deref, DerefMut, Copy, Clone)]
pub struct UserKeyComponent(pub UserKey);

pub fn server_init(mut commands: Commands, mut server: Server<Protocol, Channels>) {
    info!("init server");

    let server_addresses = ServerAddrs::new(
        "127.0.0.1:3478".parse().unwrap(),
        "127.0.0.1:3478".parse().unwrap(),
        "http://127.0.0.1:3478",
    );

    server.listen(&server_addresses);

    commands.insert_resource(Global {
        main_room_key: server.make_room().key(),
        user_keys_entities: HashMap::new(),
        lobbies_room_key: HashMap::new(),
    });
}

pub fn tick(
    global: Res<Global>,
    mut server: Server<Protocol, Channels>,
    players_query: Query<(&UserKeyComponent, &Player)>,
    mut network_players_query: Query<(Entity, &mut NetworkPlayer)>,
) {
    // info!("tick");

    // Sync player number of cards and username with clients
    for (user_key, player) in players_query.iter() {
        for (entity, mut network_player) in network_players_query.iter_mut() {
            if entity == global.user_keys_entities[user_key] {
                *network_player.username = player.username.clone();
                *network_player.hand_size = player.hand.len();
            }
        }
    }

    for (_room_key, user_key, entity) in server.scope_checks() {
        server.user_scope(&user_key).include(&entity);
    }

    server.send_all_updates();
}

pub fn username_updated(
    mut username_changed_events: EventReader<UsernameChangedEvent>,
    mut players_query: Query<(&UserKeyComponent, &mut Player)>,
) {
    for UsernameChangedEvent { user_key, username } in username_changed_events.iter() {
        for (player_user_key, mut player) in players_query.iter_mut() {
            if **player_user_key == *user_key {
                player.username = username.clone();
                break;
            }
        }
    }
}
