use bevy::prelude::*;
use bevy_egui::{ egui, EguiContext };
use crate::{ Settings, Server };
use uno::packet::{
    Command, write_socket, read_socket,
};

pub struct MenuPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum LobbyState {
    InLobby(Lobby),
    LobbiesList,
}
struct LobbiesList(Vec<Lobby>);

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct Lobby {
    id: u8,
    number_players: u8,
}

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(LobbiesList(vec![]))
            .add_state(LobbyState::LobbiesList)
            .add_system_set(
                SystemSet::on_enter(LobbyState::LobbiesList)
                    .with_system(lobbies_info.system())
            )
            .add_system(settings_panel.system())
            .add_system(read_incoming.system())
            .add_system(lobby_panel.system());
    }
}

fn lobbies_info(mut server: ResMut<Server>) {
    write_socket(&mut server.socket, Command::LobbiesInfo, vec![]).unwrap();
}

fn settings_panel(
    egui_context: ResMut<EguiContext>,
    mut settings: ResMut<Settings>,
) {
    egui::TopBottomPanel::top("Settings").show(egui_context.ctx(), |ui| {
        ui.vertical_centered(|ui| {
            ui.add(
                egui::Label::new("Settings")
                    .text_style(egui::TextStyle::Heading)
                    .strong()
            );
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Username: ");
            if ui.text_edit_singleline(&mut settings.username).lost_focus() {
                println!("{}", settings.username);
            }

            ui.checkbox(&mut settings.enable_animations, "Enable animations");
        })
    });
}

fn read_incoming(
    mut server: ResMut<Server>,
    mut lobby_state: ResMut<State<LobbyState>>,
    mut lobbies: ResMut<LobbiesList>,
) {
    if let Ok(packets) = read_socket(&mut server.socket) {
        // println!("{:?}", packets);
        for mut packet in packets {
            info!("{:?}", packet);
            match packet.command {
                Command::JoinLobby => {
                    lobby_state.set(LobbyState::InLobby(Lobby {
                        id: *packet.args.get(0).unwrap(),
                        number_players: 1,
                    })).unwrap();
                },
                Command::LeaveLobby => {
                    lobby_state.set(LobbyState::LobbiesList).unwrap();
                },
                Command::LobbiesInfo => {
                    if let LobbyState::LobbiesList = lobby_state.current() {
                        lobbies.0.drain(..);
                        let lobbies_raw = packet.args.get_range(..);
                        for i in (0..lobbies_raw.len()).step_by(2) {
                            lobbies.0.push(Lobby {
                                id: lobbies_raw[i],
                                number_players: lobbies_raw[i + 1],
                            }) 
                        }
                    }
                }
                _ => {},
            };
        }
    }
}

fn lobby_panel(
    egui_context: ResMut<EguiContext>,
    lobby_state: Res<State<LobbyState>>,
    mut server: ResMut<Server>,
    lobbies: Res<LobbiesList>,
) {
    let window = egui::Window::new("Uno")
        // .fixed_size([400.0, 400.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        // .default_height(400.0)
        // .min_width(400.0)
        .collapsible(false)
        .resizable(false);

    match lobby_state.current() {
        LobbyState::LobbiesList => window.show(egui_context.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Lobbies");
            });

            ui.separator();
            egui::ScrollArea::auto_sized().show(ui, |ui| {
                ui.vertical(|ui| {
                    for lobby in &lobbies.0 {
                        ui.add_space(10.0);
                        ui.group(|ui| {
                            ui.heading(format!("Lobby #{}", lobby.id));
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label(format!("{}/10", lobby.number_players));
                                if ui.button("Join Lobby").clicked() {
                                    write_socket(&mut server.socket, Command::JoinLobby, lobby.id).unwrap();
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
                    info!("here");
                }
            });
        }),
        LobbyState::InLobby(lobby) => window.show(egui_context.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(format!("Lobby #{}", lobby.id));
            });

            ui.separator();
            ui.separator();
            ui.vertical_centered(|ui| {
                if ui.button("Leave lobby").clicked() {
                    write_socket(&mut server.socket, Command::LeaveLobby, lobby.id).unwrap();
                }
            });
        }),
    };
}
