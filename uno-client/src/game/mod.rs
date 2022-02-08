use crate::{
    utils::constants::{CARDS_SPRITESHEET_COLUMNS, CARD_HEIGHT, CARD_WIDTH},
    Draggable, GameState, IncomingPackets, SpriteSize,
};
use bevy::{ecs::schedule::ShouldRun, prelude::*};
use uno::{
    card::{Color, Value},
    packet::{Command, Packet},
};

pub struct GamePlugin;

struct GameAssets {
    background: Handle<Image>,
    cards: Handle<TextureAtlas>,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_assets)
            // .add_system(log_transform)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_in_game)
                    .with_system(execute_packets),
            )
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(setup));
    }
}

fn run_if_in_game(game_state: Res<State<GameState>>) -> ShouldRun {
    if game_state.current() == &GameState::Game {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn log_transform(q: Query<&Transform>) {
    for transform in q.iter() {
        info!("{:?}", transform);
    }
}

pub fn card_to_spritesheet_index(card: uno::card::Card) -> usize {
    let index = match card.value {
        Value::Skip | Value::Reverse | Value::DrawTwo => {
            4 * CARDS_SPRITESHEET_COLUMNS + card.color as usize * 3
        }
        Value::Wild => 53,
        Value::WildFour => 55,
        Value::Back => 56,
        value => card.color as usize * CARDS_SPRITESHEET_COLUMNS + value as usize,
    };

    info!("CARD INDEX: {}", index);
    index
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let cards = asset_server.load("cards_03.png");
    let cards_atlas = TextureAtlas::from_grid(cards, Vec2::new(CARD_WIDTH, CARD_HEIGHT), 10, 6);

    commands.insert_resource(GameAssets {
        background: asset_server.load("game_background.png"),
        cards: texture_atlases.add(cards_atlas),
    });
}

fn execute_packets(
    mut commands: Commands,
    mut incoming_packets: ResMut<IncomingPackets>,
    game_assets: Res<GameAssets>,
) {
    let packets = incoming_packets.0.drain(..).collect::<Vec<Packet>>();

    for mut packet in packets {
        info!("{:?}", packet);
        match packet.command {
            Command::DrawCard => {
                let card = &packet.args.get_range(..)[0..2].into();
                info!("{:?}", card);
                let index = card_to_spritesheet_index(*card);

                commands
                    .spawn_bundle(SpriteSheetBundle {
                        sprite: TextureAtlasSprite {
                            index,
                            ..TextureAtlasSprite::default()
                        },
                        texture_atlas: game_assets.cards.clone_weak(),
                        transform: Transform::from_translation(Vec3::splat(
                            card.color as u32 as f32 * 10.0,
                        )),
                        ..Default::default()
                    })
                    .insert(Draggable)
                    .insert(SpriteSize {
                        width: CARD_WIDTH,
                        height: CARD_HEIGHT,
                    });
            }
            _ => incoming_packets.0.push(packet),
        }
    }
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
