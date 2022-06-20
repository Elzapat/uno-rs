use super::{
    run_if_in_end_game_lobby, run_if_in_game, CallCounterUno, CallUno, ChooseColor,
    ColorChosenEvent, DrawCard,
};
use crate::{
    game::GameExitEvent,
    utils::constants::{CARD_SCALE, CARD_WIDTH, COLORS},
    PlayerId, Settings,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use itertools::Itertools;
use naia_bevy_client::Client;
use uno::{
    card::{Card, Color},
    network::{
        protocol::{self, CurrentColor, GameExit, Player, Protocol},
        Channels,
    },
    texts::{TextId, Texts},
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
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_run_criteria(run_if_in_end_game_lobby)
                .with_system(end_game_lobby),
        );
    }
}

fn end_game_lobby(
    mut client: Client<Protocol, Channels>,
    mut egui_context: ResMut<EguiContext>,
    mut game_exit_event: EventWriter<GameExitEvent>,
    players_query: Query<(Entity, &Player)>,
    player_id: Res<PlayerId>,
    settings: Res<Settings>,
    texts: Res<Texts>,
) {
    let language = settings.language;

    egui::Window::new(egui::RichText::new(texts.get(TextId::EndGameTitle, language)).strong())
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .min_width(700.0)
        .min_height(400.0)
        .collapsible(false)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.columns(3, |cols| {
                for (i, col) in cols.iter_mut().enumerate() {
                    col.label(
                        egui::RichText::new(match i {
                            0 => texts.get(TextId::Username, language),
                            1 => texts.get(TextId::Score, language),
                            2 => texts.get(TextId::RemainingCards, language),
                            _ => unreachable!(),
                        })
                        .heading()
                        .strong(),
                    );
                }
            });

            ui.separator();

            let players = players_query
                .iter()
                .sorted_by(|(_, p1), (_, p2)| p1.score.cmp(&p2.score));

            for (_, player) in players {
                ui.columns(3, |cols| {
                    if *player.id == player_id.unwrap_or(0) {
                        cols[0].label(egui::RichText::new(&*player.username).strong());
                        cols[1].label(egui::RichText::new(player.score.to_string()).strong());
                    } else {
                        cols[0].label(&*player.username);
                        cols[1].label(player.score.to_string());
                    }

                    small_card_count(&mut cols[2], player, Color::Yellow);
                });

                ui.separator();
            }

            ui.vertical_centered(|ui| {
                // if ui.button("Replay").clicked() {
                //     todo!();
                // }

                if ui.button(texts.get(TextId::BackToMenu, language)).clicked() {
                    game_exit_event.send(GameExitEvent);
                    client.send_message(Channels::Uno, &GameExit::new());
                }
            })
        });
}

fn players_panel(
    mut egui_context: ResMut<EguiContext>,
    players_query: Query<&Player>,
    current_color_query: Query<&CurrentColor>,
    player_id: Res<PlayerId>,
) {
    egui::TopBottomPanel::top("Players").show(egui_context.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            let size = players_query.iter().count();
            let current_color = match current_color_query.get_single() {
                Ok(CurrentColor { color }) => (**color).into(),
                Err(_) => Color::Black,
            };

            let players = players_query.iter().sorted_by(|p1, p2| p1.id.cmp(&p2.id));

            ui.columns(size, |cols| {
                for (col, player) in cols.iter_mut().zip(players) {
                    col.vertical_centered(|ui| {
                        let mut text = egui::RichText::new(&*player.username);

                        if *player.id == player_id.unwrap_or(0) {
                            text = text.color(egui::Color32::WHITE).underline();
                        }

                        ui.label(text);
                        small_card_count(ui, player, current_color);
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
    mut client: Client<Protocol, Channels>,
    call_uno: Query<Entity, With<CallUno>>,
    call_counter_uno: Query<Entity, With<CallCounterUno>>,
) {
    if let Ok(entity) = call_uno.get_single() {
        button_window(
            egui_context.ctx_mut(),
            "Uno!",
            egui::Align2::RIGHT_BOTTOM,
            egui::Vec2::new(-50.0, -50.0),
            || {
                client.send_message(Channels::Uno, &protocol::Uno::new());
                commands.entity(entity).despawn();
            },
        );
    } else if let Ok(entity) = call_counter_uno.get_single() {
        button_window(
            egui_context.ctx_mut(),
            "Counter Uno!",
            egui::Align2::RIGHT_BOTTOM,
            egui::Vec2::new(-50.0, -50.0),
            || {
                client.send_message(Channels::Uno, &protocol::CounterUno::new());
                commands.entity(entity).despawn();
            },
        );
    }
}

fn draw_card_window(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    draw_card_query: Query<Entity, With<DrawCard>>,
    mut client: Client<Protocol, Channels>,
) {
    if let Ok(entity) = draw_card_query.get_single() {
        button_window(
            egui_context.ctx_mut(),
            "Draw card",
            egui::Align2::LEFT_BOTTOM,
            egui::Vec2::new(50.0, -50.0),
            || {
                client.send_message(Channels::Uno, &protocol::DrawCard::new(Card::back()));
                commands.entity(entity).despawn();
            },
        );
    }
}

fn button_window(
    ctx: &egui::Context,
    text: &str,
    align: egui::Align2,
    offset: egui::Vec2,
    on_click: impl FnOnce(),
) {
    egui::Window::new(egui::RichText::new(text).strong())
        .anchor(align, offset)
        .frame(egui::Frame {
            inner_margin: egui::style::Margin::same(0.0),
            outer_margin: egui::style::Margin::same(0.0),
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

fn small_card_count(ui: &mut egui::Ui, player: &Player, current_color: Color) {
    const CARD_WIDTH: f32 = 12.5;
    const CARD_HEIGHT: f32 = 19.25; // height = 1.54 * width for a uno card
    const CARD_PADDING: f32 = 2.0;

    let width = (CARD_WIDTH + CARD_PADDING) * *player.hand_size as f32 - CARD_PADDING;
    let size = egui::Vec2::new(width, CARD_HEIGHT);
    let (mut rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
    rect.set_width(CARD_WIDTH);

    let card_color = if *player.is_playing {
        let mut color = egui::Color32::WHITE;

        for (uno_color, egui_color) in COLORS {
            if uno_color == current_color {
                color = egui_color;
                break;
            }
        }

        color
    } else {
        egui::Color32::from_gray(150)
    };

    for _ in 0..*player.hand_size {
        ui.painter().rect_filled(rect, 2.0, card_color);
        rect.set_center(rect.center() + egui::Vec2::new(CARD_WIDTH + CARD_PADDING, 0.0));
    }
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

    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = *ui.style().interact(&response);
        let text_pos = ui
            .layout()
            .align_size_within_rect(text.size(), rect.shrink2(BUTTON_PADDING))
            .min;

        ui.painter().rect(
            rect.expand(visuals.expansion),
            visuals.rounding,
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
