use serde::Deserialize;
use std::{collections::HashMap, fmt};

#[derive(Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Language {
    Francais,
    English,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Language::Francais => write!(f, "Français"),
            _ => write!(f, "{self:?}"),
        }
    }
}

#[derive(Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TextId {
    // Menu
    UnoTitle,
    LobbiesTitle,
    Lobby,
    CreateLobby,
    JoinLobby,
    LeaveLobby,
    StartGame,
    // Menu Settings
    Settings,
    Username,
    EnableAnimations,
    Language,
    // Game
    DrawCard,
    Uno,
    CounterUno,
    ChooseColor,
    // End Game
    Score,
    RemainingCards,
    EndGameTitle,
    BackToMenu,
    // Errors
    EnterUsername,
}

pub type Text = HashMap<Language, String>;

#[derive(Deserialize, Debug, Clone)]
pub struct Texts(HashMap<TextId, Text>);

impl Texts {
    pub fn get_all() -> Texts {
        Texts(
            ron::de::from_bytes(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/texts.ron"
            )))
            .unwrap(),
        )
    }

    pub fn get(&self, id: TextId, language: Language) -> String {
        self.0[&id][&language].clone()
    }
}
