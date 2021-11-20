use std::net::TcpStream;
use uno::prelude::*;
use crate::client::Client;

enum GameState {
    Lobby,
    Playing,
    Finished,
}

pub struct Game {
    state: GameState,
    clients: Vec<Client>
}

impl Game {
    pub fn new() -> Game {
        Game {
            state: GameState::Lobby,
            clients: Vec::new(),
        }
    }

    pub fn add_client(&mut self, client: Client) {
        self.clients.push(client);
    }
}
