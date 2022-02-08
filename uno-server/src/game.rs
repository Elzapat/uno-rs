use crate::client::Client;
use log::{error, info};
use uno::{
    packet::{read_socket, write_socket, Command},
    prelude::*,
};

enum GameState {
    Playing,
    EndLobby,
}

pub struct Game {
    state: GameState,
    clients: Vec<Client>,
    deck: Deck,
    discard: Deck,
}

impl Game {
    pub fn new(clients: Vec<Client>) -> Game {
        Game {
            deck: Deck::full(),
            discard: Deck::empty(),
            state: GameState::Playing,
            clients,
        }
    }

    pub fn run(&mut self) {
        let mut turn_index = 0;

        self.deck.shuffle();

        // Deal the initial seven cards to the players
        const INITIAL_CARDS: usize = 7;
        for client in self.clients.iter_mut() {
            for _ in 0..INITIAL_CARDS {
                let card: [u8; 2] = self.deck.draw().unwrap().into();
                if let Err(e) = write_socket(&mut client.socket, Command::DrawCard, &card[..]) {
                    error!("{}", e);
                }
            }
        }

        loop {
            if self.clients.len() == 0 {
                return;
            }

            self.pass_turn(&mut turn_index);

            let mut pass_turn = false;
            while !pass_turn {
                if let Err(e) = self.read_sockets() {
                    error!("{}", e);
                    continue;
                }

                match self.execute_commands() {
                    Ok(pass) => pass_turn = pass,
                    Err(e) => {
                        error!("{}", e);
                        continue;
                    }
                }
            }
        }
    }

    fn execute_commands(&mut self) -> Result<bool> {
        Ok(true)
    }

    fn pass_turn(&mut self, turn_index: &mut usize) {
        if *turn_index >= self.clients.len() {
            for client in self.clients.iter_mut() {
                client.player.is_playing = false;
            }
        } else {
            self.clients[*turn_index].player.is_playing = false;
        }

        *turn_index = (*turn_index + 1) % self.clients.len();
        self.clients[*turn_index].player.is_playing = true;
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

            todo!();
        }

        Ok(())
    }
}
