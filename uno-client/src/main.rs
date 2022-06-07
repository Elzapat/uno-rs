#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
mod game;
mod menu;
pub mod utils;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use naia_bevy_client::{Client, ClientConfig, Plugin as ClientPlugin};
use serde::{Deserialize, Serialize};
use uno::network::{shared_config, Channels, Protocol};
use utils::drag_and_drop::*;

#[derive(Serialize, Deserialize, Debug)]
struct NetworkPacket(Vec<u8>);

// States
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameState {
    Lobbies,
    Game,
    EndLobby,
}

// Components
#[derive(Component)]
pub struct SpriteSize {
    width: f32,
    height: f32,
}

// Resources
pub struct Settings {
    username: String,
    enable_animations: bool,
}

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "Uno!".to_owned(),
        // width: 1920.0,
        // height: 1080.0,
        // vsync: false,
        ..WindowDescriptor::default()
    })
    .add_state(GameState::Lobbies)
    .add_plugins(DefaultPlugins)
    .add_plugin(EguiPlugin)
    .add_plugin(ClientPlugin::<Protocol, Channels>::new(
        ClientConfig::default(),
        shared_config(),
    ))
    .add_plugin(utils::cursor_state::CursorStatePlugin)
    .add_plugin(utils::drag_and_drop::DragAndDropPlugin)
    .add_plugin(menu::MenuPlugin)
    .add_plugin(game::GamePlugin)
    .add_startup_system(setup)
    .add_system(utils::errors::display_error)
    // .add_system(animate_sprite_system)
    .insert_resource(Settings {
        username: String::from(""),
        enable_animations: true,
    });

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_web_resizer::Plugin);

    app.run();
}

fn setup(mut commands: Commands, mut client: Client<Protocol, Channels>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    client.auth(uno::network::protocol::Uno::new());
    client.connect("http://127.0.0.1:3478");
}
