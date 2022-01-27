use super::{lobbies::connect_to_server, LobbiesList, Lobby, LobbyState};
use crate::{utils::errors::Error, Server, Settings};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use uno::packet::{write_socket, Command};

pub fn settings_panel(
    mut commands: Commands,
    mut settings: ResMut<Settings>,
    mut server: ResMut<Server>,
    egui_context: ResMut<EguiContext>,
) {
    egui::TopBottomPanel::top("Settings").show(egui_context.ctx(), |ui| {
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
                    write_socket(
                        &mut server.socket,
                        Command::Username,
                        settings.username.as_bytes(),
                    )
                    .unwrap();
                }
            }

            ui.checkbox(&mut settings.enable_animations, "Enable animations");
        })
    });
}

pub fn lobby_panel(
    mut commands: Commands,
    mut server: ResMut<Server>,
    egui_context: Res<EguiContext>,
    settings: Res<Settings>,
    lobby_state: ResMut<State<LobbyState>>,
    lobbies: Res<LobbiesList>,
    current_lobby: Res<Option<Lobby>>,
) {
    let window = egui::Window::new("Uno")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .resizable(false);

    match lobby_state.current() {
        LobbyState::LobbiesList => window.show(egui_context.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Lobbies");
            });

            ui.separator();
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        for lobby in &lobbies.0 {
                            ui.add_space(10.0);
                            ui.group(|ui| {
                                ui.heading(format!("Lobby #{}", lobby.id));
                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}/10", lobby.number_players));
                                    if ui.button("Join Lobby").clicked() {
                                        if settings.username.trim().is_empty() {
                                            commands.spawn().insert(Error {
                                                message:
                                                    "Please enter a username before joining a lobby"
                                                        .to_owned(),
                                            });
                                        } else {
                                            write_socket(
                                                &mut server.socket,
                                                Command::JoinLobby,
                                                lobby.id,
                                            )
                                            .unwrap();
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
                    write_socket(&mut server.socket, Command::CreateLobby, vec![]).unwrap();
                }
            });
        }),
        LobbyState::InLobby => window.show(egui_context.ctx(), |ui| {
            let lobby = match (*current_lobby).as_ref() {
                Some(l) => l,
                None => return,
            };

            ui.vertical_centered(|ui| {
                ui.heading(format!("Lobby #{}", lobby.id));
            });

            ui.separator();
            for player in &lobby.players {
                ui.label(
                    egui::RichText::new(format!("➡ {}", player.1))
                        .monospace()
                        .heading(),
                );
            }
            ui.separator();

            ui.vertical_centered(|ui| {
                if ui.button("Leave lobby").clicked() {
                    write_socket(&mut server.socket, Command::LeaveLobby, lobby.id).unwrap();
                }

                if ui.button("Start game").clicked() {
                    write_socket(&mut server.socket, Command::StartGame, 0).unwrap();
                }
            });
        }),
        _ => window.show(egui_context.ctx(), |ui| {
            ui.label("This window isn't supposed to show");
        }),
    };
}

pub fn unconnected_panel(
    commands: Commands,
    egui_context: Res<EguiContext>,
    lobby_state: ResMut<State<LobbyState>>,
) {
    egui::Window::new("Unconnected")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .resizable(false)
        .show(egui_context.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("You're not connected to the server");
                ui.add_space(10.0);
                if ui.button("Reconnect").clicked() {
                    connect_to_server(commands, lobby_state);
                }
            });
        });
}
