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
mod join_lobby;
mod leave_lobby;
mod lobby_created;
mod lobby_destroyed;
mod lobby_info;
mod pass_turn;
mod play_card;
mod player_joined_lobby;
mod player_left_lobby;
mod player_score;
mod start_game;
mod stop_counter_uno;
mod stop_uno;
mod uno;
mod username;
mod your_player_id;

use card_played::CardPlayed;
use card_validation::CardValidation;
use choose_color::ChooseColor;
use color_chosen::ColorChosen;
use counter_uno::CounterUno;
use create_lobby::CreateLobby;
use current_color::CurrentColor;
use draw_card::DrawCard;
use error::Error;
use game_end::GameEnd;
use hand_size::HandSize;
use have_to_draw_card::HaveToDrawCard;
use join_lobby::JoinLobby;
use leave_lobby::LeaveLobby;
use lobby_created::LobbyCreated;
use lobby_destroyed::LobbyDestroyed;
use lobby_info::LobbyInfo;
use pass_turn::PassTurn;
use play_card::PlayCard;
use player_joined_lobby::PlayerJoinedLobby;
use player_left_lobby::PlayerLeftLobby;
use player_score::PlayerScore;
use start_game::StartGame;
use stop_counter_uno::StopCounterUno;
use stop_uno::StopUno;
use uno::Uno;
use username::Username;
use your_player_id::YourPlayerId;

use naia_shared::Protocolize;

#[derive(Protocolize)]
pub enum Protocol {
    CreateLobby(CreateLobby),
    LobbyCreated(LobbyCreated),
    LobbyDestroyed(LobbyDestroyed),
    JoinLobby(JoinLobby),
    LeaveLobby(LeaveLobby),
    PlayerJoinedLobby(PlayerJoinedLobby),
    PlayerLeftLobby(PlayerLeftLobby),
    LobbyInfo(LobbyInfo),
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
