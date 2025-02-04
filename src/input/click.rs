use crate::{
    double_click::{ClickCount, ClickState},
    prelude::*,
};

use super::InputState;
use cosmic_text::{Action, Motion, Selection};
use render_implementations::{RelativeQuery, RenderTargetError, RenderTypeScan};

impl InputState {
    /// Handler for [`Click`] event
    pub fn handle_click(&self) {
        trace!("Clicked");
        match self {
            InputState::Idle | InputState::Hovering => {}
            InputState::Dragging { .. } => {
                // warn!(
                //     message = "Click event received while dragging",
                //     state = ?self,
                // )
            }
        }
    }

    /// Should only [`Action::Click`] when not already dragging
    pub fn should_click(&self) -> bool {
        !matches!(self, InputState::Dragging { .. })
    }
}

/// An [`Observer`] that focuses on the desired editor when clicked
pub fn focus_on_click(
    trigger: Trigger<Pointer<Click>>,
    mut focused: ResMut<FocusedWidget>,
    editor_confirmation: Query<RenderTypeScan, With<CosmicEditBuffer>>,
) {
    let Ok(scan) = editor_confirmation.get(trigger.target) else {
        warn!(
            "An entity with the `focus_on_click` observer added was clicked, but didn't have a `CosmicEditBuffer` component",
        );
        return;
    };

    match scan.confirm_conformance() {
        Ok(_) => {
            focused.0 = Some(trigger.target);
        }
        Err(RenderTargetError::NoTargetsAvailable) => {
            warn!("Please use a high-level driver component from `bevy_cosmic_edit::render_implementations` to add the `CosmicEditBuffer` component, e.g. `TextEdit` or `TextEdit2d`");
        }
        Err(err) => {
            warn!(message = "For some reason, the entity that `focus_on_click` was triggered for isn't a valid `CosmicEditor`", ?err);
            // render_implementations::debug_error::<()>(In(Err(err)));
        }
    }
}

/// Handles [`CosmicEditor`] widgets that are already focussed
pub(super) fn handle_focused_click(
    trigger: Trigger<Pointer<Click>>,
    focused: Res<FocusedWidget>,
    mut editor: Query<(&mut InputState, &mut CosmicEditor, RelativeQuery)>,
    mut font_system: ResMut<CosmicFontSystem>,
    buttons: Res<ButtonInput<KeyCode>>,
    mut click_state: ClickState,
) -> render_implementations::Result<()> {
    let font_system = &mut font_system.0;
    let target = trigger.target;
    let click = trigger.event();

    // must be focused
    if focused.0 != Some(target) {
        return Ok(());
    }

    if click.button != PointerButton::Primary {
        return Ok(());
    }

    let Ok((input_state, mut editor, buffer_relative)) = editor.get_mut(target) else {
        // this is expected on first click, idk order of observers
        // warn_no_editor_on_picking_event("handling focussed cursor `Click` event");
        return Ok(());
    };
    let mut editor = editor.borrow_with(font_system);
    input_state.handle_click();

    let buffer_coord = buffer_relative.compute_buffer_coord(&click.hit, editor.expected_size())?;

    if !input_state.should_click() {
        return Ok(());
    }

    match click_state.feed_click() {
        ClickCount::Single => {
            let shift_pressed = buttons.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

            if shift_pressed {
                editor.action(Action::Drag {
                    x: buffer_coord.x as i32,
                    y: buffer_coord.y as i32,
                });
            } else {
                editor.action(Action::Click {
                    x: buffer_coord.x as i32,
                    y: buffer_coord.y as i32,
                });
            }
        }
        ClickCount::Double => {
            // selects word
            editor.action(Action::DoubleClick {
                x: buffer_coord.x as i32,
                y: buffer_coord.y as i32,
            });
        }
        ClickCount::Triple => {
            // selects line
            editor.action(Action::TripleClick {
                x: buffer_coord.x as i32,
                y: buffer_coord.y as i32,
            });
        }
        ClickCount::MoreThanTriple => {
            // select all
            editor.action(Action::Motion(Motion::BufferStart));
            let cursor = editor.cursor();
            editor.set_selection(Selection::Normal(cursor));
            editor.action(Action::Motion(Motion::BufferEnd));
        }
    }

    Ok(())
}
