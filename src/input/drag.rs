use crate::prelude::*;

use super::{warn_no_editor_on_picking_event, InputState};
use cosmic_text::Action;
use render_implementations::RelativeQuery;

pub(super) fn handle_dragstart(
    trigger: Trigger<Pointer<DragStart>>,
    mut editor: Query<(&mut InputState, &mut CosmicEditor, RelativeQuery)>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    let font_system = &mut font_system.0;
    let event = trigger.event();
    let Ok((mut input_state, mut editor, sprite_relative)) = editor.get_mut(trigger.target) else {
        warn_no_editor_on_picking_event();
        return;
    };
    let buffer_size = editor.with_buffer_mut(|b| b.borrow_with(font_system).logical_size());
    let Ok(buffer_coord) = sprite_relative.compute_buffer_coord(&event.hit, buffer_size) else {
        return;
    };
    let mut editor = editor.borrow_with(font_system);

    if event.button != PointerButton::Primary {
        return;
    }

    match *input_state {
        InputState::Idle => {
            *input_state = InputState::Dragging {
                initial_buffer_coord: buffer_coord,
            };
            editor.action(Action::Click {
                x: buffer_coord.x as i32,
                y: buffer_coord.y as i32,
            });
            editor.action(Action::Drag {
                x: buffer_coord.x as i32,
                y: buffer_coord.y as i32,
            });
        }
        InputState::Hovering | InputState::Dragging { .. } => {
            warn!(
                message = "Somehow, a `DragStart` event was received before a previous `DragStart` event was ended with a `DragEnd`",
                note = "Ignoring",
            );
        }
    }
}

pub(super) fn handle_drag(
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
        warn_no_editor_on_picking_event();
        return;
    };
    match *input_state {
        InputState::Hovering | InputState::Idle => {
            warn!(
                message = "Somehow, a `Drag` event was received before a previous `DragStart` event was received",
                note = "Ignoring",
            );
        }
        InputState::Dragging {
            initial_buffer_coord,
        } => {
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
}

pub(super) fn handle_dragend(
    trigger: Trigger<Pointer<DragEnd>>,
    mut editor: Query<&mut InputState>,
) {
    let event = &trigger.event;
    let entity = trigger.target;

    if event.button != PointerButton::Primary {
        return;
    }

    let Ok(entity_mut) = editor.get_mut(entity) else {
        warn_no_editor_on_picking_event();
        return;
    };
    let input_state = entity_mut.into_inner();

    match *input_state {
        InputState::Hovering | InputState::Idle => {
            warn!(
                message = "Somehow, a `DragEnd` event was received before a previous `DragStart` event was received",
                note = "Ignoring",
            );
        }
        InputState::Dragging { .. } => {
            *input_state = InputState::Idle;
        }
    }
}
