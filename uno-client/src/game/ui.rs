use super::{run_if_in_game, CallUno, ChooseColor, ColorChosenEvent, Player, ThisPlayer};
use crate::utils::constants::{CARD_SCALE, CARD_WIDTH};
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
    mut color_chosen_event: EventWriter<ColorChosenEvent>,
) {
    if let Ok(entity) = choose_color.get_single() {
        egui::Window::new(egui::RichText::new("Choose color").strong())
            .anchor(
                egui::Align2::CENTER_CENTER,
                [0.0, CARD_WIDTH * CARD_SCALE / 2.0 + 30.0],
            )
            .collapsible(false)
            .resizable(false)
            .show(egui_context.ctx(), |ui| {
                ui.horizontal(|ui| {
                    const COLORS: [(Color, egui::Color32); 4] = [
                        (Color::Yellow, egui::Color32::from_rgb(255, 255, 22)),
                        (Color::Red, egui::Color32::from_rgb(237, 28, 36)),
                        (Color::Blue, egui::Color32::from_rgb(0, 114, 188)),
                        (Color::Green, egui::Color32::from_rgb(80, 170, 68)),
                    ];
                    const CARD_WIDTH: f32 = 30.0;
                    const CARD_HEIGHT: f32 = 46.2;

                    ui.painter()
                        .rect_filled(ui.clip_rect(), 0.0, egui::Color32::WHITE);

                    for (card_color, egui_color) in COLORS {
                        let size = egui::Vec2::new(CARD_WIDTH, CARD_HEIGHT);
                        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

                        ui.painter().rect_filled(rect, 3.0, egui_color);

                        if response.clicked() {
                            commands.entity(entity).despawn();
                            color_chosen_event.send(ColorChosenEvent(card_color));
                        }
                    }
                });
            });
    }
}

fn call_uno_window(
    mut commands: Commands,
    egui_context: ResMut<EguiContext>,
    call_uno: Query<Entity, With<CallUno>>,
) {
    if let Ok(entity) = call_uno.get_single() {
        egui::Window::new(egui::RichText::new("Choose color").strong())
            .anchor(
                egui::Align2::CENTER_CENTER,
                [0.0, CARD_WIDTH * CARD_SCALE / 2.0 + 30.0],
            )
            .collapsible(false)
            .resizable(false)
            .show(egui_context.ctx(), |ui| {});
    }
}
