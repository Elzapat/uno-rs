use crate::{
    utils::constants::{CARD_HEIGHT, CARD_PADDING, CARD_WIDTH},
    GameState,
};
use bevy::{ecs::schedule::ShouldRun, prelude::*};
use cards::*;
use naia_bevy_client::events::MessageEvent;
use uno::{
    card::Color,
    network::{Channels, Protocol},
    Player as UnoPlayer,
};

mod cards;
mod ui;

pub struct GamePlugin;

// Components
#[derive(Debug, Component, Deref, DerefMut)]
pub struct Player(UnoPlayer);
#[derive(Component)]
pub struct ThisPlayer;
#[derive(Component)]
pub struct ChooseColor;
#[derive(Component)]
pub struct CallUno;
#[derive(Component)]
pub struct CallCounterUno;
#[derive(Component)]
pub struct DrawCard;
#[derive(Component)]
pub struct ToBeRemoved {
    timer: Timer,
}
#[derive(Component)]
pub struct Winner;

// Ressources
pub struct GameAssets {
    cards: Handle<TextureAtlas>,
}

// Events
pub struct StartGameEvent;
#[derive(Deref, DerefMut)]
pub struct ColorChosenEvent(pub Color);
#[derive(Deref, DerefMut)]
pub struct PlayedCardValidationEvent(pub bool);
pub struct GameEndEvent;
#[derive(Deref, DerefMut)]
pub struct ExtraMessageEvent(pub Protocol);
pub struct GameExitEvent;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(cards::CardsPlugin)
            .add_plugin(ui::GameUiPlugin)
            .add_event::<StartGameEvent>()
            .add_event::<ColorChosenEvent>()
            .add_event::<PlayedCardValidationEvent>()
            .add_event::<GameEndEvent>()
            .add_event::<ExtraMessageEvent>()
            .add_event::<GameExitEvent>()
            .add_startup_system(load_assets)
            .add_system(start_game)
            .add_system(game_exit)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_in_game)
                    .with_system(extra_messages)
                    .with_system(execute_packets)
                    .with_system(to_be_removed),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new().with_system(game_end),
            );
    }
}

fn run_if_in_game(game_state: Res<State<GameState>>) -> ShouldRun {
    if game_state.current() == &GameState::Game {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn run_if_in_end_game_lobby(game_state: Res<State<GameState>>) -> ShouldRun {
    if game_state.current() == &GameState::EndLobby {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn start_game(
    mut game_state: ResMut<State<GameState>>,
    mut start_game_event: EventReader<StartGameEvent>,
) {
    for StartGameEvent in start_game_event.iter() {
        if game_state.current() != &GameState::Game {
            game_state.set(GameState::Game).unwrap();
        }
    }
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let cards = asset_server.load("cards_02.png");
    let cards_atlas = TextureAtlas::from_grid_with_padding(
        cards,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        12,
        6,
        Vec2::splat(CARD_PADDING),
    );

    let background = asset_server.load("game_background.png");

    commands.insert_resource(GameAssets {
        cards: texture_atlases.add(cards_atlas),
    });

    // Game background
    commands.spawn_bundle(SpriteBundle {
        texture: background,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..SpriteBundle::default()
    });
}

fn extra_messages(
    mut extra_message_events: EventReader<ExtraMessageEvent>,
    mut message_events: EventWriter<MessageEvent<Protocol, Channels>>,
) {
    for ExtraMessageEvent(extra_message) in extra_message_events.iter() {
        message_events.send(MessageEvent(Channels::Uno, extra_message.clone()));
    }
}

fn execute_packets(
    mut commands: Commands,
    mut message_events: EventReader<MessageEvent<Protocol, Channels>>,
    uno_query: Query<Entity, With<CallUno>>,
    counter_uno_query: Query<Entity, With<CallCounterUno>>,
    mut draw_card_event: EventWriter<DrawCardEvent>,
    mut played_card_validation_event: EventWriter<PlayedCardValidationEvent>,
    mut card_played_event: EventWriter<CardPlayedEvent>,
    mut game_end_event: EventWriter<GameEndEvent>,
) {
    for MessageEvent(_, message) in message_events.iter() {
        match message {
            Protocol::GameEnd(_) => {
                game_end_event.send(GameEndEvent);
            }
            Protocol::DrawCard(card) => {
                draw_card_event.send(DrawCardEvent((*card.color, *card.value).into()))
            }
            Protocol::CardPlayed(card) => {
                card_played_event.send(CardPlayedEvent((*card.color, *card.value).into()))
            }
            Protocol::CardValidation(validation) => {
                played_card_validation_event.send(PlayedCardValidationEvent(*validation.valid));
            }
            Protocol::HaveToDrawCard(_) => {
                commands.spawn().insert(DrawCard);
            }
            Protocol::Uno(_) => {
                commands.spawn().insert(CallUno);
            }
            Protocol::CounterUno(_) => {
                commands.spawn().insert(CallCounterUno);
            }
            Protocol::StopUno(_) => {
                for entity in uno_query.iter() {
                    commands.entity(entity).despawn();
                }
                for entity in counter_uno_query.iter() {
                    commands.entity(entity).despawn();
                }
            }
            _ => {}
        }
    }
}

fn to_be_removed(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ToBeRemoved)>,
) {
    for (entity, mut tbr) in query.iter_mut() {
        tbr.timer.tick(time.delta());

        if tbr.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn game_end(
    mut commands: Commands,
    mut game_end_event: EventReader<GameEndEvent>,
    mut game_state: ResMut<State<GameState>>,
    cards_query: Query<Entity, With<TextureAtlasSprite>>,
) {
    for GameEndEvent in game_end_event.iter() {
        if game_state.current() != &GameState::EndLobby {
            game_state.set(GameState::EndLobby).unwrap();
        }

        for entity in cards_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

fn game_exit(
    mut commands: Commands,
    mut game_exit_event: EventReader<GameExitEvent>,
    mut game_state: ResMut<State<GameState>>,
    draw_card_query: Query<Entity, With<DrawCard>>,
    players_query: Query<Entity, With<Player>>,
) {
    for _e in game_exit_event.iter() {
        for entity in players_query.iter() {
            commands.entity(entity).despawn();
        }

        for entity in draw_card_query.iter() {
            commands.entity(entity).despawn();
        }

        if game_state.current() != &GameState::Lobbies {
            game_state.set(GameState::Lobbies).unwrap();
        }
    }
}
