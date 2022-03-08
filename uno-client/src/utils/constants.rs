pub const CARD_WIDTH: f32 = 2010.0 / 12.0 - CARD_PADDING;
pub const CARD_HEIGHT: f32 = 1549.0 / 6.0 - CARD_PADDING;
pub const CARD_PADDING: f32 = 2.0;
pub const CARD_SCALE: f32 = 1.1;
pub const CARD_ANIMATION_SPEED: f32 = 400.0;
pub const CARD_ANIMATION_TIME_S: f32 = 1.0;
pub const BASE_CARD_Z: f32 = 1.0;

pub const DRAGGED_ENTITY_Z: f32 = 100.0;
pub const Z_INCREASE: f32 = 0.01;
pub const DISCARD_Z_INCREASE: f32 = 0.001;

pub const DECK_POS: (f32, f32) = (-CARD_WIDTH / 2.0 - 20.0, 50.0);
pub const DISCARD_POS: (f32, f32) = (CARD_WIDTH / 2.0 + 20.0, 50.0);

pub const CARD_DROP_ZONE: f32 = CARD_WIDTH;

use bevy_egui::egui;
use uno::card::Color;
pub const COLORS: [(Color, egui::Color32); 4] = [
    (Color::Yellow, egui::Color32::from_rgb(255, 255, 22)),
    (Color::Red, egui::Color32::from_rgb(237, 28, 36)),
    (Color::Blue, egui::Color32::from_rgb(0, 114, 188)),
    (Color::Green, egui::Color32::from_rgb(80, 170, 68)),
];
