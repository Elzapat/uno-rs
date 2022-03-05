use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

#[derive(Component)]
pub struct Error {
    pub message: String,
}

pub fn display_error(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
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
        .show(egui_context.ctx_mut(), |ui| {
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
