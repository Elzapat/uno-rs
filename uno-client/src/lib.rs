use bevy::prelude::*;
use wasm_bindgen::prelude::*;
use drag_and_drop::*;
use cursor_state::*;

pub mod cursor_state;
pub mod drag_and_drop;

#[derive(Component)]
pub struct SpriteSize {
    width: f32,
    height: f32,
}

fn main() {
    crate::run();
}

#[wasm_bindgen]
pub fn run() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);

    // when building for Web, use WebGL2 rendering
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
    app.add_plugin(DragAndDropPlugin);
    app.add_plugin(CursorStatePlugin);
    app.add_startup_system(setup);
    app.add_system(animate_sprite_system);

    app.run();
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
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // const SPRITE_WIDTH: f32 = 4860.0 / 10.0;
    // const SPRITE_HEIGHT: f32 = 4554.0 / 6.0;

    // let texture_handle = asset_server.load("cards_a_03.png");
    // let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(SPRITE_WIDTH, SPRITE_HEIGHT), 10, 6);
    // let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    // commands
    //     .spawn_bundle(SpriteSheetBundle {
    //         texture_atlas: texture_atlas_handle,
    //         transform: Transform::from_scale(Vec3::splat(1.0)),
    //         ..Default::default()
    //     })
    //     .insert(Timer::from_seconds(0.5, true))
    //     .insert(Hoverable)
    //     .insert(Draggable)
    //     .insert(Size { width: SPRITE_WIDTH, height: SPRITE_HEIGHT });

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("dirt.png"),
            transform: Transform::from_scale(Vec3::splat(0.5)),
            ..Default::default()
        })
        .insert(Draggable)
        // .insert(Hoverable)
        .insert(SpriteSize { width: 512.0, height: 512.0 });
}
