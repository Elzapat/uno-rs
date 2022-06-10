use super::{LobbiesList, LobbyState};
use crate::{utils::errors::Error, Settings};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use naia_bevy_client::Client;
use uno::network::{
    protocol::{self, Lobby, Player},
    Channels, Protocol,
};

pub fn settings_panel(
    mut commands: Commands,
    mut settings: ResMut<Settings>,
    mut client: Client<Protocol, Channels>,
    mut egui_context: ResMut<EguiContext>,
) {
    egui::TopBottomPanel::top("Settings").show(egui_context.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("Settings").heading().strong());
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Username: ");
            if ui.text_edit_singleline(&mut settings.username).changed() {
                // Forbid characters ÿ and þ (255 and 254) because they would break packets
                if settings.username.replace('ÿ', "").replace('þ', "").len()
                    != settings.username.len()
                {
                    commands.spawn().insert(Error {
                        message: "Your username cannot contain the character ÿ or þ.".to_owned(),
                    });
                } else {
                    client.send_message(
                        Channels::Uno,
                        &protocol::Username::new(settings.username.clone()),
                    );
                }
            }

            ui.checkbox(&mut settings.enable_animations, "Enable animations");
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
) {
    let window = egui::Window::new("Uno")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .resizable(false);

    match lobby_state.current() {
        LobbyState::LobbiesList => window.show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Lobbies");
            });

            ui.separator();
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        for lobby in lobbies_query.iter() {
                            ui.add_space(10.0);
                            ui.group(|ui| {
                                ui.heading(format!("Lobby #{}", *lobby.id));
                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}/10", *lobby.number_of_players));
                                    if ui.button("Join Lobby").clicked() {
                                        if settings.username.trim().is_empty() {
                                            commands.spawn().insert(Error {
                                                message:
                                                    "Please enter a username before joining a lobby"
                                                        .to_owned(),
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
                if ui.button("Create lobby").clicked() {
                    client.send_message(Channels::Uno, &protocol::CreateLobby::new());
                }
            });
        }),
        LobbyState::InLobby(lobby_id) => window.show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(format!("Lobby #{}", lobby_id));
            });

            ui.separator();
            for player in players_query.iter() {
                ui.label(
                    egui::RichText::new(format!("➡ {}", *player.username))
                        .monospace()
                        .heading(),
                );
            }
            ui.separator();

            ui.vertical_centered(|ui| {
                if ui.button("Leave lobby").clicked() {
                    client.send_message(Channels::Uno, &protocol::LeaveLobby::new(*lobby_id));
                }

                if ui.button("Start game").clicked() {
                    client.send_message(Channels::Uno, &protocol::StartGame::new());
                }
            });
        }),
        _ => window.show(egui_context.ctx_mut(), |ui| {
            ui.label("This window isn't supposed to show");
        }),
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
