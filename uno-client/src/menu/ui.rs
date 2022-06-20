use super::LobbyState;
use crate::{utils::errors::Error, PlayerId, Settings};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use naia_bevy_client::Client;
use uno::{
    network::{
        protocol::{self, Lobby, Player},
        Channels, Protocol,
    },
    texts::{Language, TextId, Texts},
};

pub fn settings_panel(
    mut settings: ResMut<Settings>,
    mut client: Client<Protocol, Channels>,
    mut egui_context: ResMut<EguiContext>,
    texts: Res<Texts>,
) {
    let language = settings.language;

    egui::TopBottomPanel::top("Settings").show(egui_context.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new(texts.get(TextId::Settings, language))
                    .heading()
                    .strong(),
            );
        });

        ui.separator();

        ui.horizontal(|ui| {
            if ui.text_edit_singleline(&mut settings.username).changed() {
                client.send_message(
                    Channels::Uno,
                    &protocol::Username::new(settings.username.clone()),
                );
            }
            ui.label(texts.get(TextId::Username, language));

            ui.separator();

            ui.checkbox(
                &mut settings.enable_animations,
                texts.get(TextId::EnableAnimations, language),
            );

            ui.separator();

            egui::ComboBox::from_label(texts.get(TextId::Language, language))
                .selected_text(format!("{}", settings.language))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut settings.language,
                        Language::Francais,
                        Language::Francais.to_string(),
                    );
                    ui.selectable_value(
                        &mut settings.language,
                        Language::English,
                        Language::English.to_string(),
                    );
                });
        })
    });
}

pub fn lobby_panel(
    mut commands: Commands,
    mut client: Client<Protocol, Channels>,
    mut egui_context: ResMut<EguiContext>,
    settings: Res<Settings>,
    lobby_state: ResMut<State<LobbyState>>,
    lobbies_query: Query<&Lobby>,
    players_query: Query<&Player>,
    player_id: Res<PlayerId>,
    texts: Res<Texts>,
) {
    let language = settings.language;

    let window = egui::Window::new(texts.get(TextId::UnoTitle, language))
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .resizable(false);

    match lobby_state.current() {
        LobbyState::LobbiesList => window.show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(texts.get(TextId::LobbiesTitle, language));
            });

            ui.separator();
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        for lobby in lobbies_query.iter() {
                            ui.add_space(10.0);

                            ui.group(|ui| {
                                ui.heading(format!(
                                    "{} #{}",
                                    texts.get(TextId::Lobby, language),
                                    *lobby.id
                                ));

                                ui.separator();

                                ui.horizontal(|ui| {
                                    ui.label(format!("{}/10", *lobby.number_of_players));

                                    if ui.button(texts.get(TextId::JoinLobby, language)).clicked() {
                                        if settings.username.trim().is_empty() {
                                            commands.spawn().insert(Error {
                                                message: texts.get(TextId::EnterUsername, language),
                                            });
                                        } else {
                                            client.send_message(
                                                Channels::Uno,
                                                &protocol::JoinLobby::new(*lobby.id),
                                            );
                                        }
                                    }
                                });
                            });
                        }
                        ui.add_space(10.0);
                    });
                });
            ui.separator();

            ui.vertical_centered(|ui| {
                if ui
                    .button(texts.get(TextId::CreateLobby, language))
                    .clicked()
                {
                    client.send_message(Channels::Uno, &protocol::CreateLobby::new());
                }
            });
        }),
        LobbyState::InLobby(lobby_id) => window.show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(format!(
                    "{} #{}",
                    texts.get(TextId::Lobby, language),
                    lobby_id
                ));
            });

            ui.separator();

            for player in players_query.iter() {
                let mut label = egui::RichText::new(format!("➡ {}", *player.username))
                    .monospace()
                    .heading();

                if *player.id == player_id.unwrap_or(0) {
                    label = label.strong();
                }

                ui.label(label);
            }

            ui.separator();

            ui.vertical_centered(|ui| {
                if ui.button(texts.get(TextId::LeaveLobby, language)).clicked() {
                    client.send_message(Channels::Uno, &protocol::LeaveLobby::new(*lobby_id));
                }

                if ui.button(texts.get(TextId::StartGame, language)).clicked() {
                    client.send_message(Channels::Uno, &protocol::StartGame::new());
                }
            });
        }),
        _ => unreachable!(),
    };
}

/*
pub fn unconnected_panel(
    commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    lobby_state: ResMut<State<LobbyState>>,
) {
    egui::Window::new("Unconnected")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("You're not connected to the server");
                ui.add_space(10.0);
                if ui.button("Reconnect").clicked() {
                    // connect_to_server(commands, lobby_state);
                }
            });
        });
}
*/
