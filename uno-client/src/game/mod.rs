use crate::{
    utils::constants::{CARD_HEIGHT, CARD_PADDING, CARD_WIDTH, DISCARD_POS},
    Draggable, GameState, IncomingPackets,
};
use bevy::{ecs::schedule::ShouldRun, prelude::*};
use cards::*;
use uno::{
    card::Color,
    packet::{Command, Packet},
};
use uuid::Uuid;

mod cards;
mod ui;

pub struct GamePlugin;

// Components
#[derive(Component, Debug)]
pub struct Player {
    id: Uuid,
    hand_size: usize,
    is_playing: bool,
    score: u32,
    username: String,
}
#[derive(Component)]
pub struct ThisPlayer;
#[derive(Component)]
pub struct ChooseColor;
#[derive(Component)]
pub struct CallUno;
#[derive(Component)]
pub struct ToBeRemoved {
    timer: Timer,
}

// Ressources
pub struct GameAssets {
    background: Handle<Image>,
    cards: Handle<TextureAtlas>,
}

// Events
pub struct StartGameEvent(pub Vec<(Uuid, String)>);
pub struct ColorChosenEvent(pub Color);

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(cards::CardsPlugin)
            .add_plugin(ui::GameUiPlugin)
            .add_event::<StartGameEvent>()
            .add_event::<ColorChosenEvent>()
            .add_startup_system(load_assets)
            .add_system(start_game)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_in_game)
                    .with_system(execute_packets)
                    .with_system(to_be_removed),
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

fn start_game(
    mut commands: Commands,
    mut game_state: ResMut<State<GameState>>,
    mut start_game_event: EventReader<StartGameEvent>,
) {
    #[allow(clippy::never_loop)]
    for StartGameEvent(clients) in start_game_event.iter() {
        for client in clients {
            commands.spawn().insert(Player {
                id: client.0,
                hand_size: 7,
                score: 0,
                username: client.1.clone(),
                is_playing: false,
            });
        }

        game_state.set(GameState::Game).unwrap();
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
    mut incoming_packets: ResMut<IncomingPackets>,
    //
    mut draw_card_event: EventWriter<DrawCardEvent>,
    //
    mut reorganize_hand_event: EventWriter<ReorganizeHandEvent>,
    //
    mut card_played_event: EventWriter<CardPlayedEvent>,
    mut players_query: Query<(Entity, &mut Player)>,
    mut card_validation_query: Query<
        (Entity, &CardComponent, &mut CardPosition, &HandItem),
        With<CardWaitingForValidation>,
    >,
    mut hand_item_query: Query<&mut HandItem, Without<CardWaitingForValidation>>,
    mut hand: ResMut<Hand>,
    mut discard_query: Query<(Entity, &mut Transform), With<Discard>>,
) {
    let packets = incoming_packets.0.drain(..).collect::<Vec<Packet>>();

    for mut packet in packets {
        info!("{:?}", packet);
        match packet.command {
            Command::DrawCard => draw_card_event.send(DrawCardEvent(packet.args.into())),
            Command::CardPlayed => card_played_event.send(CardPlayedEvent(packet.args.into())),
            Command::CardValidation => {
                let valid = *packet.args.get(0).unwrap() != 0;

                if let Ok((entity, card, mut card_pos, played_card_item)) =
                    card_validation_query.get_single_mut()
                {
                    if valid {
                        card_pos.0 = Vec3::new(DISCARD_POS.0, DISCARD_POS.1, 0.0);

                        for (entity, mut transform) in discard_query.iter_mut() {
                            transform.translation.z = 0.0;
                            commands
                                .entity(entity)
                                .remove::<Discard>()
                                .insert(ToBeRemoved {
                                    timer: Timer::from_seconds(1.0, false),
                                });
                        }

                        commands
                            .entity(entity)
                            .remove::<Draggable>()
                            .remove::<HandItem>()
                            .insert(Discard);

                        hand.size -= 1;

                        for mut hand_item in hand_item_query.iter_mut() {
                            if hand_item.index > played_card_item.index {
                                hand_item.index -= 1;
                            }
                        }

                        reorganize_hand_event.send(ReorganizeHandEvent);

                        if card.0.color == Color::Black {
                            commands.spawn().insert(ChooseColor);
                        }

                        if hand.size == 1 {
                            commands.spawn().insert(CallUno);
                        }
                    }

                    commands
                        .entity(entity)
                        .remove::<CardWaitingForValidation>()
                        .insert(CardAnimation::default());
                }
            }
            Command::PassTurn => {
                let uuid = Uuid::from_slice(&packet.args.get_range(..)).unwrap();
                for (_, mut player) in players_query.iter_mut() {
                    player.is_playing = player.id == uuid;
                }
            }
            Command::YourPlayerId => {
                let uuid = Uuid::from_slice(&packet.args.get_range(..)).unwrap();
                for (entity, player) in players_query.iter() {
                    if player.id == uuid {
                        commands.entity(entity).insert(ThisPlayer);
                        break;
                    }
                }
            }
            Command::HandSize => {
                let nb_cards = *packet.args.get(0).unwrap();
                let uuid = Uuid::from_slice(&packet.args.get_range(1..)).unwrap();

                for (_, mut player) in players_query.iter_mut() {
                    if player.id == uuid {
                        player.hand_size = nb_cards as usize;
                        break;
                    }
                }
            }
            _ => incoming_packets.0.push(packet),
        }
    }
}

fn setup(mut commands: Commands, game_assets: Res<GameAssets>) {
    // Game background
    commands.spawn_bundle(SpriteBundle {
        texture: game_assets.background.clone(),
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
