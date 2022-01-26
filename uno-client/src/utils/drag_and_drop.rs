use crate::{utils::cursor_state::*, SpriteSize};
use bevy::prelude::*;

pub struct DragAndDropPlugin;

impl Plugin for DragAndDropPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_system(hoverable.system())
            .insert_resource(CursorState::default())
            .add_system(start_drag)
            .add_system(drag);
    }
}

#[derive(Component)]
pub struct Draggable;
#[derive(Component)]
pub struct Dragged;
#[derive(Component)]
pub struct Dropped;

fn start_drag(
    mut commands: Commands,
    mouse_button: Res<Input<MouseButton>>,
    mut cursor_state: ResMut<CursorState>,
    query_pressed: Query<(Entity, &Transform, &SpriteSize), With<Draggable>>,
    query_released: Query<Entity, With<Dragged>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        for (entity, transform, size) in query_pressed.iter() {
            // let size = Size { width: 512.0, height: 512.0};
            let half_width = (size.width * transform.scale.x) / 2.0;
            let half_height = (size.height * transform.scale.y) / 2.0;

            if transform.translation.x - half_width < cursor_state.cursor_world.x
                && transform.translation.x + half_width > cursor_state.cursor_world.x
                && transform.translation.y - half_height < cursor_state.cursor_world.y
                && transform.translation.y + half_height > cursor_state.cursor_world.y
            {
                commands.entity(entity).insert(Dragged);
                cursor_state.delta = Vec2::new(0.0, 0.0);
            }
        }
    } else if mouse_button.just_released(MouseButton::Left) {
        for entity in query_released.iter() {
            commands.entity(entity).remove::<Dragged>().insert(Dropped);
        }
    }
}

fn drag(
    mut cursor_state: ResMut<CursorState>,
    mut query_dragged: Query<&mut Transform, With<Dragged>>,
) {
    for mut transform in query_dragged.iter_mut() {
        transform.translation += Vec3::new(cursor_state.delta.x, cursor_state.delta.y, 0.0);
        cursor_state.delta = Vec2::new(0.0, 0.0);
    }
}
