mod lobbies;
mod ui;

use crate::GameState;
use bevy::{ecs::schedule::ShouldRun, prelude::*};
use uuid::Uuid;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum LobbyState {
    InLobby,
    LobbiesList,
    Unconnected,
}

pub struct LobbiesList(Vec<Lobby>);

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Lobby {
    id: u8,
    number_players: u8,
    players: Vec<(Uuid, String)>,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LobbiesList(vec![]))
            .insert_resource(Option::<Lobby>::None)
            .add_startup_system(lobbies::connect_to_server)
            .add_state(LobbyState::Unconnected)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_connected)
                    .with_system(ui::settings_panel)
                    .with_system(lobbies::execute_packets),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new()
                    .with_run_criteria(run_if_connected)
                    .with_system(ui::lobby_panel),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_not_connected)
                    .with_system(ui::unconnected_panel),
            );
    }
}

fn run_if_connected(
    lobby_state: Res<State<LobbyState>>,
    game_state: Res<State<GameState>>,
) -> ShouldRun {
    if lobby_state.current() != &LobbyState::Unconnected
        && game_state.current() == &GameState::Lobbies
    {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn run_if_not_connected(
    lobby_state: Res<State<LobbyState>>,
    game_state: Res<State<GameState>>,
) -> ShouldRun {
    if lobby_state.current() == &LobbyState::Unconnected
        && game_state.current() == &GameState::Lobbies
    {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}
