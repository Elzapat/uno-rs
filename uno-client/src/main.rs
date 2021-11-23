use bevy::prelude::*;

pub mod cursor_state;
pub mod drag_and_drop;

use drag_and_drop::*;
use cursor_state::*;

pub struct Size {
    width: f32,
    height: f32,
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(CursorStatePlugin)
        .add_plugin(DragAndDropPlugin)
        .add_startup_system(setup.system())
        .add_system(animate_sprite_system.system())
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
            sprite.index = ((sprite.index as usize + 1) % texture_atlas.textures.len()) as u32;
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    const SPRITE_WIDTH: f32 = 4860.0 / 10.0;
    const SPRITE_HEIGHT: f32 = 4554.0 / 6.0;

    let texture_handle = asset_server.load("cards_a_03.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(SPRITE_WIDTH, SPRITE_HEIGHT), 10, 6);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(1.0)),
            ..Default::default()
        })
        .insert(Timer::from_seconds(0.5, true))
        .insert(Draggable)
        .insert(Size { width: SPRITE_WIDTH, height: SPRITE_HEIGHT });

    let texture_handle = asset_server.load("dirt.png");
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            transform: Transform::from_scale(Vec3::splat(0.5)),
            ..Default::default()
        })
        .insert(Draggable)
        .insert(Size { width: 512.0 * 0.5, height: 512.0 * 0.5 });
}
