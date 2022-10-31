use crate::{lobbies::InLobby, server::UserKeyComponent, Global};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_log::{error, info};
use naia_bevy_server::{Server, UserKey};
use rand::Rng;
use std::{collections::HashMap, default::Default};
use uno::{
    card::{Card, Color, Value},
    lobby::LobbyId,
    network::{
        protocol::{Player as NetworkPlayer, *},
        Channels, Protocol,
    },
    player::PlayerState,
    Deck, Player,
};

pub struct PassTurnEvent {
    pub skipping: bool,
    pub game_id: LobbyId,
}

pub struct StartGameEvent {
    pub lobby_id: LobbyId,
}

pub struct DrawCardEvent {
    pub user_key: UserKey,
    pub game_id: LobbyId,
    pub player_action: bool,
}

pub struct CardPlayedEvent {
    pub user_key: UserKey,
    pub game_id: LobbyId,
    pub card: Card,
}

pub struct PlayCardEvent {
    pub card: Card,
    pub game_id: LobbyId,
}

pub struct ColorChosenEvent {
    pub color: Color,
    pub game_id: LobbyId,
}

pub struct UnoEvent {
    pub user_key: UserKey,
    pub game_id: LobbyId,
}

pub struct CounterUnoEvent {
    pub user_key: UserKey,
    pub game_id: LobbyId,
}

pub struct GameEndEvent {
    pub game_id: LobbyId,
}

pub struct GameExitEvent {
    pub user_key: UserKey,
    pub game_id: LobbyId,
}

#[derive(Clone)]
pub struct Game {
    pub players: Vec<PlayerData>,
    pub current_color: Color,
    settings: GameSettings,
    deck: Deck,
    discard: Deck,
    turn_index: usize,
    reverse_turn: bool,
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

#[derive(Clone)]
pub struct PlayerData {
    pub player: Player,
    pub user_key: UserKey,
    pub server_entity: Entity,
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
    players_query: Query<(&Player, Entity, &InLobby, &UserKeyComponent)>,
    network_players_query: Query<(Entity, &NetworkPlayer)>,
) {
    for StartGameEvent { lobby_id } in start_game_event.iter() {
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

        for (player, entity, InLobby(player_lobby_id), user_key) in players_query.iter() {
            if player_lobby_id == lobby_id {
                commands.entity(entity).despawn();

                server.send_message(user_key, Channels::Uno, &StartGame::new());

                // Add the player to the game
                game.players.push(PlayerData {
                    player: player.clone(),
                    user_key: **user_key,
                    server_entity: global.user_keys_entities[user_key],
                });

                // Send the intial card to the player
                for _ in 0..game.settings.initial_cards {
                    draw_card_event.send(DrawCardEvent {
                        user_key: **user_key,
                        game_id: *lobby_id,
                        player_action: false,
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

        game.players
            .sort_unstable_by(|p1, p2| p1.server_entity.id().cmp(&p2.server_entity.id()));

        games.insert(*lobby_id, game);
    }
}

pub fn draw_card(
    mut server: Server<Protocol, Channels>,
    mut games: ResMut<Games>,
    mut draw_card_events: EventReader<DrawCardEvent>,
    mut pass_turn_event: EventWriter<PassTurnEvent>,
) {
    for DrawCardEvent {
        user_key,
        game_id,
        player_action,
    } in draw_card_events.iter()
    {
        let game = match games.get_mut(game_id) {
            Some(g) => g,
            None => {
                error!("Game not found in draw_card");
                continue;
            }
        };

        let card = game.draw_card();
        for player_data in &mut game.players {
            if player_data.user_key == *user_key {
                player_data.player.hand.push(card);
                server.send_message(&player_data.user_key, Channels::Uno, &DrawCard::new(card));

                if *player_action {
                    if !player_data
                        .player
                        .can_play(*game.discard.top().unwrap(), game.current_color)
                    {
                        pass_turn_event.send(PassTurnEvent {
                            skipping: false,
                            game_id: *game_id,
                        });
                    } else {
                        player_data.player.state = PlayerState::PlayingCard;
                    }
                }

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
    for PassTurnEvent { skipping, game_id } in pass_turn_events.iter() {
        let game = match games.get_mut(game_id) {
            Some(g) => g,
            None => {
                error!("Game not found in pass_turn");
                continue;
            }
        };

        game.turn_index = game.next_player_index();

        let current_player_user_key = game.players[game.turn_index].user_key;

        for player_data in &mut game.players {
            if player_data.user_key == current_player_user_key {
                player_data.player.is_playing = true;
                player_data.player.state = PlayerState::WaitingToPlay;
            } else {
                player_data.player.is_playing = false;
            }
        }

        if !skipping {
            let PlayerData {
                player, user_key, ..
            } = &mut game.players[game.turn_index];

            if player.can_play(*game.discard.top().unwrap(), game.current_color) {
                player.state = PlayerState::PlayingCard;
            } else {
                player.state = PlayerState::DrawingCard;
                server.send_message(user_key, Channels::Uno, &HaveToDrawCard::new());
            }
        }
    }
}

pub fn card_played(
    mut server: Server<Protocol, Channels>,
    mut games: ResMut<Games>,
    mut card_played_events: EventReader<CardPlayedEvent>,
    mut play_card_event: EventWriter<PlayCardEvent>,
) {
    for event in card_played_events.iter() {
        let CardPlayedEvent {
            user_key,
            game_id,
            card,
        } = event;

        let game = match games.get_mut(game_id) {
            Some(g) => g,
            None => {
                error!("Game not found in card_played");
                continue;
            }
        };

        let PlayerData { player, .. } = &mut game.players[game.turn_index];

        let valid = if player.state == PlayerState::PlayingCard {
            card.can_be_played(*game.discard.top().unwrap(), game.current_color)
                && player.hand.contains(card)
        } else {
            false
        };

        server.send_message(user_key, Channels::Uno, &CardValidation::new(valid));

        if valid {
            play_card_event.send(PlayCardEvent {
                card: *card,
                game_id: *game_id,
            });
        }
    }
}

pub fn play_card(
    mut server: Server<Protocol, Channels>,
    mut games: ResMut<Games>,
    mut play_card_event: EventReader<PlayCardEvent>,
    mut pass_turn_event: EventWriter<PassTurnEvent>,
    mut draw_card_event: EventWriter<DrawCardEvent>,
    mut game_end_event: EventWriter<GameEndEvent>,
) {
    for PlayCardEvent { card, game_id } in play_card_event.iter() {
        let mut game = match games.get_mut(game_id) {
            Some(g) => g,
            None => {
                error!("Game not found in play_card");
                continue;
            }
        };

        for PlayerData { user_key, .. } in &game.players {
            if *user_key != game.players[game.turn_index].user_key {
                server.send_message(user_key, Channels::Uno, &CardPlayed::new(*card));
            }
        }

        let in_uno = {
            let PlayerData { player, .. } = &mut game.players[game.turn_index];

            let card_idx = if let Some(card_idx) = player.hand.iter().position(|&c| c == *card) {
                card_idx
            } else {
                error!("Player has played a card they doesn't have have in their hand.");
                return;
            };

            player.hand.remove(card_idx);
            game.discard.add(*card);
            game.current_color = card.color;

            if player.hand.len() == 1 {
                player.state = PlayerState::Uno;
            }

            player.state == PlayerState::Uno
        };

        if game.players[game.turn_index].player.hand.is_empty() {
            game_end_event.send(GameEndEvent { game_id: *game_id });

            return;
        }

        if in_uno {
            let current_user_key = game.players[game.turn_index].user_key;

            for player_data in &game.players {
                if current_user_key == player_data.user_key {
                    server.send_message(&player_data.user_key, Channels::Uno, &Uno::new());
                } else {
                    server.send_message(&player_data.user_key, Channels::Uno, &CounterUno::new());
                }
            }
        }

        let pass_turn = match card.value {
            Value::Reverse => reverse_played(*game_id, game, in_uno, &mut pass_turn_event),
            Value::DrawTwo => draw_two_played(
                *game_id,
                game,
                in_uno,
                &mut pass_turn_event,
                &mut draw_card_event,
            ),
            Value::Skip => skip_played(*game_id, in_uno, &mut pass_turn_event),
            Value::Wild => wild_played(game, in_uno),
            Value::WildFour => wild_four_played(*game_id, game, in_uno, &mut draw_card_event),
            _ => true,
        };

        if !in_uno && pass_turn {
            pass_turn_event.send(PassTurnEvent {
                skipping: false,
                game_id: *game_id,
            });
        }
    }
}

fn reverse_played(
    game_id: LobbyId,
    game: &mut Game,
    in_uno: bool,
    pass_turn_event: &mut EventWriter<PassTurnEvent>,
) -> bool {
    game.reverse_turn = !game.reverse_turn;

    if game.players.len() == 2 && !in_uno {
        pass_turn_event.send(PassTurnEvent {
            skipping: true,
            game_id,
        });
    }

    true
}
fn draw_two_played(
    game_id: LobbyId,
    game: &mut Game,
    in_uno: bool,
    pass_turn_event: &mut EventWriter<PassTurnEvent>,
    draw_card_event: &mut EventWriter<DrawCardEvent>,
) -> bool {
    for _ in 0..2 {
        draw_card_event.send(DrawCardEvent {
            user_key: game.players[game.next_player_index()].user_key,
            game_id,
            player_action: false,
        })
    }

    if !in_uno {
        pass_turn_event.send(PassTurnEvent {
            skipping: true,
            game_id,
        });
    }

    true
}

fn skip_played(
    game_id: LobbyId,
    in_uno: bool,
    pass_turn_event: &mut EventWriter<PassTurnEvent>,
) -> bool {
    if !in_uno {
        pass_turn_event.send(PassTurnEvent {
            skipping: true,
            game_id,
        });
    }

    true
}

fn wild_played(game: &mut Game, in_uno: bool) -> bool {
    game.players[game.turn_index].player.state = if in_uno {
        PlayerState::ChoosingColorWildUno {
            uno_done: false,
            color_chosen: false,
        }
    } else {
        PlayerState::ChoosingColorWild
    };

    false
}

fn wild_four_played(
    game_id: LobbyId,
    game: &mut Game,
    in_uno: bool,
    draw_card_event: &mut EventWriter<DrawCardEvent>,
) -> bool {
    game.players[game.turn_index].player.state = if in_uno {
        PlayerState::ChoosingColorWildFourUno {
            uno_done: false,
            color_chosen: false,
        }
    } else {
        PlayerState::ChoosingColorWildFour
    };

    let next_player = game.players[game.next_player_index()].user_key;
    for _ in 0..4 {
        draw_card_event.send(DrawCardEvent {
            user_key: next_player,
            player_action: false,
            game_id,
        });
    }

    false
}

pub fn uno(
    mut server: Server<Protocol, Channels>,
    mut games: ResMut<Games>,
    mut uno_events: EventReader<UnoEvent>,
    mut pass_turn_event: EventWriter<PassTurnEvent>,
) {
    for UnoEvent { game_id, .. } in uno_events.iter() {
        let game = match games.get_mut(game_id) {
            Some(g) => g,
            None => {
                error!("Game not found in uno");
                continue;
            }
        };

        for PlayerData { user_key, .. } in &game.players {
            server.send_message(user_key, Channels::Uno, &StopUno::new());
        }

        let PlayerData { player, .. } = &mut game.players[game.turn_index];

        let (pass_turn, skip_turn) = match player.state {
            PlayerState::Uno => (true, false),
            PlayerState::ChoosingColorWildUno {
                ref mut uno_done,
                color_chosen,
            } => {
                *uno_done = true;
                (*uno_done && color_chosen, false)
            }
            PlayerState::ChoosingColorWildFourUno {
                ref mut uno_done,
                color_chosen,
            } => {
                *uno_done = true;
                (*uno_done && color_chosen, true)
            }
            _ => {
                error!("Player {player:?} called uno without being in Uno state");
                (true, false)
            }
        };

        if pass_turn {
            if skip_turn
                || game.discard.top().unwrap().value == Value::DrawTwo
                || game.discard.top().unwrap().value == Value::Skip
                || (game.players.len() == 2 && game.discard.top().unwrap().value == Value::Reverse)
            {
                pass_turn_event.send(PassTurnEvent {
                    skipping: true,
                    game_id: *game_id,
                });
            }

            pass_turn_event.send(PassTurnEvent {
                skipping: false,
                game_id: *game_id,
            });
        }
    }
}

pub fn counter_uno(
    mut server: Server<Protocol, Channels>,
    mut games: ResMut<Games>,
    mut counter_uno_events: EventReader<CounterUnoEvent>,
    mut draw_card_event: EventWriter<DrawCardEvent>,
    mut pass_turn_event: EventWriter<PassTurnEvent>,
) {
    for CounterUnoEvent { game_id, .. } in counter_uno_events.iter() {
        let game = match games.get_mut(game_id) {
            Some(g) => g,
            None => {
                error!("Game not found in counter_uno");
                continue;
            }
        };

        for PlayerData { user_key, .. } in &game.players {
            server.send_message(user_key, Channels::Uno, &StopUno::new());
        }

        let PlayerData {
            player, user_key, ..
        } = &mut game.players[game.turn_index];

        let (pass_turn, skip_turn) = match player.state {
            PlayerState::Uno => (true, false),
            PlayerState::ChoosingColorWildUno {
                ref mut uno_done,
                color_chosen,
            } => {
                *uno_done = true;
                (*uno_done && color_chosen, false)
            }
            PlayerState::ChoosingColorWildFourUno {
                ref mut uno_done,
                color_chosen,
            } => {
                *uno_done = true;
                (*uno_done && color_chosen, true)
            }
            _ => {
                error!(
                    "Player {player:?} called counter uno when playing player wasn't in Uno state"
                );
                (true, false)
            }
        };

        for _ in 0..2 {
            draw_card_event.send(DrawCardEvent {
                user_key: *user_key,
                game_id: *game_id,
                player_action: false,
            });
        }

        if pass_turn {
            if skip_turn || game.discard.top().unwrap().value == Value::DrawTwo {
                pass_turn_event.send(PassTurnEvent {
                    skipping: true,
                    game_id: *game_id,
                });
            }

            pass_turn_event.send(PassTurnEvent {
                skipping: false,
                game_id: *game_id,
            });
        }
    }
}

pub fn color_chosen(
    mut games: ResMut<Games>,
    mut color_chosen_events: EventReader<ColorChosenEvent>,
    mut pass_turn_event: EventWriter<PassTurnEvent>,
) {
    for ColorChosenEvent { color, game_id } in color_chosen_events.iter() {
        let game = match games.get_mut(game_id) {
            Some(g) => g,
            None => {
                error!("Game not found in color_chosen");
                continue;
            }
        };

        let PlayerData { player, .. } = &mut game.players[game.turn_index];

        let (pass_turn, skip_turn) = match player.state {
            PlayerState::ChoosingColorWild => (true, false),
            PlayerState::ChoosingColorWildFour => (true, true),
            PlayerState::ChoosingColorWildUno {
                uno_done,
                ref mut color_chosen,
            } => {
                *color_chosen = true;
                (uno_done && *color_chosen, false)
            }
            PlayerState::ChoosingColorWildFourUno {
                uno_done,
                ref mut color_chosen,
            } => {
                *color_chosen = true;
                (uno_done && *color_chosen, true)
            }
            _ => {
                error!("Player {player:?} changed color without being in ChoosingColor state");
                return;
            }
        };

        game.current_color = *color;

        if pass_turn {
            if skip_turn {
                pass_turn_event.send(PassTurnEvent {
                    skipping: true,
                    game_id: *game_id,
                });
            }

            pass_turn_event.send(PassTurnEvent {
                skipping: false,
                game_id: *game_id,
            });
        }
    }
}

pub fn game_end(
    mut server: Server<Protocol, Channels>,
    mut games: ResMut<Games>,
    mut game_end_events: EventReader<GameEndEvent>,
    mut players_query: Query<&mut Player>,
) {
    for GameEndEvent { game_id } in game_end_events.iter() {
        let game = match games.get_mut(game_id) {
            Some(g) => g,
            None => {
                error!("Game not found in game_end");
                continue;
            }
        };

        for player_data in &mut game.players {
            player_data.player.score += player_data.player.compute_score();
            server.send_message(&player_data.user_key, Channels::Uno, &GameEnd::new());
        }

        for mut player in players_query.iter_mut() {
            player.hand.clear();
            player.state = PlayerState::WaitingToPlay;
        }
    }
}

pub fn game_exit(
    mut commands: Commands,
    mut server: Server<Protocol, Channels>,
    mut games: ResMut<Games>,
    mut game_exit_events: EventReader<GameExitEvent>,
    mut global: ResMut<Global>,
) {
    for GameExitEvent { user_key, game_id } in game_exit_events.iter() {
        let game = match games.get_mut(game_id) {
            Some(g) => g,
            None => {
                error!("Game not found in game_exit");
                continue;
            }
        };

        let player_index = game
            .players
            .iter()
            .position(|p| p.user_key == *user_key)
            .unwrap();

        commands
            .spawn()
            .insert(game.players[player_index].player.clone())
            .insert(UserKeyComponent(*user_key));

        game.players.remove(player_index);

        server
            .user_mut(user_key)
            .leave_room(&global.lobbies_room_key[game_id])
            .enter_room(&global.main_room_key);

        server
            .room_mut(&global.lobbies_room_key[game_id])
            .remove_entity(&global.user_keys_entities[user_key]);

        server
            .room_mut(&global.main_room_key)
            .add_entity(&global.user_keys_entities[user_key]);

        if game.players.is_empty() {
            server.room_mut(&global.lobbies_room_key[game_id]).destroy();
            global.lobbies_room_key.remove(game_id);
            games.remove(game_id);
        }
    }
}
