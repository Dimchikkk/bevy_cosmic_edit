// Common functions for examples
use crate::{cosmic_edit::ReadOnly, prelude::*};
use bevy::window::PrimaryWindow;
use cosmic_text::Edit;

/// Trait for adding color conversion from [`bevy::prelude::Color`] to [`cosmic_text::Color`]
pub trait ColorExtras {
    fn to_cosmic(self) -> CosmicColor;
}

impl<T: Into<Color>> ColorExtras for T {
    fn to_cosmic(self) -> CosmicColor {
        let Srgba {
            red,
            green,
            blue,
            alpha,
        } = Into::<Color>::into(self).to_srgba();
        CosmicColor::rgba(
            (red * 255.) as u8,
            (green * 255.) as u8,
            (blue * 255.) as u8,
            (alpha * 255.) as u8,
        )
    }
}

/// System to unfocus editors when \[Esc\] is pressed
pub fn deselect_editor_on_esc(i: Res<ButtonInput<KeyCode>>, mut focus: ResMut<FocusedWidget>) {
    if i.just_pressed(KeyCode::Escape) {
        focus.0 = None;
    }
}

/// Function to find the location of the mouse cursor in a cosmic widget
pub(crate) fn get_node_cursor_pos(
    window: &Window,
    node_transform: &GlobalTransform,
    size: Vec2,
    is_ui_node: bool,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    let node_translation = node_transform.affine().translation;
    let node_bounds = Rect::new(
        node_translation.x - size.x / 2.,
        node_translation.y - size.y / 2.,
        node_translation.x + size.x / 2.,
        node_translation.y + size.y / 2.,
    );

    window.cursor_position().and_then(|pos| {
        if is_ui_node {
            if node_bounds.contains(pos) {
                Some(Vec2::new(
                    pos.x - node_bounds.min.x,
                    pos.y - node_bounds.min.y,
                ))
            } else {
                None
            }
        } else {
            camera
                .viewport_to_world_2d(camera_transform, pos)
                .ok()
                .and_then(|pos| {
                    if node_bounds.contains(pos) {
                        Some(Vec2::new(
                            pos.x - node_bounds.min.x,
                            node_bounds.max.y - pos.y,
                        ))
                    } else {
                        None
                    }
                })
        }
    })
}

/// System to allow focus on click for sprite widgets
pub fn change_active_editor_sprite(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut cosmic_edit_query: Query<
        (&mut Sprite, &GlobalTransform, &Visibility, Entity),
        (With<CosmicEditBuffer>, Without<ReadOnly>),
    >,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    if buttons.just_pressed(MouseButton::Left) {
        for (sprite, node_transform, visibility, entity) in &mut cosmic_edit_query.iter_mut() {
            if visibility == Visibility::Hidden {
                continue;
            }
            let size = sprite.custom_size.unwrap_or(Vec2::ONE);
            let x_min = node_transform.affine().translation.x - size.x / 2.;
            let y_min = node_transform.affine().translation.y - size.y / 2.;
            let x_max = node_transform.affine().translation.x + size.x / 2.;
            let y_max = node_transform.affine().translation.y + size.y / 2.;
            if let Some(pos) = window.cursor_position() {
                if let Ok(pos) = camera.viewport_to_world_2d(camera_transform, pos) {
                    if x_min < pos.x && pos.x < x_max && y_min < pos.y && pos.y < y_max {
                        commands.insert_resource(FocusedWidget(Some(entity)))
                    };
                }
            };
        }
    }
}

/// System to allow focus on click for UI widgets
pub fn change_active_editor_ui(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &CosmicSource),
        (Changed<Interaction>, Without<ReadOnly>),
    >,
) {
    for (interaction, source) in interaction_query.iter_mut() {
        if let Interaction::Pressed = interaction {
            commands.insert_resource(FocusedWidget(Some(source.0)));
        }
    }
}

/// System to print editor text content on change
pub fn print_editor_text(
    text_inputs_q: Query<&CosmicEditor>,
    mut previous_value: Local<Vec<String>>,
) {
    for text_input in text_inputs_q.iter() {
        let current_text: Vec<String> = text_input.with_buffer(|buf| {
            buf.lines
                .iter()
                .map(|bl| bl.text().to_string())
                .collect::<Vec<_>>()
        });
        if current_text == *previous_value {
            return;
        }
        previous_value.clone_from(&current_text);
        info!("Widget text: {:?}", current_text);
    }
}

/// Calls javascript to get the current timestamp
#[cfg(target_arch = "wasm32")]
pub(crate) fn get_timestamp() -> f64 {
    js_sys::Date::now()
}

/// Utility function to get the current unix timestamp
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)] // idk why this isn't used
pub(crate) fn get_timestamp() -> f64 {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    duration.as_millis() as f64
}
