use super::{run_if_in_game, GameAssets};
use crate::{
    utils::constants::{
        CARD_ANIMATION_TIME_S, CARD_HEIGHT, CARD_SCALE, CARD_WIDTH, DECK_POS, DISCARD_POS,
        Z_INCREASE,
    },
    Draggable, Dragged, Dropped, GameState, SpriteSize,
};
use bevy::{prelude::*, window::WindowResized};
use std::time::Duration;
use uno::card::{Card, Color, Value};

// Ressources
pub struct CurrentCardZ(pub f32);
pub struct Hand {
    size: usize,
}

// Components
#[derive(Component)]
struct CardComponent(Card);
#[derive(Component)]
struct Discard;
#[derive(Component)]
struct HandItem {
    index: usize,
    position: Vec3,
}
#[derive(Component)]
struct CardAnimation {
    time: Duration,
}

// Events
pub struct SpawnCardEvent(pub Card);
struct ReorganizeHandEvent;

pub struct CardsPlugin;

impl Plugin for CardsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentCardZ(Z_INCREASE))
            .insert_resource(Hand { size: 0 })
            .add_event::<SpawnCardEvent>()
            .add_event::<ReorganizeHandEvent>()
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(setup_cards))
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_in_game)
                    .with_system(spawn_card)
                    .with_system(reorganize_hand)
                    .with_system(card_dropped)
                    .with_system(animate_card)
                    .with_system(window_resized)
                    .with_system(remove_animation_on_drag),
            );
    }
}

pub fn card_to_spritesheet_index(card: &uno::card::Card) -> usize {
    match card.value {
        Value::Back => 0,
        Value::Wild => 1,
        Value::WildFour => 6,
        value => 11 + 13 * card.color as usize + value as usize,
    }
}

fn setup_cards(mut commands: Commands, game_assets: Res<GameAssets>) {
    let mut transform = Transform::from_xyz(DECK_POS.0, DECK_POS.1, 0.0);
    transform.scale = Vec3::new(CARD_SCALE, CARD_SCALE, 1.0);

    let card_back = Card {
        value: Value::Back,
        color: Color::Black,
    };

    // Back of card representing the deck
    commands.spawn_bundle(SpriteSheetBundle {
        sprite: TextureAtlasSprite {
            index: card_to_spritesheet_index(&card_back),
            ..TextureAtlasSprite::default()
        },
        texture_atlas: game_assets.cards.clone_weak(),
        transform,
        ..SpriteSheetBundle::default()
    });

    // Discard for the cards
    transform.translation = Vec3::new(DISCARD_POS.0, DISCARD_POS.1, 0.0);
    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: card_to_spritesheet_index(&card_back),
                ..TextureAtlasSprite::default()
            },
            texture_atlas: game_assets.cards.clone_weak(),
            transform,
            ..SpriteSheetBundle::default()
        })
        .insert(Discard);
}

fn spawn_card(
    mut commands: Commands,
    mut spawn_card_event: EventReader<SpawnCardEvent>,
    mut reorganize_hand_event: EventWriter<ReorganizeHandEvent>,
    mut current_card_z: ResMut<CurrentCardZ>,
    mut hand: ResMut<Hand>,
    game_assets: Res<GameAssets>,
) {
    for SpawnCardEvent(card) in spawn_card_event.iter() {
        let index = card_to_spritesheet_index(card);

        let mut card_transform = Transform::from_xyz(DECK_POS.0, DECK_POS.1, current_card_z.0);
        card_transform.scale = Vec3::new(CARD_SCALE, CARD_SCALE, 1.0);

        commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index,
                    ..TextureAtlasSprite::default()
                },
                texture_atlas: game_assets.cards.clone_weak(),
                transform: card_transform,
                ..SpriteSheetBundle::default()
            })
            .insert(Draggable)
            .insert(SpriteSize {
                width: CARD_WIDTH,
                height: CARD_HEIGHT,
            })
            .insert(CardComponent(*card))
            .insert(HandItem {
                index: hand.size,
                position: card_transform.translation,
            });

        hand.size += 1;
        current_card_z.0 += Z_INCREASE;
        reorganize_hand_event.send(ReorganizeHandEvent);
    }
}

fn reorganize_hand(
    mut commands: Commands,
    mut reorganize_hand_event: EventReader<ReorganizeHandEvent>,
    windows: Res<Windows>,
    hand: Res<Hand>,
    mut query: Query<(Entity, &mut HandItem)>,
) {
    if hand.size == 0 {
        return;
    }

    #[allow(clippy::never_loop)]
    for _ in reorganize_hand_event.iter() {
        let window = windows.get_primary().unwrap();
        let card_y = -window.height() / 2.0 + (CARD_HEIGHT * CARD_SCALE) * 0.25;

        const X_PADDING: f32 = 0.1;
        let width = window.width() * (1.0 - 2.0 * X_PADDING);
        let part = width / hand.size as f32;

        info!("hand size: {}", hand.size);
        for (entity, mut item) in query.iter_mut() {
            info!("{}", item.index);
            item.position.y = card_y;
            item.position.x = part * item.index as f32 + part / 2.0 - width / 2.0;

            commands.entity(entity).insert(Dropped);
        }

        // We don't care how many times the event has been queued, we reorganize once
        break;
    }
}

fn card_dropped(
    mut commands: Commands,
    query: Query<Entity, (With<Dropped>, With<CardComponent>)>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .remove::<Dropped>()
            .insert(CardAnimation {
                time: Duration::default(),
            });
    }
}

fn animate_card(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &HandItem, &mut Transform, &mut CardAnimation), Without<Dragged>>,
) {
    for (entity, hand_item, mut transform, mut card_animation) in query.iter_mut() {
        card_animation.time += time.delta();

        if card_animation.time.as_secs_f32() >= CARD_ANIMATION_TIME_S {
            transform.translation = hand_item.position;
            commands.entity(entity).remove::<CardAnimation>();
            return;
        }

        transform.translation = transform.translation.lerp(
            hand_item.position,
            ease_out_sine(card_animation.time.as_secs_f32() / CARD_ANIMATION_TIME_S),
        );
    }
}

fn window_resized(
    mut window_resized: EventReader<WindowResized>,
    mut reorganize_hand_event: EventWriter<ReorganizeHandEvent>,
) {
    for _ in window_resized.iter() {
        reorganize_hand_event.send(ReorganizeHandEvent);
    }
}

fn remove_animation_on_drag(
    mut commands: Commands,
    query: Query<Entity, (With<Dragged>, With<CardAnimation>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<CardAnimation>();
    }
}

/*
fn move_towards(pos: &mut Vec3, dest: &Vec3, speed: f32) -> bool {
    let diff = *dest - *pos;

    if diff.length() > speed {
        pos.x += speed * diff.x / diff.length();
        pos.y += speed * diff.y / diff.length();

        false
    } else {
        *pos = *dest;

        true
    }
}
*/

pub fn ease_out_sine(t: f32) -> f32 {
    ((t * std::f32::consts::PI) / 2.0).sin()
}
