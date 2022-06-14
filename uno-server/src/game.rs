use crate::{lobbies::InLobby, server::UserKeyComponent, Global};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use naia_bevy_server::{Server, UserKey};
use rand::Rng;
use std::{collections::HashMap, default::Default};
use uno::{
    card::{Card, Color, Value},
    lobby::LobbyId,
    network::{protocol::*, Channels, Protocol},
    player::PlayerState,
    Deck, Player,
};

#[derive(Component, Deref, DerefMut)]
pub struct InGame(pub LobbyId);

pub struct PassTurnEvent {
    skipping: bool,
    game_id: LobbyId,
}
#[derive(Deref, DerefMut)]
pub struct StartGameEvent(pub LobbyId);
pub struct DrawCardEvent {
    user_key: UserKey,
    game_id: LobbyId,
}
pub struct CardPlayedEvent {
    user_key: UserKey,
    game_id: LobbyId,
    card: Card,
}

#[derive(Clone)]
pub struct Game {
    settings: GameSettings,
    players: Vec<(UserKey, Entity)>,
    deck: Deck,
    discard: Deck,
    turn_index: usize,
    reverse_turn: bool,
    current_color: Color,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            settings: GameSettings::default(),
            players: Vec::new(),
            deck: Deck::full(),
            discard: Deck::empty(),
            turn_index: 0,
            reverse_turn: false,
            current_color: Color::Black,
        }
    }
}

impl Game {
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

    fn next_player_index(&self) -> usize {
        if self.reverse_turn {
            if self.turn_index == 0 {
                self.players.len() - 1
            } else {
                self.turn_index - 1
            }
        } else {
            (self.turn_index + 1) % self.players.len()
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameSettings {
    initial_cards: u32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { initial_cards: 7 }
    }
}

#[derive(Clone, Deref, DerefMut)]
pub struct Games(pub HashMap<LobbyId, Game>);

pub fn setup_game(
    mut commands: Commands,
    mut server: Server<Protocol, Channels>,
    mut start_game_event: EventReader<StartGameEvent>,
    mut games: ResMut<Games>,
    mut draw_card_event: EventWriter<DrawCardEvent>,
    mut pass_turn_event: EventWriter<PassTurnEvent>,
    global: Res<Global>,
    lobbies_query: Query<(Entity, &Lobby)>,
    players_query: Query<(Entity, &InLobby, &UserKeyComponent)>,
) {
    for StartGameEvent(lobby_id) in start_game_event.iter() {
        // Remove the lobby
        for (entity, lobby) in lobbies_query.iter() {
            if *lobby.id == *lobby_id {
                server.entity_mut(&entity).despawn();
            }
        }

        // Create the new game and shuffle the deck
        let mut game = Game::default();
        game.deck.shuffle();

        // Draw the first card
        let mut first_card = game.draw_card();
        while first_card.color == Color::Black
            || first_card.value == Value::Skip
            || first_card.value == Value::Reverse
            || first_card.value == Value::DrawTwo
        {
            game.discard.add(first_card);
            first_card = game.draw_card();
        }
        game.discard.add(first_card);
        game.current_color = first_card.color;
        server
            .spawn()
            .enter_room(&global.lobbies_room_key[lobby_id])
            .insert(CurrentColor::new(game.current_color));

        for (entity, InLobby(player_lobby_id), user_key) in players_query.iter() {
            if player_lobby_id == lobby_id {
                commands
                    .entity(entity)
                    .remove::<InLobby>()
                    .insert(InGame(*lobby_id));

                server.send_message(user_key, Channels::Uno, &StartGame::new());

                // Add the player to the game
                game.players
                    .push((**user_key, global.user_keys_entities[user_key]));

                // Send the intial card to the player
                for _ in 0..game.settings.initial_cards {
                    draw_card_event.send(DrawCardEvent {
                        user_key: **user_key,
                        game_id: *lobby_id,
                    });
                }

                // Send first card drawn
                server.send_message(user_key, Channels::Uno, &CardPlayed::new(first_card));
            }
        }

        // Randomize the first player
        game.turn_index = rand::thread_rng().gen_range(0..game.players.len());
        pass_turn_event.send(PassTurnEvent {
            skipping: false,
            game_id: *lobby_id,
        });

        games.insert(*lobby_id, game);
    }
}

pub fn draw_card(
    mut server: Server<Protocol, Channels>,
    mut games: ResMut<Games>,
    mut draw_card_events: EventReader<DrawCardEvent>,
    mut players_query: Query<(&mut Player, &UserKeyComponent, &InGame)>,
) {
    for DrawCardEvent { user_key, game_id } in draw_card_events.iter() {
        let game = match games.get_mut(game_id) {
            Some(g) => g,
            None => continue,
        };

        let card = game.draw_card();
        for (mut player, player_user_key, InGame(player_game_id)) in players_query.iter_mut() {
            if player_game_id == game_id && **player_user_key == *user_key {
                println!("sending draw card message");
                server.send_message(player_user_key, Channels::Uno, &DrawCard::new(card));
                player.hand.push(card);
                break;
            }
        }
    }
}

pub fn pass_turn(
    mut server: Server<Protocol, Channels>,
    mut games: ResMut<Games>,
    mut pass_turn_events: EventReader<PassTurnEvent>,
    mut players_query: Query<(&mut Player, &UserKeyComponent, &InGame)>,
) {
    for PassTurnEvent { skipping, game_id } in pass_turn_events.iter() {
        let game = match games.get_mut(game_id) {
            Some(g) => g,
            None => continue,
        };

        game.turn_index = game.next_player_index();

        for (mut player, user_key, InGame(id)) in players_query.iter_mut() {
            if **user_key == game.players[game.turn_index].0 && id == game_id {
                player.is_playing = true;
                player.state = PlayerState::WaitingToPlay;
            } else if id == game_id {
                player.is_playing = false;
            }
        }

        if !skipping {
            for (mut player, user_key, InGame(id)) in players_query.iter_mut() {
                if id == game_id && **user_key == game.players[game.turn_index].0 {
                    if !player.can_play(*game.discard.top().unwrap(), game.current_color) {
                        println!("cant play");
                        player.state = PlayerState::DrawingCard;
                        server.send_message(user_key, Channels::Uno, &HaveToDrawCard::new());
                    } else {
                        println!("can play");
                        player.state = PlayerState::PlayingCard;
                    }

                    break;
                }
            }
        }
    }
}
