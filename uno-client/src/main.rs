use bevy::prelude::*;
use bevy_egui::{ egui, EguiContext, EguiPlugin };

pub mod cursor_state;
pub mod drag_and_drop;

use drag_and_drop::*;
use cursor_state::*;

pub struct Size {
    width: f32,
    height: f32,
}

pub struct Settings {
    username: String,
    enable_animations: bool,
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(CursorStatePlugin)
        .add_plugin(DragAndDropPlugin)
        .add_startup_system(setup.system())
        .add_system(animate_sprite_system.system())
        .add_system(ui_example.system())
        .insert_resource(Settings { username: String::from(""), enable_animations: true })
        .run();
}

fn ui_example(egui_context: ResMut<EguiContext>, mut settings: ResMut<Settings>) {
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
//     egui::SidePanel::left("my_left_panel").show(egui_context.ctx(), |ui| {
//         ui.set_visible(false);
//         ui.add_space(100.0);
//         ui.vertical_centered(|ui| {
//             ui.add(
//                 egui::Label::new("Uno")
//                     .text_style(egui::TextStyle::Heading)
//                     .strong()
//             );
//         });
//         ui.separator();
//
//         ui.label("Hello World!");
//     });

    egui::Window::new("Uno")
        .fixed_size([400.0, 400.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        // .default_height(400.0)
        // .min_width(400.0)
        .collapsible(false)
        .resizable(false)
        .show(egui_context.ctx(), |ui| {
            egui::ScrollArea::auto_sized().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Lobbies");
                });
                ui.separator();

                ui.vertical(|ui| {
                    for i in 1..1 {
                        ui.add_space(10.0);
                        ui.group(|ui| {
                            ui.heading(format!("Lobby #{}", i));
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label(format!("1/5"));
                                ui.button("Join Lobby");
                                ui.set_enabled(false);
                                ui.button("Leave Lobby");
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
        });
}

fn animate_sprite_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = ((sprite.index as usize + 1) % texture_atlas.textures.len()) as u32;
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    const SPRITE_WIDTH: f32 = 4860.0 / 10.0;
    const SPRITE_HEIGHT: f32 = 4554.0 / 6.0;

    let texture_handle = asset_server.load("cards_a_03.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(SPRITE_WIDTH, SPRITE_HEIGHT), 10, 6);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(1.0)),
            ..Default::default()
        })
        .insert(Timer::from_seconds(0.5, true))
        .insert(Draggable)
        .insert(Size { width: SPRITE_WIDTH, height: SPRITE_HEIGHT });

    let texture_handle = asset_server.load("dirt.png");
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            transform: Transform::from_scale(Vec3::splat(0.5)),
            ..Default::default()
        })
        .insert(Draggable)
        .insert(Size { width: 512.0 * 0.5, height: 512.0 * 0.5 });
}
