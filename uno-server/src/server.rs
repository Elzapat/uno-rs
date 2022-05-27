use crate::{client::Client, game::Game};
use anyhow::Result;
use log::error;
use naia_server::{Event, Server as NaiaServer, ServerAddrs, ServerConfig, UserKey};
use naia_shared::{Protocolize, ReplicateSafe};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};
use uno::{
    lobby::{Lobby, LobbyId},
    network::{protocol, shared_config, Channels, Protocol},
};
use uuid::Uuid;

const MAX_LOBBY_PLAYERS: usize = 10;
const MAX_LOBBIES: usize = 10;

fn new_lobby_id() -> LobbyId {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    COUNTER
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            Some(value % (MAX_LOBBIES * 2) + 1)
        })
        .unwrap() as LobbyId
}

pub struct Server {
    server: Arc<Mutex<NaiaServer<Protocol, u64, Channels>>>,
    clients: Vec<Client>,
    lobbies: Vec<Lobby>,
    games: Vec<GameData>,
}

pub type Packet = dyn ReplicateSafe<Protocol>;

struct GameData {
    pub game_thread: thread::JoinHandle<()>,
    pub packets_receiver: Receiver<(UserKey, Box<Packet>)>,
    pub packets_sender: Sender<(UserKey, Box<Packet>)>,
}

impl Server {
    pub fn new() -> Server {
        let server_addresses = ServerAddrs::new(
            "0.0.0.0:2904".parse().unwrap(),
            "0.0.0.0:2905".parse().unwrap(),
            "http://0.0.0.0:2905",
        );

        let mut server = NaiaServer::new(&ServerConfig::default(), &shared_config());
        server.listen(&server_addresses);

        Server {
            server: Arc::new(Mutex::new(server)),
            clients: Vec::new(),
            lobbies: Vec::new(),
            games: Vec::new(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let server = self.server.lock().unwrap();

            for event in server.receive() {
                match event {
                    Ok(Event::Connection(user_key)) => {
                        self.server.accept_connection(&user_key);
                        self.clients.push(Client::new(user_key));
                    }
                    Ok(Event::Disconnection(user_key, _)) => {
                        self.clients.retain(|c| c.user_key != user_key);
                    }
                    Ok(Event::Message(user_key, Channels::Lobby, protocol)) => {
                        self.execute_command(user_key, protocol);
                    }
                }
            }

            for game in self.games {
                if let Ok((user_key, protocol)) = game.packets_receiver.try_recv() {
                    self.server
                        .send_message(&user_key, Channels::Game, &*protocol);
                }
            }
        }
    }

    fn execute_command(&mut self, user_key: UserKey, protocol: Protocol) -> Result<()> {
        let mut player_left = None;
        let mut player_joined = None;
        let mut lobby_created = None;
        let mut start_game = None;

        if let Some(client) = self
            .clients
            .iter_mut()
            .find(|client| client.user_key == user_key)
        {
            match protocol {
                Protocol::StartGame(_) => {
                    if let Some(lobby_id) = client.in_lobby {
                        start_game = Some(lobby_id);
                        self.lobbies.retain(|lobby| lobby.id != lobby_id);
                    }
                }
                Protocol::CreateLobby(_) => {
                    if self.lobbies.len() < MAX_LOBBIES {
                        let new_lobby_id = new_lobby_id();
                        self.lobbies.push(Lobby::new(new_lobby_id));

                        lobby_created = Some(new_lobby_id);
                    } else {
                        self.server.send_message(
                            &client.user_key,
                            Channels::Lobby,
                            &protocol::Error::new("Maximum lobby amount".to_owned()),
                        );
                    }
                }
                Protocol::JoinLobby(lobby) => {
                    if client.in_lobby.is_some() {
                        self.server.send_message(
                            &client.user_key,
                            Channels::Lobby,
                            &protocol::Error::new("You're already in a lobby".to_owned()),
                        );
                        return Ok(());
                    }

                    if let Some(lobby) = self.lobbies.iter_mut().find(|l| l.id == *lobby.lobby_id) {
                        if lobby.players.len() >= MAX_LOBBY_PLAYERS {
                            self.server.send_message(
                                &client.user_key,
                                Channels::Lobby,
                                &protocol::Error::new("The lobby is full".to_owned()),
                            );
                            return Ok(());
                        }

                        player_joined = Some((lobby.id, client.player));
                        lobby.players.push(client.player);

                        self.server.send_message(
                            &client.user_key,
                            Channels::Lobby,
                            &protocol::JoinLobby::new(lobby.id, lobby.players),
                        );
                        client.in_lobby = Some(lobby.id);
                    } else {
                        self.server.send_message(
                            &client.user_key,
                            Channels::Lobby,
                            &protocol::Error::new("Lobby doesn't exist".to_owned()),
                        );
                    }
                }
                Protocol::LeaveLobby(lobby) => {
                    if client.in_lobby.is_none() {
                        self.server.send_message(
                            &client.user_key,
                            Channels::Lobby,
                            &protocol::Error::new("You're not in a lobby".to_owned()),
                        );
                        return Ok(());
                    }

                    if let Some(lobby) = self.lobbies.iter_mut().find(|l| l.id == *lobby.lobby_id) {
                        player_left = Some((client.id, lobby.id));

                        lobby.players.retain(|player| player.id != client.id);

                        self.server.send_message(
                            &client.user_key,
                            Channels::Lobby,
                            &protocol::LeaveLobby::new(lobby.id),
                        );
                        client.in_lobby = None;
                    }
                }
                Protocol::Username(player) => {
                    client.player.username = *player.username;
                }
                _ => {}
            }
        }

        if let Some((player_id, lobby_id)) = player_left {
            for client in self.clients.iter_mut() {
                self.server.send_message(
                    &client.user_key,
                    Channels::Lobby,
                    &protocol::PlayerLeftLobby::new(lobby_id, player_id),
                )
            }
        } else if let Some((lobby_id, player)) = player_joined {
            for client in self.clients.iter_mut() {
                self.server.send_message(
                    &client.user_key,
                    Channels::Lobby,
                    &protocol::PlayerJoinedLobby::new(lobby_id, player),
                );
            }
        } else if let Some(id) = lobby_created {
            for client in self.clients.iter_mut() {
                self.server.send_message(
                    &client.user_key,
                    Channels::Lobby,
                    &protocol::LobbyCreated::new(id),
                );
            }
        } else if let Some(lobby_id) = start_game {
            // Get clients in lobby
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

            // Tell clients the lobby the game was started in is gone
            for client in self.clients.iter_mut() {
                self.server.send_message(
                    &client.user_key,
                    Channels::Lobby,
                    &protocol::LobbyDestroyed::new(lobby_id),
                );
            }

            self.lobbies.retain(|l| l.id != lobby_id);

            for client in clients.iter_mut() {
                self.server.send_message(
                    &client.user_key,
                    Channels::Lobby,
                    &protocol::StartGame::new(),
                );
            }

            let (game_tx, game_rx) = mpsc::channel();
            let (server_tx, server_rx) = mpsc::channel();
            self.games.push(GameData {
                game_thread: thread::spawn(|| {
                    if let Err(e) = Game::new(clients, game_tx, game_rx).run() {
                        error!("{e}");
                    }
                }),
                packets_receiver: server_rx,
                packets_sender: server_tx,
            });
        }

        Ok(())
    }
}
