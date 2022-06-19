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
mod game_exit;
mod have_to_draw_card;
mod join_lobby;
mod leave_lobby;
mod lobby;
mod play_card;
mod player;
mod start_game;
mod stop_uno;
mod uno;
mod username;

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
pub use game_exit::GameExit;
pub use have_to_draw_card::HaveToDrawCard;
pub use join_lobby::JoinLobby;
pub use leave_lobby::LeaveLobby;
pub use lobby::Lobby;
pub use play_card::PlayCard;
pub use player::Player;
pub use start_game::StartGame;
pub use stop_uno::StopUno;
pub use uno::Uno;
pub use username::Username;

use naia_shared::Protocolize;

#[derive(Protocolize)]
pub enum Protocol {
    Player(Player),
    Lobby(Lobby),
    CreateLobby(CreateLobby),
    JoinLobby(JoinLobby),
    LeaveLobby(LeaveLobby),
    Username(Username),
    StartGame(StartGame),
    GameEnd(GameEnd),
    GameExit(GameExit),
    // In game commmands
    PlayCard(PlayCard),
    CardPlayed(CardPlayed),
    CardValidation(CardValidation),
    DrawCard(DrawCard),
    ChooseColor(ChooseColor),
    ColorChosen(ColorChosen),
    CurrentColor(CurrentColor),
    Uno(Uno),
    StopUno(StopUno),
    CounterUno(CounterUno),
    HaveToDrawCard(HaveToDrawCard),
    // Other
    Error(Error),
}
