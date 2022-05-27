use super::{LobbiesList, LobbyState};
use crate::{game::StartGameEvent, utils::errors::Error};
use bevy::prelude::*;
use naia_bevy_client::events::MessageEvent;
use uno::Player;
use uno::{
    network::{Channels, Protocol},
    Lobby,
};
use uuid::Uuid;

pub fn execute_packets(
    mut commands: Commands,
    mut lobby_state: ResMut<State<LobbyState>>,
    mut lobbies: ResMut<LobbiesList>,
    mut current_lobby: ResMut<Option<Lobby>>,
    mut start_game_event: EventWriter<StartGameEvent>,
    mut message_events: EventReader<MessageEvent<Protocol, Channels>>,
) {
    for MessageEvent(channel, message) in message_events.iter() {
        if *channel != Channels::Lobby {
            return;
        }

        match message {
            Protocol::StartGame(_) => start_game_event.send(StartGameEvent(
                current_lobby.as_ref().as_ref().unwrap().players.clone(),
            )),
            Protocol::JoinLobby(lobby) => {
                lobby_state.set(LobbyState::InLobby).unwrap();

                *current_lobby = Some(Lobby {
                    id: *lobby.lobby_id,
                    players: lobby
                        .players
                        .iter()
                        .map(|(id, name)| {
                            Player::new(Uuid::from_slice(id.as_bytes()).unwrap(), name.clone())
                        })
                        .collect(),
                });
            }
            Protocol::PlayerJoinedLobby(joined_lobby) => {
                if let LobbyState::InLobby = lobby_state.current() {
                    (*current_lobby).as_mut().unwrap().players.push(Player::new(
                        Uuid::from_slice(joined_lobby.player_id.as_bytes()).unwrap(),
                        (*joined_lobby.player_name).clone(),
                    ));
                }
            }
            Protocol::LeaveLobby(_) => {
                lobby_state.set(LobbyState::LobbiesList).unwrap();
                *current_lobby = None;
            }
            Protocol::PlayerLeftLobby(left_lobby) => {
                for lobby in lobbies.iter_mut() {
                    lobby.players.retain(|p| {
                        p.id != Uuid::from_slice(left_lobby.player_id.as_bytes()).unwrap()
                    });
                }
            }
            Protocol::LobbyCreated(lobby) => {
                lobbies.push(Lobby {
                    id: *lobby.lobby_id,
                    players: Vec::new(),
                });
            }
            Protocol::LobbyDestroyed(lobby) => {
                let idx = lobbies.0.iter().position(|l| l.id == *lobby.lobby_id);
                if let Some(idx) = idx {
                    lobbies.0.remove(idx);
                }
            }
            Protocol::LobbyInfo(lobby) => {
                let players = lobby
                    .players
                    .iter()
                    .map(|(id, name)| {
                        Player::new(Uuid::from_slice(id.as_bytes()).unwrap(), name.clone())
                    })
                    .collect();

                for existing_lobby in lobbies.iter_mut() {
                    if existing_lobby.id == *lobby.lobby_id {
                        existing_lobby.players = players;
                        return;
                    }
                }

                lobbies.push(Lobby {
                    id: *lobby.lobby_id,
                    players,
                });
            }
            Protocol::Error(error) => {
                commands.spawn().insert(Error {
                    message: (*error.error).clone(),
                });
            }
            _ => {}
        };
    }
}
