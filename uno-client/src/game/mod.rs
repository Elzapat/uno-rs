use crate::{
    utils::constants::{CARD_HEIGHT, CARD_PADDING, CARD_WIDTH},
    GameState,
};
use bevy::{ecs::schedule::ShouldRun, prelude::*};
use cards::*;
use naia_bevy_client::events::MessageEvent;
use uno::{
    card::{Card, Color},
    network::{Channels, Protocol},
    Player as UnoPlayer,
};
use uuid::Uuid;

mod cards;
mod ui;

pub struct GamePlugin;

// Components
#[derive(Component, Deref, DerefMut)]
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
    background: Handle<Image>,
    cards: Handle<TextureAtlas>,
}
#[derive(Deref, DerefMut)]
pub struct CurrentColor(Color);

// Events
#[derive(Deref, DerefMut)]
pub struct StartGameEvent(pub Vec<UnoPlayer>);
#[derive(Deref, DerefMut)]
pub struct ColorChosenEvent(pub Color);
#[derive(Deref, DerefMut)]
pub struct PlayedCardValidationEvent(pub bool);
#[derive(Deref, DerefMut)]
pub struct GameEndEvent(pub Uuid);

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(cards::CardsPlugin)
            .add_plugin(ui::GameUiPlugin)
            .insert_resource(CurrentColor(Color::Black))
            .add_event::<StartGameEvent>()
            .add_event::<ColorChosenEvent>()
            .add_event::<PlayedCardValidationEvent>()
            .add_event::<GameEndEvent>()
            .add_startup_system(load_assets)
            .add_system(start_game)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_in_game)
                    .with_system(execute_packets)
                    .with_system(to_be_removed),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new().with_system(game_end),
            )
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(setup));
    }
}

fn run_if_in_game(game_state: Res<State<GameState>>) -> ShouldRun {
    if game_state.current() == &GameState::Game || game_state.current() == &GameState::EndLobby {
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
    mut commands: Commands,
    mut game_state: ResMut<State<GameState>>,
    mut start_game_event: EventReader<StartGameEvent>,
) {
    #[allow(clippy::never_loop)]
    for StartGameEvent(clients) in start_game_event.iter() {
        for client in clients.iter() {
            info!("CLIENT STRAT GAME = {client:?}");
            let mut client = client.clone();
            client.hand = vec![Card::back(); 7];
            commands.spawn().insert(Player(client));
        }

        if game_state.current() != &GameState::Game {
            game_state.set(GameState::Game).unwrap();
        }
        break;
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

    commands.insert_resource(GameAssets {
        background: asset_server.load("game_background.png"),
        cards: texture_atlases.add(cards_atlas),
    });
}

fn execute_packets(
    mut commands: Commands,
    mut message_events: EventReader<MessageEvent<Protocol, Channels>>,
    uno_query: Query<Entity, With<CallUno>>,
    counter_uno_query: Query<Entity, With<CallCounterUno>>,
    mut players_query: Query<(Entity, &mut Player)>,
    mut draw_card_event: EventWriter<DrawCardEvent>,
    mut played_card_validation_event: EventWriter<PlayedCardValidationEvent>,
    mut card_played_event: EventWriter<CardPlayedEvent>,
    mut game_end_event: EventWriter<GameEndEvent>,
    mut current_color: ResMut<CurrentColor>,
) {
    for MessageEvent(channel, message) in message_events.iter() {
        info!("RECEIVED MESSAGE IN GAME");
        // if *channel != Channels::Game {
        //     info!("WRONG CHANNEL NOT GAME");
        //     match message {
        //         Protocol::StartGame(_) => info!("START GAME"),
        //         _ => info!("SOMETHING ELSE"),
        //     }
        //     return;
        // }

        match message {
            Protocol::GameEnd(winner) => {
                game_end_event.send(GameEndEvent(Uuid::parse_str(&*winner.winner_id).unwrap()))
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
            Protocol::PassTurn(playing_player) => {
                let uuid = Uuid::parse_str(&*playing_player.playing_id).unwrap();

                for (_, mut player) in players_query.iter_mut() {
                    player.is_playing = player.id == uuid;
                }
            }
            Protocol::YourPlayerId(id) => {
                let uuid = Uuid::parse_str(&*id.player_id).unwrap();
                for (entity, player) in players_query.iter() {
                    if player.id == uuid {
                        commands.entity(entity).insert(ThisPlayer);
                        break;
                    }
                }
            }
            Protocol::HandSize(hand) => {
                let uuid = Uuid::parse_str(&*hand.player_id).unwrap();

                for (_, mut player) in players_query.iter_mut() {
                    if player.id == uuid {
                        player.hand = vec![Card::back(); *hand.size as usize];
                        break;
                    }
                }
            }
            Protocol::HaveToDrawCard(_) => {
                commands.spawn().insert(DrawCard);
            }
            Protocol::Uno(_) => {
                commands.spawn().insert(CallUno);
            }
            Protocol::StopUno(_) => {
                for entity in uno_query.iter() {
                    commands.entity(entity).despawn();
                }
            }
            Protocol::CounterUno(_) => {
                commands.spawn().insert(CallCounterUno);
            }
            Protocol::StopCounterUno(_) => {
                for entity in counter_uno_query.iter() {
                    commands.entity(entity).despawn();
                }
            }
            Protocol::PlayerScore(score) => {
                let uuid = Uuid::parse_str(&*score.player_id).unwrap();

                for (_, mut player) in players_query.iter_mut() {
                    if player.id == uuid {
                        player.score = *score.score;
                        break;
                    }
                }
            }
            Protocol::CurrentColor(color) => **current_color = (*color.color).into(),
            _ => {}
        }
    }
}

fn setup(mut commands: Commands, game_assets: Res<GameAssets>) {
    // Game background
    commands.spawn_bundle(SpriteBundle {
        texture: game_assets.background.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..SpriteBundle::default()
    });
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
    players_query: Query<(Entity, &Player)>,
    cards_query: Query<Entity, With<TextureAtlasSprite>>,
) {
    for GameEndEvent(winner_uuid) in game_end_event.iter() {
        if game_state.current() != &GameState::EndLobby {
            game_state.set(GameState::EndLobby).unwrap();
        }

        for (entity, player) in players_query.iter() {
            if &player.id == winner_uuid {
                commands.entity(entity).insert(Winner);
                break;
            }
        }

        for entity in cards_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}
