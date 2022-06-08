pub mod events;
mod lobbies;
mod server;

use bevy_app::{App, ScheduleRunnerPlugin};
use bevy_core::CorePlugin;
use bevy_log::LogPlugin;
use naia_bevy_server::{Plugin as ServerPlugin, RoomKey, ServerConfig, Stage};
use std::collections::HashMap;
use uno::{
    lobby::LobbyId,
    network::{shared_config, Channels, Protocol},
};

pub struct Global {
    pub main_room_key: RoomKey,
    pub lobbies_room_key: HashMap<RoomKey, LobbyId>,
}

fn main() {
    App::default()
        .add_plugin(CorePlugin::default())
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(LogPlugin::default())
        .add_plugin(ServerPlugin::<Protocol, Channels>::new(
            ServerConfig::default(),
            shared_config(),
        ))
        .add_startup_system(server::server_init)
        // Events
        .add_event::<events::CreateLobbyEvent>()
        .add_event::<events::SendMessageEvent>()
        .add_system(events::authorization_event)
        .add_system(events::connection_event)
        .add_system(events::disconnection_event)
        .add_system(events::message_event)
        // Server
        .add_system(server::tick)
        .run();
}
