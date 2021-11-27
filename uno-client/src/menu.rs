use bevy::prelude::*;
use bevy_egui::{ egui, EguiContext };
use crate::Settings;

pub struct MenuPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum LobbyState {
    InLobby(Lobby),
    LobbiesList(Vec<Lobby>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct Lobby {
    id: u8,
    number_players: u8,
}

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_state(LobbyState::LobbiesList(vec![]))
            .add_system(settings_panel.system())
            .add_system(lobby_panel.system());
    }
}

fn settings_panel(egui_context: ResMut<EguiContext>, mut settings: ResMut<Settings>) {
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

fn lobby_panel(
    egui_context: ResMut<EguiContext>,
    lobby_state: Res<State<LobbyState>>,
) {
    let window = egui::Window::new("Uno")
        // .fixed_size([400.0, 400.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        // .default_height(400.0)
        // .min_width(400.0)
        .collapsible(false)
        .resizable(false);

    match lobby_state.current() {
        LobbyState::LobbiesList(lobbies) => window.show(egui_context.ctx(), lobbies_list_ui(lobbies)),
        LobbyState::InLobby(lobby) => window.show(egui_context.ctx(), lobby_ui(lobby)),
    };

    fn lobbies_list_ui(lobbies: &Vec<Lobby>) -> impl Fn(&mut egui::Ui) + '_ {
        move |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Lobbies");
            });

            ui.separator();
            egui::ScrollArea::auto_sized().show(ui, |ui| {
                ui.vertical(|ui| {
                    for lobby in lobbies {
                        ui.add_space(10.0);
                        ui.group(|ui| {
                            ui.heading(format!("Lobby #{}", lobby.id));
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label(format!("{}/10", lobby.number_players));
                                ui.button("Join Lobby");
                            });
                        });
                    }
                    ui.add_space(10.0);
                });
            });
            ui.separator();

            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.button("Create lobby");
                ui.add_space(10.0);
            });
        }
    }

    fn lobby_ui(lobby: &Lobby) -> impl Fn(&mut egui::Ui) + '_ {
        move |ui| {
            ui.heading(format!("Lobby #{}", lobby.id));
        }
    }
}
