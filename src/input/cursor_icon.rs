use bevy::{
    ecs::system::SystemParam,
    window::{PrimaryWindow, SystemCursorIcon},
    winit::cursor::CursorIcon,
};

use crate::prelude::*;

use super::{hover::HoverCursor, InputState};

#[derive(SystemParam)]
pub(crate) struct CursorIconUpdate<'w, 's> {
    window: Single<'w, Entity, With<PrimaryWindow>>,
    commands: Commands<'w, 's>,
}

impl CursorIconUpdate<'_, '_> {
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

enum GlobalCursorState {
    Nothing,
    BufferHovered(CursorIcon),
    FocussedEditorDragging,
}

impl GlobalCursorState {
    pub fn account_for_hovered_buffer(&mut self, hover_cursor: CursorIcon) {
        match self {
            GlobalCursorState::Nothing => *self = GlobalCursorState::BufferHovered(hover_cursor),
            GlobalCursorState::BufferHovered(_) => {
                warn_once!(
                    message = "Multiple buffers hovered at the same time",
                    note = "What to do in this case is not yet implemented"
                );
            }
            GlobalCursorState::FocussedEditorDragging => {}
        }
    }

    pub fn account_for_dragging_focussed_editor(&mut self) {
        *self = GlobalCursorState::FocussedEditorDragging;
    }

    /// `None` indicates use the default icon
    pub fn decide_on_icon(self) -> Option<CursorIcon> {
        match self {
            GlobalCursorState::Nothing => None,
            GlobalCursorState::BufferHovered(icon) => Some(icon),
            GlobalCursorState::FocussedEditorDragging => {
                Some(CursorIcon::System(SystemCursorIcon::Text))
            }
        }
    }
}

/// Doesn't take into account [`crate::UserSelectNone`] or [`crate::ReadOnly`]
pub(super) fn update_cursor_icon(
    editors: Query<(&InputState, &HoverCursor, Entity, Has<CosmicEditor>), With<CosmicEditBuffer>>,
    focused_widget: Res<FocusedWidget>,
    mut cursor_icon: CursorIconUpdate,
) {
    // if an editor is being hovered, prioritize its hover cursor
    // else, reset to default
    let mut cursor_state = GlobalCursorState::Nothing;
    for (input_state, hover_cursor, buffer_entity, is_editor) in editors.iter() {
        match *input_state {
            InputState::Hovering => {
                cursor_state.account_for_hovered_buffer(hover_cursor.0.clone());
            }
            InputState::Idle => {}
            InputState::Dragging { .. } => {
                if is_editor && focused_widget.0 == Some(buffer_entity) {
                    cursor_state.account_for_dragging_focussed_editor();
                }
                // only a readonly non-editor buffer could be dragged,
                // this ignores such a case
            }
        }
    }

    match cursor_state.decide_on_icon() {
        Some(icon) => cursor_icon.set_cursor(icon),
        None => cursor_icon.reset_cursor(),
    }
}
