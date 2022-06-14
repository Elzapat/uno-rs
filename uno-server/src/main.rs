pub mod events;
pub mod game;
pub mod lobbies;
pub mod server;

use bevy_app::{App, ScheduleRunnerPlugin};
use bevy_core::CorePlugin;
use bevy_ecs::entity::Entity;
use bevy_log::LogPlugin;
use game::Games;
use naia_bevy_server::{Plugin as ServerPlugin, RoomKey, ServerConfig, Stage, UserKey};
use std::collections::HashMap;
use uno::{
    lobby::LobbyId,
    network::{shared_config, Channels, Protocol},
};

pub struct Global {
    pub main_room_key: RoomKey,
    pub user_keys_entities: HashMap<UserKey, Entity>,
    pub lobbies_room_key: HashMap<LobbyId, RoomKey>,
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
        // Server
        .add_event::<server::UsernameChangedEvent>()
        .add_system_to_stage(Stage::Tick, server::tick)
        .add_system(server::username_updated)
        // Events
        .add_system_to_stage(Stage::ReceiveEvents, events::authorization_event)
        .add_system_to_stage(Stage::ReceiveEvents, events::connection_event)
        .add_system_to_stage(Stage::ReceiveEvents, events::disconnection_event)
        .add_system_to_stage(Stage::ReceiveEvents, events::message_event)
        // Lobbies
        .add_event::<lobbies::CreateLobbyEvent>()
        .add_event::<lobbies::JoinLobbyEvent>()
        .add_event::<lobbies::LeaveLobbyEvent>()
        .add_system(lobbies::create_lobby)
        .add_system(lobbies::join_lobby)
        .add_system(lobbies::leave_lobby)
        // Game
        .insert_resource(game::Games(HashMap::new()))
        .add_event::<game::PassTurnEvent>()
        .add_event::<game::StartGameEvent>()
        .add_event::<game::CardPlayedEvent>()
        .add_event::<game::DrawCardEvent>()
        .add_system(game::setup_game)
        .add_system(game::draw_card)
        .add_system(game::pass_turn)
        .run();
}
