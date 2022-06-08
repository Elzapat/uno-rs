use crate::Global;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{
    event::{EventReader, EventWriter},
    system::{Commands, ResMut},
};
use naia_bevy_server::{
    events::{AuthorizationEvent, ConnectionEvent, DisconnectionEvent, MessageEvent},
    Server,
};
use uno::network::{protocol::Player, Channels, Protocol};

// Lobby sent when the user wants to create a lobby
pub struct CreateLobbyEvent;
// Event to ask the server to send a message
#[derive(Deref, DerefMut)]
pub struct SendMessageEvent(pub Protocol);

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
        server.user_mut(&user_key).enter_room(&global.main_room_key);
        commands.spawn();
        server.spawn().insert(Player::new("test".to_owned(), 12));
    }
}

pub fn disconnection_event(mut disconnection_events: EventReader<DisconnectionEvent>) {
    for DisconnectionEvent(user_key, _) in disconnection_events.iter() {}
}

pub fn message_event(
    server: Server<Protocol, Channels>,
    mut global: ResMut<Global>,
    mut message_events: EventReader<MessageEvent<Protocol, Channels>>,
    mut create_lobby_event: EventWriter<CreateLobbyEvent>,
) {
    for MessageEvent(user_key, channels, protocol) in message_events.iter() {
        match protocol {
            Protocol::CreateLobby(_) => create_lobby_event.send(CreateLobbyEvent),
            _ => todo!(),
        }
    }
}
