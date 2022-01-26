use crate::client::Client;

enum GameState {
    Playing,
    EndLobby,
}

pub struct Game {
    state: GameState,
    clients: Vec<Client>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            state: GameState::Playing,
            clients: Vec::new(),
        }
    }

    pub fn add_client(&mut self, client: Client) {
        self.clients.push(client);
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
