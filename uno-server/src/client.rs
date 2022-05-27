use naia_server::UserKey;
use uno::{lobby::LobbyId, Player};
use uuid::Uuid;

#[derive(Clone, Eq)]
pub struct Client {
    pub id: Uuid,
    pub user_key: UserKey,
    pub in_lobby: Option<LobbyId>,
    pub player: Player,
}

impl Client {
    pub fn new(user_key: UserKey) -> Client {
        let id = Uuid::new_v4();
        Client {
            id,
            user_key,
            in_lobby: None,
            player: Player::new(id, "Unknown Player".to_owned()),
        }
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
