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

#[derive(Component)]
struct Error {
    message: String,
    timer: Timer,
}

#[derive(Component)]
pub struct RefreshTimer(Timer);

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LobbiesList(vec![]))
            .insert_resource(RefreshTimer(Timer::from_seconds(1.0, true)))
            .add_state(LobbyState::LobbiesList)
            .add_system_set(
                SystemSet::on_enter(LobbyState::LobbiesList)
                    .with_system(refresh_lobbies_list)
            )
            .add_system(settings_panel)
            .add_system(read_incoming)
            .add_system(refresh_lobbies_list)
            .add_system(display_error)
            .add_system(lobby_panel);
    }
}

fn display_error(
    mut commands: Commands,
    time: Res<Time>,
    egui_context: ResMut<EguiContext>,
    mut query: Query<(Entity, &mut Error)>
) {
    for (entity, mut error) in query.iter_mut() {
        error.timer.tick(time.delta());

        egui::show_tooltip(egui_context.ctx(), egui::Id::new("Error"), |ui| {
            ui.label(&error.message);
        });

        if error.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn refresh_lobbies_list(
    time: Res<Time>,
    mut refresh_timer: ResMut<RefreshTimer>,
    mut server: ResMut<Server>,
) {
    refresh_timer.0.tick(time.delta());

    if refresh_timer.0.finished() {
        write_socket(&mut server.socket, Command::LobbiesInfo, vec![]).unwrap();
    }
}

fn settings_panel(
    mut settings: ResMut<Settings>,
    mut server: ResMut<Server>,
    egui_context: ResMut<EguiContext>,
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
                write_socket(&mut server.socket, Command::Username, settings.username.as_bytes()).unwrap();
            }

            ui.checkbox(&mut settings.enable_animations, "Enable animations");
        })
    });
}

fn read_incoming(
    mut commands: Commands,
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
                            });
                        }
                    }
                },
                Command::Error => {
                    if let Ok(error) = String::from_utf8(packet.args.get_range(..)) {
                        commands.spawn().insert(Error {
                            message: error,
                            timer: Timer::from_seconds(5.0, false),
                        });
                    }
                }
                _ => {},
            };
        }
    }
}

fn lobby_panel(
    mut commands: Commands,
    mut server: ResMut<Server>,
    egui_context: Res<EguiContext>,
    settings: Res<Settings>,
    lobby_state: Res<State<LobbyState>>,
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
            egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
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
                                            message: "Please enter a username before joining a lobby".to_owned(),
                                            timer: Timer::from_seconds(3.0, false),
                                        });
                                    } else {
                                        write_socket(&mut server.socket, Command::JoinLobby, lobby.id).unwrap();
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
                    if settings.username.trim().is_empty() {
                        commands.spawn().insert(Error {
                            message: "Please enter a username before creating a lobby".to_owned(),
                            timer: Timer::from_seconds(3.0, false),
                        });
                    } else {
                        write_socket(&mut server.socket, Command::CreateLobby, vec![]).unwrap();
                    }
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
