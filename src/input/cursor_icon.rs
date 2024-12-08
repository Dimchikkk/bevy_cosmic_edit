use bevy::{
    ecs::system::SystemParam,
    window::{PrimaryWindow, SystemCursorIcon},
    winit::cursor::CursorIcon,
};

use crate::prelude::*;

use super::{hover::HoverCursor, InputState};

#[derive(SystemParam)]
pub struct CursorUpdate<'w, 's> {
    window: Single<'w, Entity, With<PrimaryWindow>>,
    commands: Commands<'w, 's>,
}

impl CursorUpdate<'_, '_> {
    pub fn set_cursor(&mut self, icon: CursorIcon) {
        // trace!(message = "Setting window icon", ?icon);
        self.commands.entity(*self.window).insert(icon);
    }

    /// Resets to default
    pub fn reset_cursor(&mut self) {
        // could also set to default value
        // trace!("Resetting window icon");
        self.commands.entity(*self.window).remove::<CursorIcon>();
    }
}

pub(super) fn update_cursor_hover_state(
    editors: Query<(&InputState, &HoverCursor), With<CosmicEditBuffer>>,
    mut cursor_icon: CursorUpdate,
) {
    for (input_state, hover_cursor) in editors.iter() {
        match input_state {
            InputState::Hovering | InputState::Dragging { .. } => {
                cursor_icon.set_cursor(hover_cursor.0.clone());
            }
            InputState::Idle => {
                cursor_icon.reset_cursor();
            }
        }
    }
}
