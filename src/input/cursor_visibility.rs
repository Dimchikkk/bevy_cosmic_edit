//! Manages the OS-level cursor aka mouse pointer visibility

use bevy::input::mouse::MouseMotion;
use bevy::{ecs::system::SystemParam, window::PrimaryWindow};

use crate::prelude::*;

use crate::input::CosmicTextChanged;

#[derive(SystemParam)]
pub(crate) struct CursorVisibility<'w> {
    window: Single<'w, &'static mut Window, With<PrimaryWindow>>,
}

impl CursorVisibility<'_> {
    pub fn set_cursor_visibility(&mut self, visible: bool) {
        self.window.cursor_options.visible = visible;
    }
}

pub(super) fn update_cursor_visibility(
    editors_text_changed: EventReader<CosmicTextChanged>,
    mouse_moved: EventReader<MouseMotion>,
    mouse_clicked: Res<ButtonInput<MouseButton>>,
    mut cursor_visibility: CursorVisibility,
) {
    let text_changed_at_all = !editors_text_changed.is_empty();
    if text_changed_at_all {
        cursor_visibility.set_cursor_visibility(false);
    }

    let mouse_moved_at_all = !mouse_moved.is_empty();
    let mouse_clicked_at_all = mouse_clicked.get_just_pressed().len() != 0;
    if mouse_moved_at_all || mouse_clicked_at_all {
        cursor_visibility.set_cursor_visibility(true);
    }
}
