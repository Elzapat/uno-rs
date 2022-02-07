mod game;
mod menu;
pub mod utils;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use std::net::TcpStream;
use utils::drag_and_drop::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameState {
    Lobbies,
    Game,
    EndLobby,
}

#[derive(Component)]
pub struct SpriteSize {
    width: f32,
    height: f32,
}

pub struct Settings {
    username: String,
    enable_animations: bool,
}

#[derive(Debug)]
pub struct Server {
    socket: TcpStream,
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Uno!".to_owned(),
            width: 1920.0,
            height: 1080.0,
            ..WindowDescriptor::default()
        })
        .add_state(GameState::Lobbies)
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
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
        })
        .run();
}

/*
fn animate_sprite_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index as usize + 1) % texture_atlas.textures.len();
        }
    }
}
*/

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
