// Common functions for examples
use crate::{
    prelude::*,
    render_implementations::{ChangedCosmicWidgetSize, CosmicWidgetSize},
};
use bevy::ecs::query::QueryData;
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

/// Quick utility to print the name of an entity if available
#[derive(QueryData)]
struct DebugName {
    name: Option<&'static Name>,
    entity: Entity,
}

impl std::fmt::Debug for DebugNameItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{:?} {:?}", self.name, self.entity)
        match self.name {
            Some(name) => write!(f, "DebugName::Name({:?})", name),
            None => write!(f, "Entity({:?})", self.entity),
        }
    }
}

/// Debug print the size of all editors
#[allow(dead_code)]
#[allow(private_interfaces)]
pub fn print_editor_sizes(
    editors: Query<(CosmicWidgetSize, DebugName), (With<CosmicEditor>, ChangedCosmicWidgetSize)>,
) {
    for (size, name) in editors.iter() {
        println!("Size of editor {:?} is: {:?}", name, size.logical_size());
    }
}

/// Calls javascript to get the current timestamp
#[allow(dead_code)] // idk why this isn't used
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
