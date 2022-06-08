mod card_played;
mod card_validation;
mod choose_color;
mod color_chosen;
mod counter_uno;
mod create_lobby;
mod current_color;
mod draw_card;
mod error;
mod game_end;
mod hand_size;
mod have_to_draw_card;
mod player;
// mod join_lobby;
mod leave_lobby;
mod lobby_created;
mod lobby_destroyed;
// mod lobby_info;
mod pass_turn;
mod play_card;
// mod player_joined_lobby;
// mod player_left_lobby;
mod player_score;
mod start_game;
mod stop_counter_uno;
mod stop_uno;
mod uno;
mod username;
mod your_player_id;

pub use card_played::CardPlayed;
pub use card_validation::CardValidation;
pub use choose_color::ChooseColor;
pub use color_chosen::ColorChosen;
pub use counter_uno::CounterUno;
pub use create_lobby::CreateLobby;
pub use current_color::CurrentColor;
pub use draw_card::DrawCard;
pub use error::Error;
pub use game_end::GameEnd;
pub use hand_size::HandSize;
pub use have_to_draw_card::HaveToDrawCard;
// pub use join_lobby::JoinLobby;
pub use leave_lobby::LeaveLobby;
pub use lobby_created::LobbyCreated;
pub use lobby_destroyed::LobbyDestroyed;
// pub use lobby_info::LobbyInfo;
pub use pass_turn::PassTurn;
pub use play_card::PlayCard;
// pub use player_joined_lobby::PlayerJoinedLobby;
// pub use player_left_lobby::PlayerLeftLobby;
pub use player::Player;
pub use player_score::PlayerScore;
pub use start_game::StartGame;
pub use stop_counter_uno::StopCounterUno;
pub use stop_uno::StopUno;
pub use uno::Uno;
pub use username::Username;
pub use your_player_id::YourPlayerId;

use naia_shared::Protocolize;

#[derive(Protocolize)]
pub enum Protocol {
    Player(Player),
    CreateLobby(CreateLobby),
    LobbyCreated(LobbyCreated),
    LobbyDestroyed(LobbyDestroyed),
    // JoinLobby(JoinLobby),
    LeaveLobby(LeaveLobby),
    // PlayerJoinedLobby(PlayerJoinedLobby),
    // PlayerLeftLobby(PlayerLeftLobby),
    // LobbyInfo(LobbyInfo),
    Username(Username),
    StartGame(StartGame),
    GameEnd(GameEnd),
    PlayerScore(PlayerScore),
    // In game commmands
    PlayCard(PlayCard),
    CardPlayed(CardPlayed),
    CardValidation(CardValidation),
    DrawCard(DrawCard),
    HandSize(HandSize),
    ChooseColor(ChooseColor),
    ColorChosen(ColorChosen),
    CurrentColor(CurrentColor),
    Uno(Uno),
    StopUno(StopUno),
    CounterUno(CounterUno),
    StopCounterUno(StopCounterUno),
    HaveToDrawCard(HaveToDrawCard),
    PassTurn(PassTurn),
    YourPlayerId(YourPlayerId),
    // Other
    Error(Error),
}
