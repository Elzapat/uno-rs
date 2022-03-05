use super::{
    run_if_in_game, CallCounterUno, CallUno, ChooseColor, ColorChosenEvent, DrawCard, Player,
    ThisPlayer,
};
use crate::{
    utils::constants::{CARD_SCALE, CARD_WIDTH},
    Server,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use uno::{
    card::Color,
    packet::{write_socket, Command},
};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(run_if_in_game)
                .with_system(players_panel),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_run_criteria(run_if_in_game)
                .with_system(choose_color_window)
                .with_system(call_uno_window)
                .with_system(draw_card_window),
        );
    }
}

fn players_panel(
    mut egui_context: ResMut<EguiContext>,
    players_query: Query<(&Player, Option<&ThisPlayer>)>,
) {
    egui::TopBottomPanel::top("Players").show(egui_context.ctx_mut(), |ui| {
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
    mut egui_context: ResMut<EguiContext>,
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
            .show(egui_context.ctx_mut(), |ui| {
                ui.horizontal(|ui| {
                    const COLORS: [(Color, egui::Color32); 4] = [
                        (Color::Yellow, egui::Color32::from_rgb(255, 255, 22)),
                        (Color::Red, egui::Color32::from_rgb(237, 28, 36)),
                        (Color::Blue, egui::Color32::from_rgb(0, 114, 188)),
                        (Color::Green, egui::Color32::from_rgb(80, 170, 68)),
                    ];
                    const CARD_WIDTH: f32 = 30.0;
                    const CARD_HEIGHT: f32 = 46.2;

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
    mut egui_context: ResMut<EguiContext>,
    mut server_query: Query<&mut Server>,
    call_uno: Query<Entity, With<CallUno>>,
    call_counter_uno: Query<Entity, With<CallCounterUno>>,
) {
    if let Ok(entity) = call_uno.get_single() {
        button_window(
            egui_context.ctx_mut(),
            "Uno!",
            egui::Align2::RIGHT_CENTER,
            egui::Vec2::new(-50.0, 0.0),
            || {
                let mut server = server_query.single_mut();
                write_socket(&mut server.socket, Command::Uno, vec![]).unwrap();
                commands.entity(entity).despawn();
            },
        );
    } else if let Ok(entity) = call_counter_uno.get_single() {
        button_window(
            egui_context.ctx_mut(),
            "Counter Uno!",
            egui::Align2::RIGHT_CENTER,
            egui::Vec2::new(-50.0, 0.0),
            || {
                let mut server = server_query.single_mut();
                write_socket(&mut server.socket, Command::Uno, vec![]).unwrap();
                commands.entity(entity).despawn();
            },
        );
    }
}

fn draw_card_window(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    draw_card_query: Query<Entity, With<DrawCard>>,
    mut server_query: Query<&mut Server>,
) {
    if let Ok(entity) = draw_card_query.get_single() {
        button_window(
            egui_context.ctx_mut(),
            "Draw card",
            egui::Align2::LEFT_CENTER,
            egui::Vec2::new(50.0, 0.0),
            || {
                let mut server = server_query.single_mut();
                write_socket(&mut server.socket, Command::DrawCard, vec![]).unwrap();
                commands.entity(entity).despawn();
            },
        );
    }
}

fn button_window(
    ctx: &egui::CtxRef,
    text: &str,
    align: egui::Align2,
    offset: egui::Vec2,
    on_click: impl FnOnce(),
) {
    egui::Window::new(egui::RichText::new(text).strong())
        .anchor(align, offset)
        .frame(egui::Frame {
            margin: egui::Vec2::ZERO,
            ..egui::Frame::default()
        })
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
            if ui.add(button(text)).clicked() {
                on_click();
            }
        });
}

// Small reimplentation of egui::Button so I can make a big button. Most of the code of this
// function is copied from the source code of egui::Button
fn custom_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    const BUTTON_PADDING: egui::Vec2 = egui::Vec2::new(100.0, 25.0);

    let text = egui::WidgetText::RichText(
        egui::RichText::new(text)
            .text_style(egui::TextStyle::Heading)
            .strong(),
    );

    let wrap_width = ui.available_width() - (2.0 * BUTTON_PADDING).x;
    let text = text.into_galley(ui, None, wrap_width, egui::TextStyle::Button);
    let desired_size = text.size() + 2.0 * BUTTON_PADDING;

    let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = *ui.style().interact(&response);
        let text_pos = ui
            .layout()
            .align_size_within_rect(text.size(), rect.shrink2(BUTTON_PADDING))
            .min;

        ui.painter().rect(
            rect.expand(visuals.expansion),
            visuals.corner_radius,
            visuals.bg_fill,
            visuals.bg_stroke,
        );
        ui.vertical_centered(|ui| text.paint_with_visuals(ui.painter(), text_pos, &visuals));
    }

    response
}

fn button(text: &str) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| custom_button(ui, text)
}
