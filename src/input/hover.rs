use crate::prelude::*;

use super::{warn_no_editor_on_picking_event, InputState};

/// Whenever a pointer enters a widget
#[derive(Event, Debug, Reflect)]
pub struct TextHoverIn;

/// Whenever a pointer exits a widget
#[derive(Event, Debug, Reflect)]
pub struct TextHoverOut;

impl InputState {
    /// `Over` event handler
    pub fn start_hovering(&mut self) {
        match self {
            InputState::Idle => *self = InputState::Hovering,
            InputState::Hovering | InputState::Dragging { .. } => {
                warn!(
                    message = "Somehow, a `Over` event was received before a previous `Over` event was ended with a `Out`",
                    note = "Ignoring",
                );
            }
        }
    }

    pub fn is_hovering(&self) -> bool {
        matches!(self, InputState::Hovering)
    }

    /// `Out` event handler
    pub fn end_hovering(&mut self) {
        match self {
            InputState::Hovering => *self = InputState::Idle,
            InputState::Idle | InputState::Dragging { .. } => {
                warn!(
                    message = "Somehow, a `Out` event was received before a previous `Over` event was received",
                    note = "Ignoring",
                );
            }
        }
    }
}

pub(super) fn handle_hover_start(
    trigger: Trigger<Pointer<Over>>,
    mut editor: Query<&mut InputState, With<CosmicEditBuffer>>,
    hover_in_evw: EventWriter<TextHoverIn>,
) {
    let Ok(mut input_state) = editor.get_mut(trigger.target) else {
        warn_no_editor_on_picking_event();
        return;
    };

    input_state.start_hovering();

    if input_state.is_hovering() {

        // change cursor
    }
}

pub(super) fn handle_hover_end(
    trigger: Trigger<Pointer<Out>>,
    mut editor: Query<&mut InputState, With<CosmicEditBuffer>>,
) {
    let Ok(mut input_state) = editor.get_mut(trigger.target) else {
        warn_no_editor_on_picking_event();
        return;
    };
    input_state.end_hovering();
}
