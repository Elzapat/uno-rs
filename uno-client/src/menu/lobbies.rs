use super::{LobbiesList, LobbyState};
use crate::{
    game::{ExtraMessageEvent, StartGameEvent},
    utils::errors::Error,
    PlayerId,
};
use bevy::prelude::*;
use naia_bevy_client::events::MessageEvent;
use uno::network::{protocol::YourPlayerId, Channels, Protocol};

pub fn execute_packets(
    mut commands: Commands,
    mut lobby_state: ResMut<State<LobbyState>>,
    mut lobbies: ResMut<LobbiesList>,
    mut player_id: ResMut<PlayerId>,
    mut start_game_event: EventWriter<StartGameEvent>,
    mut message_events: EventReader<MessageEvent<Protocol, Channels>>,
    mut extra_message_events: EventWriter<ExtraMessageEvent>,
) {
    for MessageEvent(_, protocol) in message_events.iter() {
        match protocol {
            Protocol::StartGame(_) => {
                println!("in start game");
                if let LobbyState::InLobby(_) = lobby_state.current() {
                    start_game_event.send(StartGameEvent);
                    break;
                }

                if lobby_state.current() != &LobbyState::LobbiesList {
                    lobby_state.set(LobbyState::LobbiesList).unwrap();
                    lobbies.clear();
                }
            }
            Protocol::JoinLobby(lobby) => {
                lobby_state.set(LobbyState::InLobby(*lobby.id)).unwrap();
            }
            Protocol::LeaveLobby(_) => {
                lobby_state.set(LobbyState::LobbiesList).unwrap();
            }
            Protocol::Error(error) => {
                commands.spawn().insert(Error {
                    message: (*error.error).clone(),
                });
            }
            Protocol::YourPlayerId(YourPlayerId { id }) => **player_id = Some(**id),
            protocol => {
                println!("receiving extra message");
                extra_message_events.send(ExtraMessageEvent(protocol.clone()));
                // return;
            }
        };
    }
}
