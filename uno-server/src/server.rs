use crate::{lobbies::InLobby, Global};
use bevy_ecs::system::{Commands, Query, ResMut};
use bevy_log::info;
use naia_bevy_server::{Server, ServerAddrs};
use std::collections::HashMap;
use uno::{
    network::{protocol::Lobby as NetworkLobby, Channels, Protocol},
    Lobby, Player,
};

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
        lobbies_room_key: HashMap::new(),
    });
}

pub fn tick(
    mut global: ResMut<Global>,
    mut server: Server<Protocol, Channels>,
    lobbies_query: Query<&Lobby>,
    players_query: Query<(&Player, Option<&InLobby>)>,
) {
    info!("tick");

    for (room_key, user_key, entity) in server.scope_checks() {
        println!("{entity:?}");
        server.user_scope(&user_key).include(&entity);
    }

    server.send_all_updates();
}
