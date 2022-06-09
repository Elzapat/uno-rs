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
    mut global: ResMut<Global>,
    mut server: Server<Protocol, Channels>,
    lobbies_query: Query<&Lobby>,
    players_query: Query<Option<&InLobby>, With<Player>>,
) {
    // info!("tick");

    for (room_key, user_key, entity) in server.scope_checks() {
        if server.entity(&entity).has_component::<NetworkLobby>() {
            server.user_scope(&user_key).include(&entity);
        } else if server.entity(&entity).has_component::<NetworkPlayer>() {
            for (lobby_id, other_room_key) in global.lobbies_room_key.iter() {
                if room_key == *other_room_key {
                    for other_lobby_id in players_query.iter().flatten() {
                        if **other_lobby_id == *lobby_id {
                            server.user_scope(&user_key).include(&entity);
                        }
                    }
                    break;
                }
            }
        }
        // println!("{entity:?}");
        // server.user_scope(&user_key).include(&entity);
    }

    server.send_all_updates();
}
