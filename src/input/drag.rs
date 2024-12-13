use crate::prelude::*;

use super::{warn_no_editor_on_picking_event, InputState};
use cosmic_text::Action;
use impls::coords::RelativeQuery;

impl InputState {
    pub fn is_dragging(&self) -> bool {
        matches!(self, InputState::Dragging { .. })
    }

    /// Handler for [`DragStart`] event
    pub fn start_dragging(&mut self, initial_buffer_coord: Vec2) {
        trace!("Starting a drag");
        match self {
            InputState::Idle | InputState::Hovering => {
                *self = InputState::Dragging {
                    initial_buffer_coord,
                };
            }
            InputState::Dragging { .. } => {
                // warn!(
                //     message = "Somehow, a `DragStart` event was received before a previous `DragStart` event was ended with a `DragEnd`",
                //     note = "Ignoring",
                // );
            }
        }
    }

    /// Handler for [`Move`]
    pub fn continue_dragging(&self) {
        match self {
            InputState::Dragging { .. } => {}
            InputState::Idle | InputState::Hovering => {
                // warn!(
                //     message = "Somehow, a `Move` event was received before a previous `DragStart` event was received",
                //     note = "Ignoring",
                // );
            }
        }
    }

    /// Handler for [`Out`] event
    pub fn end_dragging(&mut self) {
        trace!("Ending drag");
        match self {
            InputState::Dragging { .. } => {
                *self = InputState::Idle;
            }
            InputState::Idle | InputState::Hovering => {
                // warn!(
                //     message = "Somehow, a `DragEnd` event was received before a previous `DragStart` event was received",
                //     note = "Ignoring",
                // );
            }
        }
    }
}

pub(super) fn handle_dragstart(
    trigger: Trigger<Pointer<DragStart>>,
    mut editor: Query<(&mut InputState, &mut CosmicEditor, RelativeQuery), With<CosmicEditBuffer>>,
    mut font_system: ResMut<CosmicFontSystem>,
) -> impls::Result<()> {
    let font_system = &mut font_system.0;
    let event = trigger.event();
    let Ok((mut input_state, mut editor, sprite_relative)) = editor.get_mut(trigger.target) else {
        warn_no_editor_on_picking_event("handling cursor `DragStart` event");
        return Ok(());
    };
    let buffer_size = editor.with_buffer_mut(|b| b.borrow_with(font_system).expected_size());
    let buffer_coord = sprite_relative.compute_buffer_coord(&event.hit, buffer_size)?;
    let mut editor = editor.borrow_with(font_system);

    if event.button != PointerButton::Primary {
        return Ok(());
    }

    input_state.start_dragging(buffer_coord);

    if input_state.is_dragging() {
        editor.action(Action::Click {
            x: buffer_coord.x as i32,
            y: buffer_coord.y as i32,
        });
        editor.action(Action::Drag {
            x: buffer_coord.x as i32,
            y: buffer_coord.y as i32,
        });
    }

    Ok(())
}

pub(super) fn handle_drag_continue(
    trigger: Trigger<Pointer<Drag>>,
    mut editor: Query<(&InputState, &mut CosmicEditor)>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    let font_system = &mut font_system.0;
    let event = &trigger.event;
    let entity = trigger.target;

    if event.button != PointerButton::Primary {
        return;
    }

    let Ok((input_state, mut editor)) = editor.get_mut(entity) else {
        warn_no_editor_on_picking_event("handling cursor `Drag` event");
        return;
    };

    input_state.continue_dragging();

    if let InputState::Dragging {
        initial_buffer_coord,
    } = *input_state
    {
        let new_buffer_coord = initial_buffer_coord + event.distance;
        editor.action(
            font_system,
            Action::Drag {
                x: new_buffer_coord.x as i32,
                y: new_buffer_coord.y as i32,
            },
        );
    }
}

pub(super) fn handle_dragend(
    trigger: Trigger<Pointer<DragEnd>>,
    mut editor: Query<&mut InputState, With<CosmicEditBuffer>>,
) {
    let event = &trigger.event;
    let entity = trigger.target;

    if event.button != PointerButton::Primary {
        return;
    }

    let Ok(entity_mut) = editor.get_mut(entity) else {
        warn_no_editor_on_picking_event("handling cursor `DragEnd` event");
        return;
    };
    let input_state = entity_mut.into_inner();

    input_state.end_dragging();
}
