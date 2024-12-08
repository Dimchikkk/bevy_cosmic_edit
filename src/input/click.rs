use crate::{
    double_click::{ClickCount, ClickState},
    prelude::*,
};

use super::{warn_no_editor_on_picking_event, InputState};
use cosmic_text::{Action, Motion, Selection};
use render_implementations::RelativeQuery;

pub(super) fn handle_click(
    trigger: Trigger<Pointer<Click>>,
    mut editor: Query<(&mut InputState, &mut CosmicEditor, RelativeQuery)>,
    mut font_system: ResMut<CosmicFontSystem>,
    buttons: Res<ButtonInput<KeyCode>>,
    mut click_state: ClickState,
) {
    let font_system = &mut font_system.0;
    let target = trigger.target;
    let click = trigger.event();

    if click.button != PointerButton::Primary {
        return;
    }

    let Ok((input_state, mut editor, sprite_relative)) = editor.get_mut(target) else {
        warn_no_editor_on_picking_event();
        return;
    };
    let mut editor = editor.borrow_with(font_system);

    let Ok(buffer_coord) = sprite_relative.compute_buffer_coord(&click.hit, editor.logical_size())
    else {
        return;
    };

    match *input_state {
        InputState::Idle => {}
        InputState::Hovering | InputState::Dragging { .. } => {
            warn!(
                message = "Somehow, a `Click` event was received before a previous `DragEnd` event was received",
                note = "Ignoring",
            );
            return;
        }
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
            // // select word
            // editor.action(Action::Motion(Motion::LeftWord));
            // let cursor = editor.cursor();
            // editor.set_selection(Selection::Normal(cursor));
            // editor.action(Action::Motion(Motion::RightWord));
        }
        ClickCount::Triple => {
            // selects line
            editor.action(Action::TripleClick {
                x: buffer_coord.x as i32,
                y: buffer_coord.y as i32,
            });
            // // select paragraph
            // editor.action(Action::Motion(Motion::ParagraphStart));
            // let cursor = editor.cursor();
            // editor.set_selection(Selection::Normal(cursor));
            // editor.action(Action::Motion(Motion::ParagraphEnd));
        }
        ClickCount::MoreThanTriple => {
            // select all
            editor.action(Action::Motion(Motion::BufferStart));
            let cursor = editor.cursor();
            editor.set_selection(Selection::Normal(cursor));
            editor.action(Action::Motion(Motion::BufferEnd));
        }
    }
}
