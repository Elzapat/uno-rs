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
use uuid::Uuid;

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
    players: Vec<(Uuid, String)>,
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
                info!("New connection: {}", ip);

                // Send all lobbies info to the new client
                let mut infos = vec![];
                for (id, lobby) in &self.lobbies {
                    infos.push(*id as u8);
                    infos.push(lobby.players.len() as u8);
                }

                let mut client = Client::new(socket);
                write_socket(&mut client.socket, Command::LobbiesInfo, infos)?;
                self.clients.push(client);
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
            if let Some(lobby_id) = self.clients[i].in_lobby {
                let lobby = self.lobbies.get_mut(&lobby_id).unwrap();

                lobby.players.retain(|p| p.0 != self.clients[i].id);

                for client in self.clients.iter_mut() {
                    write_socket(
                        &mut client.socket,
                        Command::PlayerLeftLobby,
                        [&[lobby_id as u8], client.id.as_bytes().as_slice()].concat(),
                    )?;
                }
            }

            self.clients.remove(i);
        }

        Ok(())
    }

    fn execute_commands(&mut self) -> Result {
        let mut player_left = None;
        let mut player_joined = None;
        let mut lobby_created = None;
        let mut start_game = None;

        for client in self.clients.iter_mut() {
            for mut packet in client.incoming_packets.drain(..) {
                match packet.command {
                    Command::StartGame => {
                        if let Some(lobby_id) = client.in_lobby {
                            start_game = Some(lobby_id);
                            self.lobbies.remove(&lobby_id);
                        }
                    }
                    Command::CreateLobby => {
                        if self.lobbies.len() < MAX_LOBBIES {
                            let new_lobby_id = new_lobby_id();
                            self.lobbies.insert(new_lobby_id, Lobby { players: vec![] });

                            lobby_created = Some(new_lobby_id as u8);
                        } else {
                            write_socket(
                                &mut client.socket,
                                Command::Error,
                                "Maximum lobby amount".as_bytes(),
                            )?;
                        }
                    }
                    Command::JoinLobby => {
                        if client.in_lobby.is_some() {
                            write_socket(
                                &mut client.socket,
                                Command::Error,
                                "You're already in a lobby".as_bytes(),
                            )?;
                            return Ok(());
                        }

                        if let Some(lobby_id) = packet.args.get(0) {
                            if let Some(ref mut lobby) = self.lobbies.get_mut(&(*lobby_id as usize))
                            {
                                if lobby.players.len() >= MAX_LOBBY_PLAYERS {
                                    write_socket(
                                        &mut client.socket,
                                        Command::Error,
                                        "This lobby is full".as_bytes(),
                                    )?;
                                    return Ok(());
                                }

                                player_joined = Some((
                                    client.id,
                                    client.player.username.clone(),
                                    *lobby_id as usize,
                                ));

                                lobby
                                    .players
                                    .push((client.id, client.player.username.clone()));

                                let mut args = vec![*lobby_id];

                                for c in &lobby.players {
                                    args.extend_from_slice(c.0.as_bytes());
                                    args.push(ARG_DELIMITER);
                                    args.extend_from_slice(c.1.as_bytes());
                                    args.push(ARG_DELIMITER);
                                }

                                // Remove the last ARG_DELIMITER that is not needed
                                args.pop();

                                write_socket(&mut client.socket, Command::JoinLobby, args)?;
                                client.in_lobby = Some(*lobby_id as usize);
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
                        if client.in_lobby.is_none() {
                            write_socket(
                                &mut client.socket,
                                Command::Error,
                                "You're not in a lobby".as_bytes(),
                            )?;
                            return Ok(());
                        }

                        if let Some(lobby_id) = packet.args.get(0) {
                            if let Some(lobby) = self.lobbies.get_mut(&(*lobby_id as usize)) {
                                player_left = Some((client.id, *lobby_id as usize));

                                lobby.players.retain(|p| p.0 != client.id);

                                write_socket(&mut client.socket, Command::LeaveLobby, vec![])?;
                                client.in_lobby = None;
                            }
                        }
                    }
                    Command::Username => {
                        if let Ok(username) = String::from_utf8(packet.args.get_range(..)) {
                            client.player.username = username;
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some((id, lobby_id)) = player_left {
            for client in self.clients.iter_mut() {
                write_socket(
                    &mut client.socket,
                    Command::PlayerLeftLobby,
                    [&[lobby_id as u8], id.as_bytes().as_slice()].concat(),
                )?;
            }
        } else if let Some((id, username, lobby_id)) = player_joined {
            for client in self.clients.iter_mut() {
                write_socket(
                    &mut client.socket,
                    Command::PlayerJoinedLobby,
                    [
                        &[lobby_id as u8],
                        id.as_bytes().as_slice(),
                        &[ARG_DELIMITER],
                        username.as_bytes(),
                    ]
                    .concat(),
                )?;
            }
        } else if let Some(id) = lobby_created {
            for client in self.clients.iter_mut() {
                write_socket(&mut client.socket, Command::LobbyCreated, id)?;
            }
        } else if let Some(lobby_id) = start_game {
            let mut clients = self
                .clients
                .drain_filter(|client| {
                    if let Some(id) = client.in_lobby {
                        id == lobby_id
                    } else {
                        false
                    }
                })
                .collect::<Vec<Client>>();

            self.lobbies.remove(&lobby_id);

            for client in clients.iter_mut() {
                write_socket(&mut client.socket, Command::StartGame, vec![])?;
            }

            self.game_threads
                .push(thread::spawn(|| Game::new(clients).run()));

            for client in self.clients.iter_mut() {
                write_socket(&mut client.socket, Command::LobbyDestroyed, lobby_id as u8)?;
            }
        }

        Ok(())
    }
}
