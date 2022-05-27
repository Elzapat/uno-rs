use crate::client::Client;
use anyhow::Result;
use log::{error, info};
use naia_server::UserKey;
use std::sync::mpsc::{Receiver, Sender};
use uno::{
    card::{Card, Color, Value},
    network::{protocol, Channels, Protocol},
    player::PlayerState,
    Deck,
};
use uuid::Uuid;

pub struct Game {
    packets_receiver: Receiver<(UserKey, Protocol)>,
    packets_sender: Sender<(UserKey, Protocol)>,
    clients: Vec<Client>,
    deck: Deck,
    discard: Deck,
    turn_index: usize,
    reverse_turn: bool,
    current_color: Color,
}

impl Game {
    pub fn new(
        clients: Vec<Client>,
        sender: Sender<(UserKey, Protocol)>,
        receiver: Receiver<(UserKey, Protocol)>,
    ) -> Game {
        Game {
            packets_sender: sender,
            packets_receiver: receiver,
            deck: Deck::full(),
            discard: Deck::empty(),
            clients,
            turn_index: 0,
            reverse_turn: false,
            current_color: Color::Black,
        }
    }

    pub fn run(mut self) -> Result<Vec<Client>> {
        self.send_player_ids()?;
        self.deck.shuffle();
        self.give_first_cards()?;
        self.draw_first_card()?;

        loop {
            if self.clients.is_empty() {
                return Ok(self.clients);
            }

            if let Some(winner_uuid) = self.check_if_game_end() {
                self.game_end(winner_uuid)?;
                return Ok(self.clients);
            }

            self.game_turn()?;
        }
    }

    fn game_turn(&mut self) -> Result<()> {
        self.pass_turn(false)?;

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

        Ok(())
    }

    fn execute_commands(&mut self) -> Result<bool> {
        let mut card_played = None;
        let mut wild_four_played = false;
        let mut stop_uno = false;
        let mut stop_counter_uno = false;
        let mut draw_card = false;

        for client in self.clients.iter_mut() {
            for packet in client.incoming_packets.drain(..) {
                match packet.command {
                    Command::PlayCard => {
                        if client.player.state == PlayerState::PlayingCard {
                            let card: Card = packet.args.as_slice().into();
                            let valid = card
                                .can_be_played(*self.discard.top().unwrap(), self.current_color);

                            write_socket(&mut client.socket, Command::CardValidation, valid as u8)?;

                            if valid {
                                card_played = Some(card);
                            }
                        } else {
                            write_socket(&mut client.socket, Command::CardValidation, 0)?;
                        }
                    }
                    Command::ColorChosen => {
                        if client.player.state == PlayerState::ChoosingColorWild
                            || client.player.state == PlayerState::ChoosingColorWildFour
                            || client.player.state == PlayerState::ChoosingColorWildUno
                            || client.player.state == PlayerState::ChoosingColorWildFourUno
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
                    Command::DrawCard => {
                        if client.player.state == PlayerState::DrawingCard {
                            draw_card = true;
                        }
                    }
                    Command::Uno => stop_uno = true,
                    Command::CounterUno => stop_counter_uno = true,
                    _ => return Ok(false),
                }
            }
        }

        if let Some(card) = card_played {
            let card_idx = if let Some(card_idx) = self.clients[self.turn_index]
                .player
                .hand
                .iter()
                .position(|&c| c == card)
            {
                card_idx
            } else {
                return Ok(false);
            };

            self.clients[self.turn_index].player.hand.remove(card_idx);
            self.discard.add(card);
            self.current_color = card.color;

            let mut in_uno = false;
            if self.clients[self.turn_index].player.hand.len() == 1 {
                if self.clients[self.turn_index].player.state == PlayerState::ChoosingColorWild {
                    self.clients[self.turn_index].player.state = PlayerState::ChoosingColorWildUno;
                } else if self.clients[self.turn_index].player.state
                    == PlayerState::ChoosingColorWildFour
                {
                    self.clients[self.turn_index].player.state =
                        PlayerState::ChoosingColorWildFourUno;
                } else {
                    self.clients[self.turn_index].player.state = PlayerState::Uno;
                }
                in_uno = true;
            }
            for client in self.clients.iter_mut() {
                if client.player.state == PlayerState::WaitingToPlay {
                    write_socket(&mut client.socket, Command::CardPlayed, card)?;
                    if in_uno {
                        write_socket(&mut client.socket, Command::CounterUno, vec![])?;
                    }
                } else if in_uno {
                    write_socket(&mut client.socket, Command::Uno, vec![])?;
                }
            }

            let pass_turn = match card.value {
                Value::Reverse => {
                    self.reverse_turn = !self.reverse_turn;

                    if self.clients.len() == 2 {
                        self.pass_turn(true)?;
                    }

                    true
                }
                Value::DrawTwo => {
                    self.pass_turn(true)?;
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
                    self.pass_turn(true)?;
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

            return Ok(if in_uno { false } else { pass_turn });
        } else if wild_four_played {
            self.pass_turn(true)?;

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
        } else if stop_uno {
            for client in self.clients.iter_mut() {
                write_socket(&mut client.socket, Command::StopUno, vec![])?;
                write_socket(&mut client.socket, Command::StopCounterUno, vec![])?;
            }

            return Ok(true);
        } else if stop_counter_uno {
            for client in self.clients.iter_mut() {
                write_socket(&mut client.socket, Command::StopUno, vec![])?;
                write_socket(&mut client.socket, Command::StopCounterUno, vec![])?;
            }

            for _ in 0..2 {
                let card = self.draw_card();
                self.clients[self.turn_index].player.hand.push(card);
                write_socket(
                    &mut self.clients[self.turn_index].socket,
                    Command::DrawCard,
                    card,
                )?;
            }

            return Ok(true);
        } else if draw_card {
            let card = self.draw_card();
            self.clients[self.turn_index].player.hand.push(card);

            write_socket(
                &mut self.clients[self.turn_index].socket,
                Command::DrawCard,
                card,
            )?;

            if self.clients[self.turn_index]
                .player
                .can_play(*self.discard.top().unwrap(), self.current_color)
            {
                self.clients[self.turn_index].player.state = PlayerState::PlayingCard;
                return Ok(false);
            } else {
                self.clients[self.turn_index].player.state = PlayerState::WaitingToPlay;
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn pass_turn(&mut self, passing_turn: bool) -> Result<()> {
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
            client.player.state = PlayerState::WaitingToPlay;

            write_socket(&mut client.socket, Command::PassTurn, &id.as_bytes()[..])?;
            write_socket(
                &mut client.socket,
                Command::HandSize,
                [&[nb_cards as u8], &id.as_bytes()[..]].concat(),
            )?;
            write_socket(
                &mut client.socket,
                Command::CurrentColor,
                self.current_color as u8,
            )?;
        }

        if !passing_turn {
            let playing_client = &mut self.clients[self.turn_index];
            if !playing_client
                .player
                .can_play(*self.discard.top().unwrap(), self.current_color)
            {
                playing_client.player.state = PlayerState::DrawingCard;
                write_socket(&mut playing_client.socket, Command::HaveToDrawCard, vec![])?;
            } else {
                playing_client.player.state = PlayerState::PlayingCard;
            }
        }

        Ok(())
    }

    fn read_sockets(&mut self) -> Result<()> {
        let mut to_remove = None;

        for (i, client) in self.clients.iter_mut().enumerate() {
            client
                .incoming_packets
                .push(match read_socket(&mut client.socket) {
                    Ok(packets) => {
                        info!("{:?}", packets);
                        packets
                    }
                    Err(e) => {
                        if let Some(tungstenite::Error::ConnectionClosed) =
                            e.downcast_ref::<tungstenite::Error>()
                        {
                            to_remove = Some(i);
                            continue;
                        } else {
                            return Err(e);
                        }
                    }
                });
        }

        if let Some(i) = to_remove {
            self.clients.remove(i);

            todo!();
        }

        Ok(())
    }

    fn send_player_ids(&mut self) -> Result<()> {
        for client in self.clients.iter_mut() {
            write_socket(
                &mut client.socket,
                Command::YourPlayerId,
                &client.id.as_bytes()[..],
            )?;
        }

        Ok(())
    }

    fn draw_first_card(&mut self) -> Result<()> {
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

    fn give_first_cards(&mut self) -> Result<()> {
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

    fn check_if_game_end(&self) -> Option<Uuid> {
        for client in self.clients.iter() {
            if client.player.hand.is_empty() {
                return Some(client.id);
            }
        }

        None
    }

    fn game_end(&mut self, winner_uuid: Uuid) -> Result<()> {
        self.compute_scores()?;

        for client in self.clients.iter_mut() {
            write_socket(
                &mut client.socket,
                Command::GameEnd,
                &winner_uuid.as_bytes()[..],
            )?;
        }

        Ok(())
    }

    fn compute_scores(&mut self) -> Result<()> {
        let mut player_scores = vec![];

        for client in self.clients.iter_mut() {
            for card in client.player.hand.iter() {
                client.player.score += match card.value {
                    Value::Wild | Value::WildFour => 50,
                    Value::Reverse | Value::DrawTwo | Value::Skip => 20,
                    Value::Zero => 0,
                    value => value as u32,
                }
            }

            player_scores.push((client.id, client.player.score));
        }

        for (client, (uuid, score)) in self.clients.iter_mut().zip(player_scores.iter()) {
            write_socket(
                &mut client.socket,
                Command::PlayerScore,
                [&uuid.as_bytes()[..], score.to_string().as_bytes()].concat(),
            )?;
        }

        Ok(())
    }
}
