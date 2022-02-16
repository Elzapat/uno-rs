use crate::{
    utils::{constants::DRAGGED_ENTITY_Z, cursor_state::*},
    SpriteSize,
};
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
pub struct Dragged {
    old_z: f32,
}
#[derive(Component)]
pub struct Dropped;

fn start_drag(
    mut commands: Commands,
    mouse_button: Res<Input<MouseButton>>,
    mut cursor_state: ResMut<CursorState>,
    mut query_pressed: Query<(Entity, &mut Transform, &SpriteSize), With<Draggable>>,
    mut query_released: Query<(Entity, &mut Transform, &Dragged), Without<Draggable>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        let mut max_z = -1.0;
        for (_, transform, size) in query_pressed.iter() {
            let half_width = (size.width * transform.scale.x) / 2.0;
            let half_height = (size.height * transform.scale.y) / 2.0;

            if transform.translation.z > max_z
                && transform.translation.x - half_width < cursor_state.cursor_world.x
                && transform.translation.x + half_width > cursor_state.cursor_world.x
                && transform.translation.y - half_height < cursor_state.cursor_world.y
                && transform.translation.y + half_height > cursor_state.cursor_world.y
            {
                max_z = transform.translation.z;
            }
        }

        if max_z >= 0.0 {
            for (entity, mut transform, _) in query_pressed.iter_mut() {
                if transform.translation.z == max_z {
                    cursor_state.delta = Vec2::new(0.0, 0.0);
                    commands
                        .entity(entity)
                        .remove::<Draggable>()
                        .insert(Dragged {
                            old_z: transform.translation.z,
                        });
                    transform.translation.z = DRAGGED_ENTITY_Z;
                }
            }
        }
    } else if mouse_button.just_released(MouseButton::Left) {
        for (entity, mut transform, Dragged { old_z }) in query_released.iter_mut() {
            transform.translation.z = *old_z;
            commands
                .entity(entity)
                .remove::<Dragged>()
                .insert(Draggable)
                .insert(Dropped);
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
