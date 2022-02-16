use crate::{
    utils::constants::{CARD_HEIGHT, CARD_PADDING, CARD_WIDTH},
    GameState, IncomingPackets,
};
use bevy::{ecs::schedule::ShouldRun, prelude::*};
use cards::{CardsPlugin, SpawnCardEvent};
use uno::packet::{Command, Packet};

mod cards;

pub struct GamePlugin;

// Ressource
pub struct GameAssets {
    background: Handle<Image>,
    cards: Handle<TextureAtlas>,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CardsPlugin)
            .add_startup_system(load_assets)
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
    mut incoming_packets: ResMut<IncomingPackets>,
    mut spawn_card_event: EventWriter<SpawnCardEvent>,
) {
    let packets = incoming_packets.0.drain(..).collect::<Vec<Packet>>();

    for packet in packets {
        info!("{:?}", packet);
        match packet.command {
            Command::DrawCard => {
                let card = (*packet.args.get(0).unwrap(), *packet.args.get(1).unwrap()).into();
                spawn_card_event.send(SpawnCardEvent(card));
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
