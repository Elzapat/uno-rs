use crate::{Server, Settings};
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use itertools::Itertools;
use std::net::TcpStream;
use uno::packet::{read_socket, write_socket, Command, ARG_DELIMITER};
use uuid::Uuid;

pub struct MenuPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum LobbyState {
    InLobby,
    LobbiesList,
    Unconnected,
}
struct LobbiesList(Vec<Lobby>);

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct Lobby {
    id: u8,
    number_players: u8,
    players: Vec<(Uuid, String)>,
}

#[derive(Component)]
struct Error {
    message: String,
}

#[derive(Component)]
pub struct RefreshTimer(Timer);

struct Player {
    id: Uuid,
}

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LobbiesList(vec![]))
            .insert_resource(Option::<Lobby>::None)
            .insert_resource(RefreshTimer(Timer::from_seconds(1.0, true)))
            .add_startup_system(connect_to_server)
            .add_state(LobbyState::Unconnected)
            .add_system_set(
                SystemSet::on_enter(LobbyState::LobbiesList)
                    .with_run_criteria(run_if_connected)
                    .with_system(refresh_lobbies_list),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_connected)
                    .with_system(settings_panel)
                    .with_system(read_incoming)
                    .with_system(lobby_panel),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_not_connected)
                    .with_system(unconnected_panel),
            )
            .add_system(display_error);
    }
}

fn run_if_connected(state: Res<State<LobbyState>>) -> ShouldRun {
    if state.current() != &LobbyState::Unconnected {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn run_if_not_connected(state: Res<State<LobbyState>>) -> ShouldRun {
    if state.current() == &LobbyState::Unconnected {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn connect_to_server(mut commands: Commands, mut state: ResMut<State<LobbyState>>) {
    let socket = match TcpStream::connect("127.0.0.1:2905") {
        Ok(s) => s,
        Err(e) => {
            commands.spawn().insert(Error {
                message: format!("Couldn't connect to server ({}).\n\nYou can try reconnecting, or try another time because the service might be down.", e),
            });
            return;
        }
    };

    socket
        .set_nonblocking(true)
        .expect("Couldn't set socket to nonblocking");
    state.set(LobbyState::LobbiesList).unwrap();
    commands.insert_resource(Server { socket });
}

fn display_error(
    mut commands: Commands,
    egui_context: ResMut<EguiContext>,
    mut query: Query<(Entity, &mut Error)>,
) {
    for (entity, error) in query.iter_mut() {
        egui::Window::new(
            egui::RichText::new("Error")
                .strong()
                .color(egui::Color32::RED),
        )
        .anchor(egui::Align2::CENTER_TOP, [0.0, 100.0])
        .collapsible(false)
        .resizable(false)
        .show(egui_context.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(&error.message);
                ui.add_space(10.0);
                if ui.button("Ok").clicked() {
                    commands.entity(entity).despawn();
                }
            });
        });
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
            ui.label(egui::RichText::new("Settings").heading().strong());
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Username: ");
            if ui.text_edit_singleline(&mut settings.username).changed() {
                write_socket(
                    &mut server.socket,
                    Command::Username,
                    settings.username.as_bytes(),
                )
                .unwrap();
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
    mut current_lobby: ResMut<Option<Lobby>>,
) {
    if let Ok(packets) = read_socket(&mut server.socket) {
        // println!("{:?}", packets);
        for mut packet in packets {
            info!("{:?}", packet);
            match packet.command {
                Command::JoinLobby => {
                    let players = packet
                        .args
                        .get_range(1..)
                        .clone()
                        .split(|&x| x == ARG_DELIMITER)
                        .tuples()
                        .map(|(id, username)| {
                            (
                                Uuid::from_slice(id).unwrap(),
                                String::from_utf8(username.to_vec()).unwrap(),
                            )
                        })
                        .collect::<Vec<(Uuid, String)>>();

                    lobby_state.set(LobbyState::InLobby).unwrap();

                    *current_lobby = Some(Lobby {
                        id: *packet.args.get(0).unwrap(),
                        number_players: 1,
                        players,
                    });
                }
                Command::PlayerJoinedLobby => {
                    if let LobbyState::InLobby = lobby_state.current() {
                        let args = packet.args.get_range(..);
                        let delim_pos = args.iter().position(|&b| b == ARG_DELIMITER).unwrap();
                        let id = Uuid::from_slice(&args[..delim_pos]).unwrap();
                        let username = String::from_utf8(args[delim_pos + 1..].to_vec()).unwrap();
                        (*current_lobby)
                            .as_mut()
                            .unwrap()
                            .players
                            .push((id, username));
                    }
                }
                Command::LeaveLobby => {
                    lobby_state.set(LobbyState::LobbiesList).unwrap();
                    *current_lobby = None;
                }
                Command::PlayerLeftLobby => {
                    if let LobbyState::InLobby = lobby_state.current() {
                        let id = Uuid::from_slice(&packet.args.get_range(..)).unwrap();
                        (*current_lobby)
                            .as_mut()
                            .unwrap()
                            .players
                            .retain(|p| p.0 != id);
                    }
                }
                Command::LobbiesInfo => {
                    if let LobbyState::LobbiesList = lobby_state.current() {
                        lobbies.0 = packet
                            .args
                            .get_range(..)
                            .into_iter()
                            .tuples()
                            .map(|(id, number_players)| Lobby {
                                id,
                                number_players,
                                players: Vec::new(),
                            })
                            .collect::<Vec<Lobby>>();
                    }
                }
                Command::LobbyInfo => {
                    if let LobbyState::InLobby = lobby_state.current() {
                        if let Ok(players_raw) = String::from_utf8(packet.args.get_range(2..)) {
                            let _players = players_raw
                                .split(char::from_digit(ARG_DELIMITER.into(), 10).unwrap())
                                .map(|p| p.to_owned())
                                .collect::<Vec<String>>();
                        }
                    }
                }
                Command::Error => {
                    if let Ok(error) = String::from_utf8(packet.args.get_range(..)) {
                        commands.spawn().insert(Error { message: error });
                    }
                }
                _ => {}
            };
        }
    }
}

fn lobby_panel(
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
                    if settings.username.trim().is_empty() {
                        commands.spawn().insert(Error {
                            message: "Please enter a username before creating a lobby".to_owned(),
                        });
                    } else {
                        write_socket(&mut server.socket, Command::CreateLobby, vec![]).unwrap();
                    }
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
            });
        }),
        _ => window.show(egui_context.ctx(), |ui| {
            ui.label("This window isn't supposed to show");
        }),
    };
}

fn unconnected_panel(
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
