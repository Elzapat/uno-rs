use crate::client::Client;
use log::{error, info};
use uno::{
    card::{Color, Value},
    packet::{read_socket, write_socket, Command},
    player::PlayerState,
    prelude::*,
};

// enum GameState {
//     Playing,
//     EndLobby,
// }

pub struct Game {
    // state: GameState,
    clients: Vec<Client>,
    deck: Deck,
    discard: Deck,
    turn_index: usize,
    reverse_turn: bool,
    current_color: Color,
}

impl Game {
    pub fn new(clients: Vec<Client>) -> Game {
        Game {
            deck: Deck::full(),
            discard: Deck::empty(),
            // state: GameState::Playing,
            clients,
            turn_index: 0,
            reverse_turn: false,
            current_color: Color::Black,
        }
    }

    pub fn run(&mut self) -> Result {
        self.send_player_ids()?;
        self.deck.shuffle();
        self.give_first_cards()?;
        self.draw_first_card()?;

        loop {
            if self.clients.is_empty() {
                return Ok(());
            }

            self.pass_turn()?;

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
        let mut card_played = None;
        let mut wild_four_played = false;

        for client in self.clients.iter_mut() {
            for packet in client.incoming_packets.drain(..) {
                match packet.command {
                    Command::PlayCard => {
                        if client.player.state == PlayerState::PlayingCard
                            && client.player.is_playing
                        {
                            let card: Card = packet.args.as_slice().into();
                            let valid = {
                                let top_card = self.discard.top().unwrap();

                                self.current_color == card.color
                                    || top_card.value == card.value
                                    || card.color == Color::Black
                            };

                            write_socket(&mut client.socket, Command::CardValidation, valid as u8)?;

                            if valid {
                                card_played = Some(card);
                            }
                        }
                    }
                    Command::ColorChosen => {
                        if client.player.state == PlayerState::ChoosingColorWild
                            || client.player.state == PlayerState::ChoosingColorWildFour
                        {
                            let color: Color = (*packet.args.get(0).unwrap()).into();
                            self.current_color = color;

                            if client.player.state == PlayerState::ChoosingColorWildFour {
                                wild_four_played = true;
                            } else {
                                return Ok(true);
                            }
                        }
                    }
                    _ => return Ok(false),
                }
            }
        }

        if let Some(card) = card_played {
            let card_idx = self.clients[self.turn_index]
                .player
                .hand
                .iter()
                .position(|&c| c == card)
                .unwrap();
            self.clients[self.turn_index].player.hand.remove(card_idx);
            self.discard.add(card);
            self.current_color = card.color;

            for client in self.clients.iter_mut() {
                if client.player.state == PlayerState::WaitingToPlay {
                    write_socket(&mut client.socket, Command::CardPlayed, card)?;
                }
            }

            if self.clients[self.turn_index].player.hand.len() == 1 {
                self.clients[self.turn_index].player.state = PlayerState::Uno;
            }

            let pass_turn = match card.value {
                Value::Reverse => {
                    self.reverse_turn = !self.reverse_turn;

                    if self.clients.len() == 2 {
                        self.pass_turn()?;
                    }

                    true
                }
                Value::DrawTwo => {
                    self.pass_turn()?;
                    for _ in 0..2 {
                        let card = self.draw_card();
                        self.clients[self.turn_index].player.hand.push(card);
                        write_socket(
                            &mut self.clients[self.turn_index].socket,
                            Command::DrawCard,
                            card,
                        )?;
                    }

                    true
                }
                Value::Skip => {
                    self.pass_turn()?;
                    true
                }
                Value::Wild => {
                    self.clients[self.turn_index].player.state =
                        if self.clients[self.turn_index].player.hand.len() == 1 {
                            PlayerState::ChoosingColorWildUno
                        } else {
                            PlayerState::ChoosingColorWild
                        };
                    false
                }
                Value::WildFour => {
                    self.clients[self.turn_index].player.state =
                        if self.clients[self.turn_index].player.hand.len() == 1 {
                            PlayerState::ChoosingColorWildFourUno
                        } else {
                            PlayerState::ChoosingColorWildFour
                        };
                    false
                }
                _ => true,
            };

            return Ok(pass_turn);
        } else if wild_four_played {
            self.pass_turn()?;

            for _ in 0..4 {
                let card = self.draw_card();
                self.clients[self.turn_index].player.hand.push(card);
                write_socket(
                    &mut self.clients[self.turn_index].socket,
                    Command::DrawCard,
                    card,
                )?;
            }

            return Ok(true);
        }

        Ok(false)
    }

    fn pass_turn(&mut self) -> Result {
        if self.reverse_turn {
            if self.turn_index == 0 {
                self.turn_index = self.clients.len() - 1;
            } else {
                self.turn_index -= 1;
            }
        } else {
            self.turn_index = (self.turn_index + 1) % self.clients.len();
        }

        let id = self.clients[self.turn_index].id;
        let nb_cards = self.clients[self.turn_index].player.hand.len();
        for client in self.clients.iter_mut() {
            client.player.is_playing = false;
            client.player.state = PlayerState::WaitingToPlay;

            write_socket(&mut client.socket, Command::PassTurn, &id.as_bytes()[..])?;
            write_socket(
                &mut client.socket,
                Command::HandSize,
                [&[nb_cards as u8], &id.as_bytes()[..]].concat(),
            )?;
        }

        self.clients[self.turn_index].player.is_playing = true;
        self.clients[self.turn_index].player.state = PlayerState::PlayingCard;

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

            todo!();
        }

        Ok(())
    }

    fn send_player_ids(&mut self) -> Result {
        for client in self.clients.iter_mut() {
            write_socket(
                &mut client.socket,
                Command::YourPlayerId,
                &client.id.as_bytes()[..],
            )?;
        }

        Ok(())
    }

    fn draw_first_card(&mut self) -> Result {
        let mut first_card = self.draw_card();
        while first_card.color == Color::Black
            || first_card.value == Value::Skip
            || first_card.value == Value::Reverse
            || first_card.value == Value::DrawTwo
        {
            self.discard.add(first_card);
            first_card = self.draw_card();
        }
        self.discard.add(first_card);
        self.current_color = first_card.color;

        for client in self.clients.iter_mut() {
            write_socket(&mut client.socket, Command::CardPlayed, first_card)?;
        }

        Ok(())
    }

    fn give_first_cards(&mut self) -> Result {
        // Deal the initial seven cards to the players
        const INITIAL_CARDS: usize = 7;
        for client in self.clients.iter_mut() {
            for _ in 0..INITIAL_CARDS {
                let card = self.deck.draw().unwrap();
                client.player.hand.push(card);
                write_socket(&mut client.socket, Command::DrawCard, card)?;
            }
        }

        Ok(())
    }

    fn draw_card(&mut self) -> Card {
        if self.deck.is_empty() {
            let top_card = self.discard.draw().unwrap();
            self.deck = self.discard.clone();
            self.deck.shuffle();

            self.discard = Deck::empty();
            self.discard.add(top_card);
        }

        self.deck.draw().unwrap()
    }
}
