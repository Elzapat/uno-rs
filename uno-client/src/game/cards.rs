use super::{run_if_in_game, GameAssets, Player, ThisPlayer};
use crate::{
    utils::constants::{
        CARD_ANIMATION_TIME_S, CARD_DROP_ZONE, CARD_HEIGHT, CARD_SCALE, CARD_WIDTH, DECK_POS,
        DISCARD_POS, Z_INCREASE,
    },
    Draggable, Dragged, Dropped, GameState, Server, SpriteSize,
};
use bevy::{prelude::*, window::WindowResized};
use std::time::Duration;
use uno::{
    card::{Card, Color, Value},
    packet::{write_socket, Command},
};

// Ressources
pub struct CurrentCardZ(pub f32);
pub struct Hand {
    pub size: usize,
}

// Components
#[derive(Component)]
pub struct CardComponent(pub Card);
#[derive(Component)]
pub struct Discard;
#[derive(Component)]
pub struct HandItem {
    pub index: usize,
}
#[derive(Component)]
pub struct CardPosition(pub Vec3);
#[derive(Component, Default)]
pub struct CardAnimation {
    time: Duration,
}
#[derive(Component, Default)]
pub struct CardWaitingForValidation;

// Events
pub struct DrawCardEvent(pub Card);
pub struct CardPlayedEvent(pub Card);
struct PlayCardEvent(pub Card);
pub struct ReorganizeHandEvent;

pub struct CardsPlugin;

impl Plugin for CardsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentCardZ(Z_INCREASE))
            .insert_resource(Hand { size: 0 })
            .add_event::<DrawCardEvent>()
            .add_event::<CardPlayedEvent>()
            .add_event::<PlayCardEvent>()
            .add_event::<ReorganizeHandEvent>()
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(setup_cards))
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_in_game)
                    .with_system(draw_card)
                    .with_system(card_played)
                    .with_system(reorganize_hand)
                    .with_system(card_dropped)
                    .with_system(animate_card)
                    .with_system(window_resized)
                    .with_system(remove_animation_on_drag)
                    .with_system(play_card),
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

fn card_played(
    mut commands: Commands,
    mut card_played_event: EventReader<CardPlayedEvent>,
    discard_query: Query<Entity, With<Discard>>,
    game_assets: Res<GameAssets>,
) {
    for CardPlayedEvent(card) in card_played_event.iter() {
        let mut transform = Transform::from_xyz(2000.0, DECK_POS.1, 0.0);
        transform.scale = Vec3::new(CARD_SCALE, CARD_SCALE, 1.0);

        for entity in discard_query.iter() {
            commands.entity(entity).despawn();
        }

        commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: card_to_spritesheet_index(card),
                    ..TextureAtlasSprite::default()
                },
                texture_atlas: game_assets.cards.clone_weak(),
                transform,
                ..SpriteSheetBundle::default()
            })
            .insert(Discard)
            .insert(CardAnimation::default())
            .insert(CardPosition(Vec3::new(DISCARD_POS.0, DISCARD_POS.1, 0.0)));
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

fn draw_card(
    mut commands: Commands,
    mut draw_card_event: EventReader<DrawCardEvent>,
    mut reorganize_hand_event: EventWriter<ReorganizeHandEvent>,
    mut current_card_z: ResMut<CurrentCardZ>,
    mut hand: ResMut<Hand>,
    game_assets: Res<GameAssets>,
) {
    for DrawCardEvent(card) in draw_card_event.iter() {
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
            .insert(CardPosition(card_transform.translation))
            .insert(HandItem { index: hand.size });

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
    mut query: Query<(Entity, &HandItem, &mut CardPosition)>,
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

        for (entity, item, mut card_position) in query.iter_mut() {
            card_position.0.y = card_y;
            card_position.0.x = part * item.index as f32 + part / 2.0 - width / 2.0;

            commands.entity(entity).insert(CardAnimation::default());
        }

        // We don't care how many times the event has been queued, we reorganize once
        break;
    }
}

fn card_dropped(
    mut commands: Commands,
    mut play_card_event: EventWriter<PlayCardEvent>,
    query: Query<(Entity, &Transform, &CardComponent), With<Dropped>>,
    this_player: Query<&Player, With<ThisPlayer>>,
) {
    for (entity, transform, card) in query.iter() {
        if this_player.single().is_playing
            && transform.translation.x < DISCARD_POS.0 + CARD_DROP_ZONE
            && transform.translation.x > DISCARD_POS.0 - CARD_DROP_ZONE
            && transform.translation.y < DISCARD_POS.1 + CARD_DROP_ZONE
            && transform.translation.y > DISCARD_POS.1 - CARD_DROP_ZONE
        {
            commands
                .entity(entity)
                .remove::<Dropped>()
                .insert(CardWaitingForValidation);

            play_card_event.send(PlayCardEvent(card.0))
        } else {
            commands
                .entity(entity)
                .remove::<Dropped>()
                .insert(CardAnimation::default());
        }
    }
}

fn play_card(
    mut server_query: Query<&mut Server>,
    mut play_card_event: EventReader<PlayCardEvent>,
) {
    for PlayCardEvent(card) in play_card_event.iter() {
        let mut server = server_query.single_mut();

        write_socket::<uno::packet::Args>(&mut server.socket, Command::PlayCard, (*card).into())
            .unwrap();
    }
}

fn animate_card(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &CardPosition, &mut Transform, &mut CardAnimation), Without<Dragged>>,
) {
    for (entity, card_position, mut transform, mut card_animation) in query.iter_mut() {
        card_animation.time += time.delta();

        if card_animation.time.as_secs_f32() >= CARD_ANIMATION_TIME_S {
            transform.translation = card_position.0;
            commands.entity(entity).remove::<CardAnimation>();
            return;
        }

        transform.translation = transform.translation.lerp(
            card_position.0,
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