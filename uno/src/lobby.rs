use crate::player::Player;
use std::sync::atomic::{AtomicUsize, Ordering};

pub type LobbyId = u32;

#[derive(Debug, Clone)]
pub struct Lobby {
    pub id: LobbyId,
    pub players: Vec<Player>,
}

/// THe maximum amount of players a lobby can contain
pub const MAX_LOBBY_PLAYERS: usize = 10;
/// The maximum amount of lobbies that can be created
pub const MAX_LOBBIES: usize = 10;
/// The time it takes for a lobby to despawn in seconds
pub const LOBBY_DESPAWN_TIME_S: f32 = 150.0;

fn new_lobby_id() -> LobbyId {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    COUNTER
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            Some(value % (MAX_LOBBIES * 2) + 1)
        })
        .unwrap() as LobbyId
}

impl Lobby {
    pub fn new() -> Self {
        Lobby {
            id: new_lobby_id(),
            players: vec![],
        }
    }
}

impl std::default::Default for Lobby {
    fn default() -> Self {
        Lobby::new()
    }
}
