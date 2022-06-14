use crate::{lobbies::InLobby, server::UserKeyComponent};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use naia_bevy_server::{Server, UserKey};
use rand::Rng;
use std::{collections::HashMap, default::Default};
use uno::{
    card::{Card, Color, Value},
    lobby::LobbyId,
    network::{
        protocol::{DrawCard, StartGame},
        Channels, Protocol,
    },
    Deck, Player,
};

#[derive(Component, Deref, DerefMut)]
pub struct InGame(pub LobbyId);

pub struct PassTurnEvent {
    skipping_turn: bool,
    game_id: LobbyId,
}
#[derive(Deref, DerefMut)]
pub struct StartGameEvent(pub LobbyId);
#[derive(Deref, DerefMut)]
pub struct CardPlayedEvent(pub Card);
pub struct DrawCardEvent {
    user_key: UserKey,
    game_id: LobbyId,
}

#[derive(Debug, Clone)]
pub struct Game {
    settings: GameSettings,
    players: Vec<Entity>,
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

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct Games(pub HashMap<LobbyId, Game>);

pub fn setup_game(
    mut commands: Commands,
    mut server: Server<Protocol, Channels>,
    mut start_game_event: EventReader<StartGameEvent>,
    mut games: ResMut<Games>,
    mut draw_card_event: EventWriter<DrawCardEvent>,
    mut pass_turn_event: EventWriter<PassTurnEvent>,
    players_query: Query<(Entity, &InLobby, &UserKeyComponent)>,
) {
    for StartGameEvent(lobby_id) in start_game_event.iter() {
        // Create the new game and shuffle the deck
        let mut game = Game::default();
        game.deck.shuffle();

        for (entity, InLobby(player_lobby_id), user_key) in players_query.iter() {
            if player_lobby_id == lobby_id {
                println!("found a player");
                commands
                    .entity(entity)
                    .remove::<InLobby>()
                    .insert(InGame(*lobby_id));

                server.send_message(user_key, Channels::Uno, &StartGame::new());

                // Add the player to the game
                game.players.push(entity);

                // Send the intial card to the player
                for _ in 0..game.settings.initial_cards {
                    draw_card_event.send(DrawCardEvent {
                        user_key: **user_key,
                        game_id: *lobby_id,
                    });
                }
            }
        }

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

        // Randomize the first player
        game.turn_index = rand::thread_rng().gen_range(0..game.players.len());
        pass_turn_event.send(PassTurnEvent {
            skipping_turn: false,
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
                server.send_message(user_key, Channels::Uno, &DrawCard::new(card));
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
) {
    for PassTurnEvent {
        skipping_turn,
        game_id,
    } in pass_turn_events.iter()
    {
        let game = match games.get_mut(game_id) {
            Some(g) => g,
            None => continue,
        };
    }
}
