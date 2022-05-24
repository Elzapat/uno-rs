use super::{LobbiesList, Lobby, LobbyState};
use crate::{game::StartGameEvent, utils::errors::Error, IncomingPackets, Server};
use bevy::prelude::*;
use itertools::Itertools;
use std::net::TcpStream;
use uno::packet::{Command, Packet, ARG_DELIMITER};
use uuid::Uuid;

pub fn connect_to_server(mut commands: Commands, mut state: ResMut<State<LobbyState>>) {
    // let stream = match TcpStream::connect("127.0.0.1:2905") {
    //     Ok(s) => s,
    //     Err(e) => {
    //         commands.spawn().insert(Error {
    //             message: format!("Couldn't connect to server ({}).\n\nYou can try reconnecting, or try another time because the service might be down.", e),
    //         });
    //         return;
    //     }
    // };

    let socket = match tungstenite::connect("ws://127.0.0.1:2905") {
        Ok((s, _)) => s,
        Err(e) => {
            commands.spawn().insert(Error {
                message: format!("Couldn't connect to server ({}).\n\nYou can try reconnecting, or try another time because the service might be down.", e),
            });
            return;
        }
    };

    // commands.spawn().insert(Server { socket });
    state.set(LobbyState::LobbiesList).unwrap();
}

pub fn execute_packets(
    mut commands: Commands,
    mut lobby_state: ResMut<State<LobbyState>>,
    mut lobbies: ResMut<LobbiesList>,
    mut current_lobby: ResMut<Option<Lobby>>,
    mut start_game_event: EventWriter<StartGameEvent>,
    mut incoming_packets: ResMut<IncomingPackets>,
) {
    let packets = incoming_packets.0.drain(..).collect::<Vec<Packet>>();

    for mut packet in packets {
        info!("{:?}", packet);
        match packet.command {
            Command::StartGame => start_game_event.send(StartGameEvent(
                current_lobby.as_ref().as_ref().unwrap().players.clone(),
            )),
            Command::JoinLobby => {
                let players = packet
                    .args
                    .get_range(1..)
                    .split(|&x| x == ARG_DELIMITER)
                    .tuples()
                    .map(|(id, username)| {
                        (
                            Uuid::from_slice(id).unwrap(),
                            String::from_utf8(username.to_vec()).unwrap(),
                        )
                    })
                    .collect::<Vec<(Uuid, String)>>();

                lobby_state.set(LobbyState::InLobby).unwrap();

                *current_lobby = Some(Lobby {
                    id: *packet.args.get(0).unwrap(),
                    number_players: 1,
                    players,
                });
            }
            Command::PlayerJoinedLobby => {
                for lobby in lobbies.0.iter_mut() {
                    if lobby.id == *packet.args.get(0).unwrap() {
                        lobby.number_players += 1;
                    }
                }

                if let LobbyState::InLobby = lobby_state.current() {
                    let args = packet.args.get_range(1..);
                    let delim_pos = args.iter().position(|&b| b == ARG_DELIMITER).unwrap();
                    let id = Uuid::from_slice(&args[..delim_pos]).unwrap();
                    let username = String::from_utf8(args[delim_pos + 1..].to_vec()).unwrap();
                    (*current_lobby)
                        .as_mut()
                        .unwrap()
                        .players
                        .push((id, username));
                }
            }
            Command::LeaveLobby => {
                lobby_state.set(LobbyState::LobbiesList).unwrap();
                *current_lobby = None;
            }
            Command::PlayerLeftLobby => {
                for lobby in lobbies.0.iter_mut() {
                    if lobby.id == *packet.args.get(0).unwrap() && lobby.number_players > 0 {
                        lobby.number_players -= 1;
                    }
                }

                if let LobbyState::InLobby = lobby_state.current() {
                    let id = Uuid::from_slice(&packet.args.get_range(1..)).unwrap();
                    if let Some(current_lobby) = (*current_lobby).as_mut() {
                        current_lobby.players.retain(|p| p.0 != id);
                    }
                }
            }
            Command::LobbyCreated => {
                let id = *packet.args.get(0).unwrap();
                lobbies.0.push(Lobby {
                    id,
                    number_players: 0,
                    players: Vec::new(),
                });
            }
            Command::LobbyDestroyed => {
                let id = *packet.args.get(0).unwrap();
                let idx = lobbies.0.iter().position(|l| l.id == id).unwrap();
                lobbies.0.remove(idx);
            }
            Command::LobbyInfo => {
                if let LobbyState::InLobby = lobby_state.current() {
                    if let Ok(players_raw) = String::from_utf8(packet.args.get_range(2..)) {
                        let _players = players_raw
                            .split(char::from_digit(ARG_DELIMITER.into(), 10).unwrap())
                            .map(|p| p.to_owned())
                            .collect::<Vec<String>>();
                    }
                }
            }
            Command::LobbiesInfo => {
                if let LobbyState::LobbiesList = lobby_state.current() {
                    lobbies.0 = packet
                        .args
                        .get_range(..)
                        .into_iter()
                        .tuples()
                        .map(|(id, number_players)| Lobby {
                            id,
                            number_players,
                            players: Vec::new(),
                        })
                        .collect::<Vec<Lobby>>();
                }
            }
            Command::Error => {
                if let Ok(error) = String::from_utf8(packet.args.get_range(..)) {
                    commands.spawn().insert(Error { message: error });
                }
            }
            _ => incoming_packets.0.push(packet),
        };
    }
}
