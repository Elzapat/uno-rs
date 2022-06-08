use crate::Global;
use bevy_ecs::system::{Commands, ResMut};
use naia_bevy_server::{Server, ServerAddrs};
use std::collections::HashMap;
use uno::network::{Channels, Protocol};

pub fn server_init(mut commands: Commands, mut server: Server<Protocol, Channels>) {
    let server_addresses = ServerAddrs::new(
        "0.0.0.0:3478".parse().unwrap(),
        "0.0.0.0:3478".parse().unwrap(),
        "http://127.0.0.1:3478",
    );

    server.listen(&server_addresses);

    commands.insert_resource(Global {
        main_room_key: server.make_room().key(),
        lobbies_room_key: HashMap::new(),
    });
}

pub fn tick(mut global: ResMut<Global>, mut server: Server<Protocol, Channels>) {
    for (room_key, user_key, entity) in server.scope_checks() {
        server.user_scope(&user_key).include(&entity);
    }

    server.send_all_updates();
}
