use super::{LobbiesList, LobbyState};
use crate::{
    game::{ExtraMessageEvent, StartGameEvent},
    utils::errors::Error,
};
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
    mut start_game_event: EventWriter<StartGameEvent>,
    mut message_events: EventReader<MessageEvent<Protocol, Channels>>,
    mut extra_message_events: EventWriter<ExtraMessageEvent>,
) {
    for MessageEvent(_, protocol) in message_events.iter() {
        dbg!("received_message");
        match protocol {
            Protocol::StartGame(_) => {
                println!("in start game");
                if let LobbyState::InLobby(lobby_id) = lobby_state.current() {
                    for lobby in lobbies.iter() {
                        if lobby.id == *lobby_id {
                            start_game_event.send(StartGameEvent(lobby.players.clone()));
                            break;
                        }
                    }
                }

                if lobby_state.current() != &LobbyState::LobbiesList {
                    lobby_state.set(LobbyState::LobbiesList).unwrap();
                    lobbies.clear();
                }
            }
            Protocol::JoinLobby(lobby) => {
                lobby_state.set(LobbyState::InLobby(*lobby.id)).unwrap();
            }
            Protocol::LeaveLobby(_) => {
                lobby_state.set(LobbyState::LobbiesList).unwrap();
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
            /*
                    Protocol::LobbyInfo(lobby) => {
                        let players = lobby
                            .players
                            .iter()
                            .map(|(id, name)| Player::new(Uuid::parse_str(id).unwrap(), name.clone()))
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
            */
            Protocol::Error(error) => {
                commands.spawn().insert(Error {
                    message: (*error.error).clone(),
                });
            }
            protocol => {
                extra_message_events.send(ExtraMessageEvent(protocol.clone()));
                return;
            }
        };
    }
}
