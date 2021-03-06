use crate::{
    game::{
        CardPlayedEvent, ColorChosenEvent, CounterUnoEvent, DrawCardEvent, GameExitEvent,
        StartGameEvent, UnoEvent,
    },
    lobbies::{CreateLobbyEvent, JoinLobbyEvent, LeaveLobbyEvent},
    server::{UserKeyComponent, UsernameChangedEvent},
    Global,
};
use bevy_ecs::prelude::*;
use bevy_log::{error, info};
use naia_bevy_server::{
    events::{AuthorizationEvent, ConnectionEvent, DisconnectionEvent, MessageEvent},
    Server,
};
use std::sync::atomic::{AtomicU64, Ordering};
use uno::{
    network::{
        protocol::{Player as NetworkPlayer, *},
        Channels, Protocol,
    },
    Player,
};

fn new_player_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub fn authorization_event(
    mut auth_events: EventReader<AuthorizationEvent<Protocol>>,
    mut server: Server<Protocol, Channels>,
) {
    for AuthorizationEvent(user_key, _) in auth_events.iter() {
        server.accept_connection(user_key);
    }
}

pub fn connection_event(
    mut commands: Commands,
    mut connection_events: EventReader<ConnectionEvent>,
    mut global: ResMut<Global>,
    mut server: Server<Protocol, Channels>,
) {
    for ConnectionEvent(user_key) in connection_events.iter() {
        info!("New connection");

        commands
            .spawn()
            .insert(Player::default())
            .insert(UserKeyComponent(*user_key));

        server.user_mut(user_key).enter_room(&global.main_room_key);

        let player_id = new_player_id();
        server.send_message(user_key, Channels::Uno, &YourPlayerId::new(player_id));

        server.spawn().enter_room(&global.main_room_key);

        let id = server
            .spawn()
            .enter_room(&global.main_room_key)
            .insert(NetworkPlayer::new(
                player_id,
                None,
                "Unknown Player".to_owned(),
                0,
            ))
            .id();

        global.user_keys_entities.insert(*user_key, id);
    }
}

pub fn disconnection_event(
    mut global: ResMut<Global>,
    mut server: Server<Protocol, Channels>,
    mut disconnection_events: EventReader<DisconnectionEvent>,
    // this_player_query: Query<(Entity, &ThisPlayer)>,
    players_query: Query<(Entity, &UserKeyComponent), With<Player>>,
) {
    for DisconnectionEvent(user_key, _) in disconnection_events.iter() {
        info!("A user disconnected");

        server
            .entity_mut(&global.user_keys_entities[user_key])
            .despawn();

        /*
        for (entity, this_player) in this_player_query.iter() {
            if global.user_keys_entities[user_key] == Entity::from_bits(*this_player.entity) {
                info!("found this player entity");
                server.entity_mut(&entity).despawn();
                break;
            }
        }
        */

        for (entity, player_user_key) in players_query.iter() {
            if *user_key == **player_user_key {
                info!("found player entity");
                server.entity_mut(&entity).despawn();
                break;
            }
        }

        global.user_keys_entities.remove(user_key);
    }
}

pub fn message_event(
    global: Res<Global>,
    server: Server<Protocol, Channels>,
    mut message_events: EventReader<MessageEvent<Protocol, Channels>>,
    mut create_lobby_event: EventWriter<CreateLobbyEvent>,
    mut join_lobby_event: EventWriter<JoinLobbyEvent>,
    mut leave_lobby_event: EventWriter<LeaveLobbyEvent>,
    mut username_change_event: EventWriter<UsernameChangedEvent>,
    mut start_game_event: EventWriter<StartGameEvent>,
    mut card_validation_event: EventWriter<CardPlayedEvent>,
    mut color_chosen_event: EventWriter<ColorChosenEvent>,
    mut draw_card_event: EventWriter<DrawCardEvent>,
    mut uno_event: EventWriter<UnoEvent>,
    mut counter_uno_event: EventWriter<CounterUnoEvent>,
    mut game_exit_event: EventWriter<GameExitEvent>,
) {
    for MessageEvent(user_key, _channel, protocol) in message_events.iter() {
        info!("received message");

        let mut user_lobby = None;
        for (lobby_id, room_key) in &global.lobbies_room_key {
            if server.room(room_key).has_user(user_key) {
                user_lobby = Some(*lobby_id);
            }
        }

        match protocol {
            Protocol::CreateLobby(_) => create_lobby_event.send(CreateLobbyEvent),
            Protocol::JoinLobby(lobby) => join_lobby_event.send(JoinLobbyEvent {
                lobby_id: *lobby.id,
                user_key: *user_key,
            }),
            Protocol::LeaveLobby(lobby) => leave_lobby_event.send(LeaveLobbyEvent {
                lobby_id: *lobby.id,
                user_key: *user_key,
            }),
            Protocol::Username(player) => username_change_event.send(UsernameChangedEvent {
                username: (*player.username).to_owned(),
                user_key: *user_key,
            }),
            Protocol::StartGame(_) => start_game_event.send(StartGameEvent {
                lobby_id: user_lobby.unwrap(),
            }),
            Protocol::CardPlayed(CardPlayed { color, value }) => {
                card_validation_event.send(CardPlayedEvent {
                    user_key: *user_key,
                    game_id: user_lobby.unwrap(),
                    card: (**color, **value).into(),
                })
            }
            Protocol::ColorChosen(ColorChosen { color }) => {
                color_chosen_event.send(ColorChosenEvent {
                    color: (**color).into(),
                    game_id: user_lobby.unwrap(),
                })
            }
            Protocol::Uno(_) => uno_event.send(UnoEvent {
                user_key: *user_key,
                game_id: user_lobby.unwrap(),
            }),
            Protocol::CounterUno(_) => counter_uno_event.send(CounterUnoEvent {
                user_key: *user_key,
                game_id: user_lobby.unwrap(),
            }),
            Protocol::DrawCard(_) => draw_card_event.send(DrawCardEvent {
                user_key: *user_key,
                game_id: user_lobby.unwrap(),
                player_action: true,
            }),
            Protocol::GameExit(_) => game_exit_event.send(GameExitEvent {
                user_key: *user_key,
                game_id: user_lobby.unwrap(),
            }),
            _ => error!("Received unhandled message!"),
        }
    }
}
