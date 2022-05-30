use crate::{
    client::Client,
    game::Game,
    world::{Entity, World},
};
use naia_server::{Event, Server as NaiaServerType, ServerAddrs, ServerConfig, UserKey};
use std::sync::atomic::{AtomicUsize, Ordering};
use uno::{
    lobby::{Lobby, LobbyId},
    network::{protocol, shared_config, Channels, Protocol},
};

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

pub type NaiaServer = NaiaServerType<Protocol, Entity, Channels>;

pub struct Server {
    server: NaiaServer,
    clients: Vec<Client>,
    lobbies: Vec<Lobby>,
    games: Vec<Game>,
}

impl Server {
    pub fn new() -> Server {
        let server_addresses = ServerAddrs::new(
            "0.0.0.0:3478".parse().unwrap(),
            "0.0.0.0:3478".parse().unwrap(),
            "http://127.0.0.1:3478",
        );

        let mut server = NaiaServer::new(
            &ServerConfig {
                // require_auth: false,
                ..ServerConfig::default()
            },
            &shared_config(),
        );
        server.listen(&server_addresses);

        Server {
            server,
            clients: Vec::new(),
            lobbies: Vec::new(),
            games: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        let room_key = self.server.make_room().key();
        let world = World::default();

        loop {
            for event in self.server.receive() {
                match event {
                    Ok(Event::Authorization(user_key, _)) => {
                        self.server.accept_connection(&user_key);
                    }
                    Ok(Event::Connection(user_key)) => {
                        log::info!(
                            "Naia Server connected to: {}",
                            self.server.user(&user_key).address()
                        );
                        // self.server.accept_connection(&user_key);
                        self.server.room_mut(&room_key).add_user(&user_key);
                        self.clients.push(Client::new(user_key));

                        self.send_lobbies_info(&user_key);
                    }
                    Ok(Event::Disconnection(user_key, _)) => {
                        log::info!("DISCONNECT :(");
                        self.clients.retain(|c| c.user_key != user_key);
                    }
                    Ok(Event::Message(user_key, _, protocol)) => {
                        if let Some(game_protocol) = self.execute_command(user_key, protocol) {
                            for game in &mut self.games {
                                if game.clients.iter().any(|c| c.user_key == user_key)
                                    && game.execute_commands(
                                        &mut self.server,
                                        user_key,
                                        game_protocol.clone(),
                                    )
                                {
                                    game.pass_turn(&mut self.server, false);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            self.games.retain_mut(|game| {
                if let Some(winner_uuid) = game.check_if_game_end() {
                    game.game_end(&mut self.server, winner_uuid);
                    self.clients.append(&mut game.clients);
                    false
                } else {
                    true
                }
            });

            self.server.send_all_updates(world.proxy());
        }
    }

    fn execute_command(&mut self, user_key: UserKey, protocol: Protocol) -> Option<Protocol> {
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
                            Channels::Uno,
                            &protocol::Error::new("Maximum lobby amount".to_owned()),
                        );
                    }
                }
                Protocol::JoinLobby(lobby) => {
                    if client.in_lobby.is_some() {
                        self.server.send_message(
                            &client.user_key,
                            Channels::Uno,
                            &protocol::Error::new("You're already in a lobby".to_owned()),
                        );
                        return None;
                    }

                    if let Some(lobby) = self.lobbies.iter_mut().find(|l| l.id == *lobby.lobby_id) {
                        if lobby.players.len() >= MAX_LOBBY_PLAYERS {
                            self.server.send_message(
                                &client.user_key,
                                Channels::Uno,
                                &protocol::Error::new("The lobby is full".to_owned()),
                            );
                            return None;
                        }

                        player_joined = Some((lobby.id, client.player.clone()));
                        lobby.players.push(client.player.clone());

                        self.server.send_message(
                            &client.user_key,
                            Channels::Uno,
                            &protocol::JoinLobby::new(lobby.id, lobby.players.clone()),
                        );
                        client.in_lobby = Some(lobby.id);
                    } else {
                        self.server.send_message(
                            &client.user_key,
                            Channels::Uno,
                            &protocol::Error::new("Lobby doesn't exist".to_owned()),
                        );
                    }
                }
                Protocol::LeaveLobby(lobby) => {
                    if client.in_lobby.is_none() {
                        self.server.send_message(
                            &client.user_key,
                            Channels::Uno,
                            &protocol::Error::new("You're not in a lobby".to_owned()),
                        );
                        return None;
                    }

                    if let Some(lobby) = self.lobbies.iter_mut().find(|l| l.id == *lobby.lobby_id) {
                        player_left = Some((client.id, lobby.id));

                        lobby.players.retain(|player| player.id != client.id);

                        self.server.send_message(
                            &client.user_key,
                            Channels::Uno,
                            &protocol::LeaveLobby::new(lobby.id),
                        );
                        client.in_lobby = None;
                    }
                }
                Protocol::Username(player) => {
                    client.player.username = (*player.username).clone();
                }
                protocol => {
                    return Some(protocol);
                }
            }
        } else {
            return Some(protocol);
        }

        if let Some((player_id, lobby_id)) = player_left {
            for client in self.clients.iter_mut() {
                self.server.send_message(
                    &client.user_key,
                    Channels::Uno,
                    &protocol::PlayerLeftLobby::new(lobby_id, player_id),
                )
            }
        } else if let Some((lobby_id, player)) = player_joined {
            for client in self.clients.iter_mut() {
                self.server.send_message(
                    &client.user_key,
                    Channels::Uno,
                    &protocol::PlayerJoinedLobby::new(lobby_id, player.clone()),
                );
            }
        } else if let Some(id) = lobby_created {
            for client in self.clients.iter_mut() {
                self.server.send_message(
                    &client.user_key,
                    Channels::Uno,
                    &protocol::LobbyCreated::new(id),
                );
            }
        } else if let Some(lobby_id) = start_game {
            // Get clients in lobby
            let mut game_clients = self
                .clients
                .drain_filter(|client| client.in_lobby == Some(lobby_id))
                .collect::<Vec<Client>>();

            // Tell clients the lobby the game was started in is gone
            for client in self.clients.iter_mut() {
                self.server.send_message(
                    &client.user_key,
                    Channels::Uno,
                    &protocol::LobbyDestroyed::new(lobby_id),
                );
            }

            self.lobbies.retain(|l| l.id != lobby_id);

            for client in game_clients.iter_mut() {
                client.in_lobby = None;
                self.server.send_message(
                    &client.user_key,
                    Channels::Uno,
                    &protocol::StartGame::new(),
                );
            }

            self.games.push(Game::new(game_clients, &mut self.server));
        }

        None
    }

    fn send_lobbies_info(&mut self, user_key: &UserKey) {
        for lobby in &self.lobbies {
            self.server
                .send_message(&user_key, Channels::Uno, &protocol::LobbyInfo::new(lobby));
        }
    }
}
