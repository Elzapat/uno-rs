mod menu;
pub mod utils;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use std::net::TcpStream;
use utils::drag_and_drop::*;

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
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(utils::cursor_state::CursorStatePlugin)
        .add_plugin(utils::drag_and_drop::DragAndDropPlugin)
        .add_plugin(menu::MenuPlugin)
        .add_startup_system(setup)
        .add_system(utils::errors::display_error)
        // .add_system(animate_sprite_system)
        .insert_resource(Settings {
            username: String::from(""),
            enable_animations: true,
        })
        .run();
}

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

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    const SPRITE_WIDTH: f32 = 4860.0 / 10.0;
    const SPRITE_HEIGHT: f32 = 4554.0 / 6.0;

    let texture_handle = asset_server.load("cards_a_03.png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(SPRITE_WIDTH, SPRITE_HEIGHT),
        10,
        6,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(1.0)),
            ..Default::default()
        })
        .insert(Timer::from_seconds(0.5, true))
        .insert(Draggable)
        .insert(SpriteSize {
            width: SPRITE_WIDTH,
            height: SPRITE_HEIGHT,
        });

    let texture_handle = asset_server.load("dirt.png");
    commands
        .spawn_bundle(SpriteBundle {
            texture: texture_handle,
            transform: Transform::from_scale(Vec3::splat(0.5)),
            ..Default::default()
        })
        .insert(Draggable)
        .insert(SpriteSize {
            width: 512.0 * 0.5,
            height: 512.0 * 0.5,
        });
}
