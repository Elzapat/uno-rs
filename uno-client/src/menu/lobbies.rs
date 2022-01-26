use super::{LobbiesList, Lobby, LobbyState, RefreshTimer};
use crate::{utils::errors::Error, Server};
use bevy::prelude::*;
use itertools::Itertools;
use std::net::TcpStream;
use uno::packet::{read_socket, write_socket, Command, ARG_DELIMITER};
use uuid::Uuid;

pub fn connect_to_server(mut commands: Commands, mut state: ResMut<State<LobbyState>>) {
    let socket = match TcpStream::connect("127.0.0.1:2905") {
        Ok(s) => s,
        Err(e) => {
            commands.spawn().insert(Error {
                message: format!("Couldn't connect to server ({}).\n\nYou can try reconnecting, or try another time because the service might be down.", e),
            });
            return;
        }
    };

    socket
        .set_nonblocking(true)
        .expect("Couldn't set socket to nonblocking");
    state.set(LobbyState::LobbiesList).unwrap();
    commands.insert_resource(Server { socket });
}

pub fn refresh_lobbies_list(
    time: Res<Time>,
    mut refresh_timer: ResMut<RefreshTimer>,
    mut server: ResMut<Server>,
) {
    refresh_timer.0.tick(time.delta());

    if refresh_timer.0.finished() {
        write_socket(&mut server.socket, Command::LobbiesInfo, vec![]).unwrap();
    }
}

pub fn read_incoming(
    mut commands: Commands,
    mut server: ResMut<Server>,
    mut lobby_state: ResMut<State<LobbyState>>,
    mut lobbies: ResMut<LobbiesList>,
    mut current_lobby: ResMut<Option<Lobby>>,
) {
    if let Ok(packets) = read_socket(&mut server.socket) {
        // println!("{:?}", packets);
        for mut packet in packets {
            info!("{:?}", packet);
            match packet.command {
                Command::JoinLobby => {
                    let players = packet
                        .args
                        .get_range(1..)
                        .clone()
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
                    if let LobbyState::InLobby = lobby_state.current() {
                        let args = packet.args.get_range(..);
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
                    if let LobbyState::InLobby = lobby_state.current() {
                        let id = Uuid::from_slice(&packet.args.get_range(..)).unwrap();
                        (*current_lobby)
                            .as_mut()
                            .unwrap()
                            .players
                            .retain(|p| p.0 != id);
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
                Command::Error => {
                    if let Ok(error) = String::from_utf8(packet.args.get_range(..)) {
                        commands.spawn().insert(Error { message: error });
                    }
                }
                _ => {}
            };
        }
    }
}
