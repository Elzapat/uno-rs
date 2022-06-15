use crate::{
    game::{CardPlayedEvent, StartGameEvent},
    lobbies::{CreateLobbyEvent, JoinLobbyEvent, LeaveLobbyEvent},
    server::{UserKeyComponent, UsernameChangedEvent},
    Global,
};
use bevy_ecs::prelude::*;
use bevy_log::info;
use naia_bevy_server::{
    events::{AuthorizationEvent, ConnectionEvent, DisconnectionEvent, MessageEvent},
    Server,
};
use uno::{
    network::{
        protocol::{Player as NetworkPlayer, *},
        Channels, Protocol,
    },
    Player,
};

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

        let id = server
            .spawn()
            .enter_room(&global.main_room_key)
            .insert(NetworkPlayer::new(None, "Unknown Player".to_owned(), 0))
            .id();

        global.user_keys_entities.insert(*user_key, id);
    }
}

pub fn disconnection_event(mut disconnection_events: EventReader<DisconnectionEvent>) {
    for DisconnectionEvent(_user_key, _) in disconnection_events.iter() {
        info!("A user disconnected");
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
) {
    for MessageEvent(user_key, _channel, protocol) in message_events.iter() {
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
            Protocol::StartGame(_) => start_game_event.send(StartGameEvent(0)),
            Protocol::CardPlayed(CardPlayed { color, value }) => {
                card_validation_event.send(CardPlayedEvent {
                    user_key: *user_key,
                    game_id: user_lobby.unwrap(),
                    card: (**color, **value).into(),
                })
            }
            _ => todo!(),
        }
    }
}
