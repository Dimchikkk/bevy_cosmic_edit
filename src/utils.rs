#[cfg(target_arch = "wasm32")]
pub fn get_timestamp() -> f64 {
    js_sys::Date::now()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_timestamp() -> f64 {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    duration.as_millis() as f64
}

use crate::{ActiveEditor, CosmicEditor, ReadOnly};
use bevy::{prelude::*, ui::Interaction, window::PrimaryWindow};

// util fns for examples
//
pub fn change_active_editor_ui(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, Entity),
        (
            Changed<Interaction>,
            (With<CosmicEditor>, Without<ReadOnly>),
        ),
    >,
) {
    for (interaction, entity) in interaction_query.iter_mut() {
        if let Interaction::Pressed = interaction {
            commands.insert_resource(ActiveEditor {
                entity: Some(entity),
            });
        }
    }
}

pub fn change_active_editor_sprite(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<Input<MouseButton>>,
    mut cosmic_edit_query: Query<
        (&mut Sprite, &GlobalTransform, Entity),
        (With<CosmicEditor>, Without<ReadOnly>),
    >,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    if buttons.just_pressed(MouseButton::Left) {
        for (sprite, node_transform, entity) in &mut cosmic_edit_query.iter_mut() {
            let size = sprite.custom_size.unwrap_or(Vec2::new(1., 1.));
            let x_min = node_transform.affine().translation.x - size.x / 2.;
            let y_min = node_transform.affine().translation.y - size.y / 2.;
            let x_max = node_transform.affine().translation.x + size.x / 2.;
            let y_max = node_transform.affine().translation.y + size.y / 2.;
            if let Some(pos) = window.cursor_position() {
                if let Some(pos) = camera.viewport_to_world_2d(camera_transform, pos) {
                    if x_min < pos.x && pos.x < x_max && y_min < pos.y && pos.y < y_max {
                        commands.insert_resource(ActiveEditor {
                            entity: Some(entity),
                        });
                    };
                }
            };
        }
    }
}
