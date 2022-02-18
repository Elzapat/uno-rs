use super::{run_if_in_game, ChooseColor, Player, ThisPlayer};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use uno::card::Color;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(run_if_in_game)
                .with_system(players_panel)
                .with_system(choose_color_window),
        );
    }
}

fn players_panel(
    egui_context: ResMut<EguiContext>,
    players_query: Query<(&Player, Option<&ThisPlayer>)>,
) {
    egui::TopBottomPanel::top("Players").show(egui_context.ctx(), |ui| {
        ui.vertical_centered(|ui| {
            let size = players_query.iter().count();

            ui.columns(size, |cols| {
                for (col, (player, this_player)) in cols.iter_mut().zip(players_query.iter()) {
                    col.vertical_centered(|ui| {
                        // const Y_PADDING: f32 = 5.0;
                        const CARD_WIDTH: f32 = 12.5;
                        const CARD_HEIGHT: f32 = 19.25; // height = 1.54 * width for a uno card
                        const CARD_PADDING: f32 = 2.0;

                        let mut text = egui::RichText::new(&player.username);

                        // if player.is_playing {
                        //     text = text.underline();
                        // }

                        if this_player.is_some() {
                            text = text.color(egui::Color32::LIGHT_BLUE);
                        }

                        ui.label(text);

                        let size = egui::Vec2::new(
                            (CARD_WIDTH + CARD_PADDING) * player.hand_size as f32 - CARD_PADDING,
                            CARD_HEIGHT,
                        );
                        let (mut rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                        rect.set_width(CARD_WIDTH);

                        let card_color = if player.is_playing {
                            egui::Color32::YELLOW
                        } else {
                            egui::Color32::from_gray(200)
                        };

                        for _ in 0..player.hand_size {
                            ui.painter().rect_filled(rect, 2.0, card_color);
                            rect.set_center(
                                rect.center() + egui::Vec2::new(CARD_WIDTH + CARD_PADDING, 0.0),
                            );
                        }
                    });
                }
            });
        });
    });
}

fn choose_color_window(
    mut commands: Commands,
    egui_context: ResMut<EguiContext>,
    choose_color: Query<Entity, With<ChooseColor>>,
) {
    if let Ok(entity) = choose_color.get_single() {
        egui::Window::new(egui::RichText::new("Choose color").strong())
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 50.0])
            .collapsible(false)
            .resizable(false)
            .show(egui_context.ctx(), |ui| {
                ui.horizontal(|ui| {
                    const COLORS: [(Color, egui::Color32); 4] = [
                        (Color::Yellow, egui::Color32::YELLOW),
                        (Color::Red, egui::Color32::RED),
                        (Color::Blue, egui::Color32::BLUE),
                        (Color::Green, egui::Color32::GREEN),
                    ];
                    const CARD_WIDTH: f32 = 30.0;
                    const CARD_HEIGHT: f32 = 46.2;
                    const CARD_PADDING: f32 = 5.0;

                    let size = egui::Vec2::new(
                        (CARD_WIDTH + CARD_PADDING) * 4.0 - CARD_PADDING,
                        CARD_HEIGHT,
                    );
                    let (mut rect, _) = ui.allocate_exact_size(size, egui::Sense::click());
                    rect.set_width(CARD_WIDTH);

                    for (_card_color, egui_color) in COLORS {
                        ui.painter().rect_filled(rect, 3.0, egui_color);
                        rect.set_center(
                            rect.center() + egui::Vec2::new(CARD_WIDTH + CARD_PADDING, 0.0),
                        );
                    }
                });
            });
    }
}
