use bevy::{input::mouse::MouseMotion, prelude::*, window::PrimaryWindow};

use crate::{CosmicEditor, CosmicTextChanged, Focus, ReadOnly};

pub fn hover_sprites(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut cosmic_edit_query: Query<
        (&mut Sprite, &GlobalTransform),
        (With<CosmicEditor>, Without<ReadOnly>),
    >,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut hovered: Local<bool>,
    mut last_hovered: Local<bool>,
) {
    *hovered = false;
    let mut window = windows.single_mut();
    let (camera, camera_transform) = camera_q.single();
    for (sprite, node_transform) in &mut cosmic_edit_query.iter_mut() {
        let size = sprite.custom_size.unwrap_or(Vec2::new(1., 1.));
        let x_min = node_transform.affine().translation.x - size.x / 2.;
        let y_min = node_transform.affine().translation.y - size.y / 2.;
        let x_max = node_transform.affine().translation.x + size.x / 2.;
        let y_max = node_transform.affine().translation.y + size.y / 2.;
        if let Some(pos) = window.cursor_position() {
            if let Some(pos) = camera.viewport_to_world_2d(camera_transform, pos) {
                if x_min < pos.x && pos.x < x_max && y_min < pos.y && pos.y < y_max {
                    *hovered = true;
                }
            }
        }
    }

    if *hovered {
        window.cursor.icon = CursorIcon::Text;
    } else if *last_hovered != *hovered {
        window.cursor.icon = CursorIcon::Default;
    }

    *last_hovered = *hovered;
}

pub fn hover_ui(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, Entity),
        (
            Changed<Interaction>,
            (With<CosmicEditor>, Without<ReadOnly>),
        ),
    >,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut window = windows.single_mut();
    for (interaction, entity) in interaction_query.iter_mut() {
        match interaction {
            Interaction::None => {
                window.cursor.icon = CursorIcon::Default;
            }
            Interaction::Hovered => {
                window.cursor.icon = CursorIcon::Text;
            }
            _ => {}
        }
        if let Interaction::Pressed = interaction {
            commands.insert_resource(Focus(Some(entity)));
        }
    }
}

pub fn hide_cursor_when_typing(
    evr: EventReader<CosmicTextChanged>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut window = windows.single_mut();

    if !evr.is_empty() {
        window.cursor.visible = false;
    }
}

pub fn show_cursor_on_use(
    buttons: Res<Input<MouseButton>>,
    mouse_motion_evr: EventReader<MouseMotion>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut window = windows.single_mut();
    if buttons.get_just_pressed().len() != 0 || !mouse_motion_evr.is_empty() {
        window.cursor.visible = true;
    }
}
