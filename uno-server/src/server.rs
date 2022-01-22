use crate::{client::Client, game::Game};
use log::{error, info};
use std::{
    collections::HashMap,
    net::TcpListener,
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};
use uno::{
    packet::{read_socket, write_socket, Command, ARG_DELIMITER},
    prelude::*,
};

const MAX_LOBBY_PLAYERS: usize = 10;
const MAX_LOBBIES: usize = 10;

fn new_lobby_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    COUNTER
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            Some(value % (MAX_LOBBIES * 2) + 1)
        })
        .unwrap()
}

struct Lobby {
    number_players: usize,
    players: Vec<String>,
}

pub struct Server {
    listener: TcpListener,
    clients: Vec<Client>,
    lobbies: HashMap<usize, Lobby>,
    game_threads: Vec<thread::JoinHandle<()>>,
}

impl Server {
    fn new() -> Result<Server> {
        Ok(Server {
            listener: TcpListener::bind("0.0.0.0:2905")?,
            clients: Vec::new(),
            lobbies: HashMap::new(),
            game_threads: Vec::new(),
        })
    }

    pub fn run() -> Result {
        let mut server = Server::new()?;
        server.listener.set_nonblocking(true)?;

        loop {
            server.new_connections()?;
            server.read_sockets()?;
            server.execute_commands()?;
        }
    }

    fn new_connections(&mut self) -> Result {
        match self.listener.accept() {
            Ok((socket, ip)) => {
                socket.set_nonblocking(true)?;
                self.clients.push(Client::new(socket));
                info!("New connection: {}", ip);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => error!("{}", e),
        };

        Ok(())
    }

    fn read_sockets(&mut self) -> Result {
        let mut to_remove = None;

        for (i, client) in self.clients.iter_mut().enumerate() {
            client.incoming_packets = match read_socket(&mut client.socket) {
                Ok(packets) => {
                    info!("{:?}", packets);
                    packets
                }
                Err(e) => {
                    if let Error::IoError(e) = e {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            continue;
                        } else {
                            return Err(Error::IoError(e));
                        }
                    } else if let Error::UnoError(uno::error::UnoError::Disconnected) = e {
                        to_remove = Some(i);
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            };
        }

        if let Some(i) = to_remove {
            self.clients.remove(i);
        }

        Ok(())
    }

    fn execute_commands(&mut self) -> Result {
        for client in self.clients.iter_mut() {
            for mut packet in client.incoming_packets.drain(..) {
                match packet.command {
                    Command::CreateLobby => {
                        if self.lobbies.len() < MAX_LOBBIES {
                            let new_lobby_id = new_lobby_id();
                            /*
                            write_socket(
                                &mut client.socket,
                                Command::JoinLobby,
                                new_lobby_id as u8,
                            )?;
                            */
                            self.lobbies.insert(
                                new_lobby_id,
                                Lobby {
                                    number_players: 0,
                                    players: vec![],
                                },
                            );
                        } else {
                            write_socket(
                                &mut client.socket,
                                Command::Error,
                                "Maximum lobby amount".as_bytes(),
                            )?;
                        }
                    }
                    Command::JoinLobby => {
                        if let Some(lobby_id) = packet.args.get(0) {
                            if let Some(ref mut lobby) = self.lobbies.get_mut(&(*lobby_id as usize))
                            {
                                lobby.number_players += 1;
                                lobby.players.push(match &client.player {
                                    Some(p) => p.username.clone(),
                                    None => "Player".to_owned(),
                                });

                                let mut args = vec![*lobby_id];

                                for player in &lobby.players {
                                    args.extend_from_slice(player.as_bytes());
                                    args.push(ARG_DELIMITER);
                                }

                                write_socket(&mut client.socket, Command::JoinLobby, args)?;
                            } else {
                                write_socket(
                                    &mut client.socket,
                                    Command::Error,
                                    "Lobby doesn't exist".as_bytes(),
                                )?;
                            }
                        }
                    }
                    Command::LeaveLobby => {
                        if let Some(lobby_id) = packet.args.get(0) {
                            write_socket(&mut client.socket, Command::LeaveLobby, vec![])?;
                            if let Some(lobby) = self.lobbies.get_mut(&(*lobby_id as usize)) {
                                if lobby.number_players > 0 {
                                    lobby.number_players -= 1;
                                }
                            }
                        }
                    }
                    Command::LobbiesInfo => {
                        let mut infos = vec![];

                        for (id, lobby) in &self.lobbies {
                            infos.push(*id as u8);
                            infos.push(lobby.number_players as u8);
                        }

                        write_socket(&mut client.socket, Command::LobbiesInfo, infos)?;
                    }
                    Command::LobbyInfo => {
                        if let Some(lobby_id) = packet.args.get(0) {
                            if let Some(lobby) = self.lobbies.get(&(*lobby_id as usize)) {
                                let mut args = vec![
                                    *lobby_id,
                                    self.lobbies[&(*lobby_id as usize)].number_players as u8,
                                ];

                                for player in &lobby.players {
                                    args.extend_from_slice(player.as_bytes());
                                    args.push(ARG_DELIMITER);
                                }

                                write_socket(&mut client.socket, Command::LobbyInfo, args)?;
                            }
                        }
                    }
                    Command::Username => {
                        if let Ok(username) = String::from_utf8(packet.args.get_range(..)) {
                            match client.player {
                                None => client.player = Some(Player::new(username)),
                                Some(ref mut p) => p.username = username,
                            };
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
