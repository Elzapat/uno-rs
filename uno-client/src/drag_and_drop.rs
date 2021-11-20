use bevy::prelude::*;
use crate::cursor_state::*;

pub struct DragAndDropPlugin;

impl Plugin for DragAndDropPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // .add_system(hoverable.system())
            .add_system(start_drag.system())
            .add_system(drag.system());
    }
}

#[derive(Default)]
pub struct Cursor(Vec2);

pub struct Draggable;
pub struct Dragged;
pub struct Dropped;

pub struct Hoverable;
pub struct Hovered;

fn start_drag(
    mut commands: Commands,
    mouse_button: Res<Input<MouseButton>>,
    query_pressed: Query<(Entity, &Transform), With<Draggable>>,
    query_released: Query<Entity, With<Dragged>>,
    query_cursor_state: Query<&CursorState>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        for (entity, transform) in query_pressed.iter() {
            let size = Size { width: 512.0, height: 512.0 };
            let cursor_state = query_cursor_state.single().unwrap();

            let half_width = (size.width * transform.scale.x) / 2.0;
            let half_height = (size.height * transform.scale.y) / 2.0;

            if transform.translation.x - half_width < cursor_state.cursor_world.x
                && transform.translation.x + half_width > cursor_state.cursor_world.x
                && transform.translation.y - half_height < cursor_state.cursor_world.y
                && transform.translation.y + half_height > cursor_state.cursor_world.y
            {
                commands.entity(entity).insert(Dragged);
            }
            // if let Ok((entity, sprite)) = query_pressed.single() {
            //     println!("{:?}", entity);
            //     // commands.entity(entity).insert(Dragged);
            // }
        }
    } else if mouse_button.just_released(MouseButton::Left) {
        for entity in query_released.iter() {
            commands.entity(entity)
                .remove::<Dragged>()
                .insert(Dropped);
        }
    }
}

fn drag(
    mut query_dragged: Query<&mut Transform, With<Dragged>>,
    query_cursor_state: Query<&CursorState>,
) {
    if let Ok(cursor_state) = query_cursor_state.single() {
        for mut transform in query_dragged.iter_mut() {
            // println!("-------------------");
            // println!("{:?}", transform.translation);
            // println!("{:?}", cursor_state.cursor_world);
            // let delta = transform.translation - (cursor_state.cursor_world, 0.0).into();
            // println!("{:?}", delta);
            // transform.translation = (cursor_state.cursor_world + delta.into(), 0.0).into();
            // transform.translation.x += transform.translation.x - cursor_state.cursor_world.x;
            // transform.translation.y += transform.translation.y - cursor_state.cursor_world.y;
            transform.translation += (cursor_state.delta, 0.0).into();
        }
    }
}

// fn hoverable(
//     mut commands: Commands,
//     mut window_cursor: EventReader<CursorMoved>,
//     query: Query<(Entity, &Size, &Transform), (With<Hoverable>, Without<Dragged>)>,
// ) {
//     for cursor in window_cursor.iter() {
//         for (entity, size, transform) in query.iter() {
//             // println!("{:?}", "hey");
//             let half_width = (size.width * transform.scale.x) / 2.0;
//             let half_height = (size.height * transform.scale.y) / 2.0;
//
//             // println!("{}:{}", transform.translation.x, transform.translation.y);
//
//             if transform.translation.x - half_width < cursor.position.x
//                 && transform.translation.x + half_width > cursor.position.x
//                 && transform.translation.y - half_height < cursor.position.y
//                 && transform.translation.y + half_height > cursor.position.y
//             {
//                 commands.entity(entity).insert(Hovered);
//             } else {
//                 commands.entity(entity).remove::<Hovered>();
//             }
//         }
//     }
// }
