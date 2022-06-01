use crate::{client::Client, server::NaiaServer};
use naia_server::UserKey;
use uno::{
    card::{Card, Color, Value},
    network::{protocol, Channels, Protocol},
    player::PlayerState,
    Deck,
};
use uuid::Uuid;

pub struct Game {
    pub clients: Vec<Client>,
    deck: Deck,
    discard: Deck,
    turn_index: usize,
    reverse_turn: bool,
    current_color: Color,
}

impl Game {
    pub fn new(clients: Vec<Client>, server: &mut NaiaServer) -> Game {
        let mut game = Game {
            deck: Deck::full(),
            discard: Deck::empty(),
            clients,
            turn_index: 0,
            reverse_turn: false,
            current_color: Color::Black,
        };

        game.send_player_ids(server);
        game.deck.shuffle();
        game.give_first_cards(server);
        game.draw_first_card(server);
        game.pass_turn(server, false);

        game
    }

    /*
    pub fn run(mut self) -> Result<Vec<Client>> {
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
    */

    /*
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
    */

    pub fn execute_commands(
        &mut self,
        server: &mut NaiaServer,
        user_key: UserKey,
        protocol: Protocol,
    ) -> bool {
        let mut card_played = None;
        let mut wild_four_played = None;
        let mut uno = None;
        let mut counter_uno = false;
        let mut draw_card = false;

        if let Some(client) = self.clients.iter_mut().find(|c| c.user_key == user_key) {
            match protocol {
                Protocol::PlayCard(card) => {
                    if client.player.state == PlayerState::PlayingCard {
                        let card: Card = (*card.color, *card.value).into();
                        let valid = card
                            .can_be_played(*self.discard.top().unwrap(), self.current_color)
                            && client.player.hand.contains(&card);

                        server.send_message(
                            &user_key,
                            Channels::Uno,
                            &protocol::CardValidation::new(valid),
                        );

                        if valid {
                            card_played = Some(card);
                        }
                    } else {
                        server.send_message(
                            &user_key,
                            Channels::Uno,
                            &protocol::CardValidation::new(false),
                        );
                    }
                }
                Protocol::ColorChosen(color) => match client.player.state {
                    PlayerState::ChoosingColorWildUno(ref mut actions_done) => {
                        self.current_color = (*color.color).into();
                        actions_done[0] = true;
                        return actions_done.iter().all(|&a| a);
                    }
                    PlayerState::ChoosingColorWildFourUno(ref mut actions_done) => {
                        self.current_color = (*color.color).into();
                        actions_done[0] = true;
                        wild_four_played = Some(actions_done.iter().all(|&a| a))
                    }
                    PlayerState::ChoosingColorWild => {
                        self.current_color = (*color.color).into();
                        return true;
                    }
                    PlayerState::ChoosingColorWildFour => {
                        self.current_color = (*color.color).into();
                        wild_four_played = Some(true);
                    }
                    _ => {}
                },
                Protocol::DrawCard(_) => {
                    if client.player.state == PlayerState::DrawingCard {
                        draw_card = true;
                    }
                }
                Protocol::Uno(_) => {
                    if let PlayerState::ChoosingColorWildUno(ref mut actions_done) =
                        client.player.state
                    {
                        actions_done[1] = true;
                        uno = Some(actions_done.iter().all(|&a| a))
                    } else if let PlayerState::ChoosingColorWildFourUno(ref mut actions_done) =
                        client.player.state
                    {
                        actions_done[1] = true;
                        if actions_done.iter().all(|&a| a) {
                            self.pass_turn(server, true);
                            uno = Some(true);
                        } else {
                            uno = Some(false);
                        }
                    } else {
                        uno = Some(true);
                    }
                }
                Protocol::CounterUno(_) => counter_uno = true,
                _ => return false,
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
                return false;
            };

            self.clients[self.turn_index].player.hand.remove(card_idx);
            self.discard.add(card);
            self.current_color = card.color;

            let mut in_uno = false;
            if self.clients[self.turn_index].player.hand.len() == 1 {
                self.clients[self.turn_index].player.state = PlayerState::Uno;
                in_uno = true;
            }

            for client in self.clients.iter_mut() {
                if client.player.state == PlayerState::WaitingToPlay {
                    server.send_message(
                        &client.user_key,
                        Channels::Uno,
                        &protocol::CardPlayed::new(card),
                    );
                    if in_uno {
                        server.send_message(
                            &client.user_key,
                            Channels::Uno,
                            &protocol::CounterUno::new(),
                        );
                    }
                } else if in_uno {
                    server.send_message(&client.user_key, Channels::Uno, &protocol::Uno::new());
                }
            }

            let pass_turn = match card.value {
                Value::Reverse => {
                    self.reverse_turn = !self.reverse_turn;

                    if self.clients.len() == 2 {
                        self.pass_turn(server, true);
                    }

                    true
                }
                Value::DrawTwo => {
                    self.pass_turn(server, true);
                    for _ in 0..2 {
                        let card = self.draw_card();
                        self.clients[self.turn_index].player.hand.push(card);
                        server.send_message(
                            &self.clients[self.turn_index].user_key,
                            Channels::Uno,
                            &protocol::DrawCard::new(card),
                        );
                    }

                    true
                }
                Value::Skip => {
                    self.pass_turn(server, true);
                    true
                }
                Value::Wild => {
                    self.clients[self.turn_index].player.state =
                        if self.clients[self.turn_index].player.hand.len() == 1 {
                            PlayerState::ChoosingColorWildUno([false; 2])
                        } else {
                            PlayerState::ChoosingColorWild
                        };

                    false
                }
                Value::WildFour => {
                    self.clients[self.turn_index].player.state =
                        if self.clients[self.turn_index].player.hand.len() == 1 {
                            PlayerState::ChoosingColorWildFourUno([false; 2])
                        } else {
                            PlayerState::ChoosingColorWildFour
                        };

                    false
                }
                _ => true,
            };

            return if in_uno { false } else { pass_turn };
        } else if let Some(skip_turn) = wild_four_played {
            let next_player_index = self.next_player_index();

            for _ in 0..4 {
                let card = self.draw_card();
                self.clients[next_player_index].player.hand.push(card);
                server.send_message(
                    &self.clients[next_player_index].user_key,
                    Channels::Uno,
                    &protocol::DrawCard::new(card),
                );
            }

            return if skip_turn {
                self.pass_turn(server, true);
                true
            } else {
                false
            };
        } else if let Some(pass_turn) = uno {
            for client in self.clients.iter_mut() {
                server.send_message(&client.user_key, Channels::Uno, &protocol::StopUno::new());
                server.send_message(
                    &client.user_key,
                    Channels::Uno,
                    &protocol::StopCounterUno::new(),
                );
            }

            return pass_turn;
        } else if counter_uno {
            for client in self.clients.iter_mut() {
                server.send_message(&client.user_key, Channels::Uno, &protocol::StopUno::new());
                server.send_message(
                    &client.user_key,
                    Channels::Uno,
                    &protocol::StopCounterUno::new(),
                );
            }

            for _ in 0..2 {
                let card = self.draw_card();
                self.clients[self.turn_index].player.hand.push(card);
                server.send_message(
                    &self.clients[self.turn_index].user_key,
                    Channels::Uno,
                    &protocol::DrawCard::new(card),
                );
            }

            let pass_turn = if let PlayerState::ChoosingColorWildUno(ref mut actions_done) =
                self.clients[self.turn_index].player.state
            {
                actions_done[1] = true;
                actions_done.iter().all(|&a| a)
            } else if let PlayerState::ChoosingColorWildFourUno(ref mut actions_done) =
                self.clients[self.turn_index].player.state
            {
                actions_done[1] = true;
                if actions_done.iter().all(|&a| a) {
                    self.pass_turn(server, true);
                    true
                } else {
                    false
                }
            } else {
                false
            };

            return pass_turn;
        } else if draw_card {
            let card = self.draw_card();
            self.clients[self.turn_index].player.hand.push(card);

            server.send_message(
                &self.clients[self.turn_index].user_key,
                Channels::Uno,
                &protocol::DrawCard::new(card),
            );

            if self.clients[self.turn_index]
                .player
                .can_play(*self.discard.top().unwrap(), self.current_color)
            {
                self.clients[self.turn_index].player.state = PlayerState::PlayingCard;
                return false;
            } else {
                self.clients[self.turn_index].player.state = PlayerState::WaitingToPlay;
                return true;
            }
        }

        false
    }

    pub fn pass_turn(&mut self, server: &mut NaiaServer, skipping_a_turn: bool) {
        self.turn_index = self.next_player_index();

        let id = self.clients[self.turn_index].id;
        let nb_cards = self.clients[self.turn_index].player.hand.len();
        for client in self.clients.iter_mut() {
            client.player.state = PlayerState::WaitingToPlay;

            server.send_message(
                &client.user_key,
                Channels::Uno,
                &protocol::PassTurn::new(id),
            );
            server.send_message(
                &client.user_key,
                Channels::Uno,
                &protocol::HandSize::new(nb_cards, id),
            );
            server.send_message(
                &client.user_key,
                Channels::Uno,
                &protocol::CurrentColor::new(self.current_color),
            );
        }

        if !skipping_a_turn {
            let playing_client = &mut self.clients[self.turn_index];
            if !playing_client
                .player
                .can_play(*self.discard.top().unwrap(), self.current_color)
            {
                playing_client.player.state = PlayerState::DrawingCard;
                server.send_message(
                    &playing_client.user_key,
                    Channels::Uno,
                    &protocol::HaveToDrawCard::new(),
                );
            } else {
                playing_client.player.state = PlayerState::PlayingCard;
            }
        }
    }

    fn next_player_index(&self) -> usize {
        if self.reverse_turn {
            if self.turn_index == 0 {
                self.clients.len() - 1
            } else {
                self.turn_index - 1
            }
        } else {
            (self.turn_index + 1) % self.clients.len()
        }
    }

    /*
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
    */

    fn send_player_ids(&mut self, server: &mut NaiaServer) {
        for client in self.clients.iter_mut() {
            server.send_message(
                &client.user_key,
                Channels::Uno,
                &protocol::YourPlayerId::new(client.id),
            );
        }
    }

    fn draw_first_card(&mut self, server: &mut NaiaServer) {
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
            server.send_message(
                &client.user_key,
                Channels::Uno,
                &protocol::CardPlayed::new(first_card),
            );
        }
    }

    fn give_first_cards(&mut self, server: &mut NaiaServer) {
        // Deal the initial seven cards to the players
        const INITIAL_CARDS: usize = 7;
        for client in self.clients.iter_mut() {
            for _ in 0..INITIAL_CARDS {
                let card = self.deck.draw().unwrap();
                client.player.hand.push(card);
                server.send_message(
                    &client.user_key,
                    Channels::Uno,
                    &protocol::DrawCard::new(card),
                );
            }
        }
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

    pub fn check_if_game_end(&self) -> Option<Uuid> {
        for client in self.clients.iter() {
            if client.player.hand.is_empty() {
                return Some(client.id);
            }
        }

        None
    }

    pub fn game_end(&mut self, server: &mut NaiaServer, winner_uuid: Uuid) {
        self.compute_scores(server);

        for client in self.clients.iter_mut() {
            server.send_message(
                &client.user_key,
                Channels::Uno,
                &protocol::GameEnd::new(winner_uuid),
            );

            client.player.hand.clear();
        }
    }

    fn compute_scores(&mut self, server: &mut NaiaServer) {
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

        for client in self.clients.iter() {
            for (uuid, score) in player_scores.iter() {
                server.send_message(
                    &client.user_key,
                    Channels::Uno,
                    &protocol::PlayerScore::new(*score, *uuid),
                );
            }
        }
    }
}
