mod lobbies;
mod ui;

use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
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

#[derive(Component)]
pub struct RefreshTimer(Timer);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LobbiesList(vec![]))
            .insert_resource(Option::<Lobby>::None)
            .insert_resource(RefreshTimer(Timer::from_seconds(1.0, true)))
            .add_startup_system(lobbies::connect_to_server)
            .add_state(LobbyState::Unconnected)
            .add_system_set(
                SystemSet::on_enter(LobbyState::LobbiesList)
                    .with_run_criteria(run_if_connected)
                    .with_system(lobbies::refresh_lobbies_list),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_connected)
                    .with_system(ui::settings_panel)
                    .with_system(lobbies::read_incoming)
                    .with_system(ui::lobby_panel),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_not_connected)
                    .with_system(ui::unconnected_panel),
            );
    }
}

fn run_if_connected(state: Res<State<LobbyState>>) -> ShouldRun {
    if state.current() != &LobbyState::Unconnected {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn run_if_not_connected(state: Res<State<LobbyState>>) -> ShouldRun {
    if state.current() == &LobbyState::Unconnected {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}
