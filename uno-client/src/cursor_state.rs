use bevy::prelude::*;
use bevy::render::camera::Camera;

pub struct CursorStatePlugin;

impl Plugin for CursorStatePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_startup_system(setup.system())
            .add_system(cursor_state.system());
    }
}

#[derive(Default)]
pub struct CursorState {
    pub cursor_world: Vec2,
    pub delta: Vec2,
    last_position: Vec2,
}

fn setup(mut commands: Commands) {
    commands.spawn().insert(CursorState::default());
}

fn cursor_state(
    mut cursor_moved: EventReader<CursorMoved>,
    windows: Res<Windows>,
    mut query_cursor_state: Query<&mut CursorState>,
    query_camera: Query<&Transform, With<Camera>>,
) {
    for mut cursor_state in query_cursor_state.iter_mut() {
        cursor_state.delta = Vec2::default();
        for cursor in cursor_moved.iter() {
            let window = windows.get_primary().unwrap();
            let cam_transform = query_camera.single().unwrap();
            let new_cursor = cursor_to_world(window, cam_transform, cursor.position);
            cursor_state.delta = cursor.position - cursor_state.last_position;
            cursor_state.last_position = cursor.position;
            // cursor_state.old_cursor = cursor_state.cursor_world;
            // println!("delta: {}", cursor_state.delta);
            cursor_state.cursor_world = new_cursor;
        }
    }
}

fn cursor_to_world(window: &Window, cam_transform: &Transform, cursor_pos: Vec2) -> Vec2 {
    // get the size of the window
    let size = Vec2::new(window.width() as f32, window.height() as f32);

    // the default orthographic projection is in pixels from the center;
    // just undo the translation
    let screen_pos = cursor_pos - size / 2.0;

    // apply the camera transform
    let out = cam_transform.compute_matrix() * screen_pos.extend(0.0).extend(1.0);
    Vec2::new(out.x, out.y)
}

