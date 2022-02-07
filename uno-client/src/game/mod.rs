use crate::{Draggable, GameState, SpriteSize};
use bevy::prelude::*;

pub struct GamePlugin;

const CARD_WIDTH: f32 = 2430.0 / 10.0;
const CARD_HEIGHT: f32 = 2277.0 / 6.0;

struct GameAssets {
    background: Handle<Image>,
    cards: Handle<TextureAtlas>,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_assets)
            .add_system(log_transform)
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(setup));
    }
}

fn log_transform(q: Query<&Transform>) {
    for transform in q.iter() {
        info!("{:?}", transform);
    }
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let cards = asset_server.load("cards_03.png");
    let cards_atlas = TextureAtlas::from_grid(cards, Vec2::new(CARD_WIDTH, CARD_HEIGHT), 10, 6);
    // const SPRITE_WIDTH: f32 = 4860.0 / 10.0;
    // const SPRITE_HEIGHT: f32 = 4554.0 / 6.0;

    commands.insert_resource(GameAssets {
        background: asset_server.load("game_background.png"),
        cards: texture_atlases.add(cards_atlas),
    });
}

fn setup(mut commands: Commands, game_assets: Res<GameAssets>) {
    // let mut sprite = SpriteBundle {
    //     texture: game_assets.background.clone(),
    //     ..SpriteBundle::default()
    // };
    // sprite.transform.translation.z -= 10.0;

    commands.spawn_bundle(SpriteBundle {
        texture: game_assets.background.clone(),
        ..SpriteBundle::default()
    });

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_assets.cards.clone_weak(),
            transform: Transform::from_translation(Vec3::splat(1.0)),
            ..Default::default()
        })
        .insert(Draggable)
        .insert(SpriteSize {
            width: CARD_WIDTH,
            height: CARD_HEIGHT,
        });
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_assets.cards.clone_weak(),
            transform: Transform::from_translation(Vec3::splat(100.0)),
            ..Default::default()
        })
        .insert(Draggable)
        .insert(SpriteSize {
            width: CARD_WIDTH,
            height: CARD_HEIGHT,
        });
}
