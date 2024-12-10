use bevy::input::keyboard::{Key, KeyboardInput};
use cosmic_text::{Action, Cursor, Motion, Selection};

use crate::{input::CosmicTextChanged, prelude::*, MaxChars, MaxLines};

pub(super) fn keypress_command(keys: &ButtonInput<KeyCode>) -> bool {
    #[cfg(target_os = "macos")]
    let command = keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);

    #[cfg(not(target_os = "macos"))]
    let command = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    #[cfg(target_arch = "wasm32")]
    let command = if web_sys::window()
        .unwrap()
        .navigator()
        .user_agent()
        .unwrap_or("NoUA".into())
        .contains("Macintosh")
    {
        keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight])
    } else {
        command
    };

    command
}

pub(crate) fn kb_move_cursor(
    active_editor: Res<FocusedWidget>,
    keys: Res<ButtonInput<KeyCode>>,
    mut cosmic_edit_query: Query<(&mut CosmicEditor,)>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    let Some(active_editor_entity) = active_editor.0 else {
        return;
    };
    if let Ok((mut editor,)) = cosmic_edit_query.get_mut(active_editor_entity) {
        if keys.get_just_pressed().len() != 0 {
            editor.cursor_visible = true;
            editor.cursor_timer.reset();
        }

        let command = keypress_command(&keys);

        #[cfg(target_arch = "wasm32")]
        let command = if web_sys::window()
            .unwrap()
            .navigator()
            .user_agent()
            .unwrap_or("NoUA".into())
            .contains("Macintosh")
        {
            keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight])
        } else {
            command
        };

        #[cfg(target_os = "macos")]
        let option = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);

        let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

        // if shift key is pressed
        let already_has_selection = editor.selection() != Selection::None;
        if shift && !already_has_selection {
            let cursor = editor.cursor();
            editor.set_selection(Selection::Normal(cursor));
        }

        #[cfg(target_os = "macos")]
        let should_jump = command && option;
        #[cfg(not(target_os = "macos"))]
        let should_jump = command;

        if should_jump && keys.just_pressed(KeyCode::ArrowLeft) {
            editor.action(&mut font_system.0, Action::Motion(Motion::PreviousWord));
            if !shift {
                editor.set_selection(Selection::None);
            }
            return;
        }
        if should_jump && keys.just_pressed(KeyCode::ArrowRight) {
            editor.action(&mut font_system.0, Action::Motion(Motion::NextWord));
            if !shift {
                editor.set_selection(Selection::None);
            }
            return;
        }
        if should_jump && keys.just_pressed(KeyCode::Home) {
            editor.action(&mut font_system.0, Action::Motion(Motion::BufferStart));
            if !shift {
                editor.set_selection(Selection::None);
            }
            return;
        }
        if should_jump && keys.just_pressed(KeyCode::End) {
            editor.action(&mut font_system.0, Action::Motion(Motion::BufferEnd));
            if !shift {
                editor.set_selection(Selection::None);
            }
            return;
        }

        if keys.just_pressed(KeyCode::ArrowLeft) {
            editor.action(&mut font_system.0, Action::Motion(Motion::Left));
            if !shift {
                editor.set_selection(Selection::None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::ArrowRight) {
            editor.action(&mut font_system.0, Action::Motion(Motion::Right));
            if !shift {
                editor.set_selection(Selection::None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::ArrowUp) {
            editor.action(&mut font_system.0, Action::Motion(Motion::Up));
            if !shift {
                editor.set_selection(Selection::None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::ArrowDown) {
            editor.action(&mut font_system.0, Action::Motion(Motion::Down));
            if !shift {
                editor.set_selection(Selection::None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::Escape) {
            editor.action(&mut font_system.0, Action::Escape);
        }
        if command && keys.just_pressed(KeyCode::KeyA) {
            editor.action(&mut font_system.0, Action::Motion(Motion::BufferEnd));
            let current_cursor = editor.cursor();
            editor.set_selection(Selection::Normal(Cursor {
                line: 0,
                index: 0,
                affinity: current_cursor.affinity,
            }));
            return;
        }
        if keys.just_pressed(KeyCode::Home) {
            editor.action(&mut font_system.0, Action::Motion(Motion::Home));
            if !shift {
                editor.set_selection(Selection::None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::End) {
            editor.action(&mut font_system.0, Action::Motion(Motion::End));
            if !shift {
                editor.set_selection(Selection::None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::PageUp) {
            editor.action(&mut font_system.0, Action::Motion(Motion::PageUp));
            if !shift {
                editor.set_selection(Selection::None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::PageDown) {
            editor.action(&mut font_system.0, Action::Motion(Motion::PageDown));
            if !shift {
                editor.set_selection(Selection::None);
            }
        }
    }
}

pub(crate) fn kb_input_text(
    active_editor: Res<FocusedWidget>,
    keys: Res<ButtonInput<KeyCode>>,
    mut char_evr: EventReader<KeyboardInput>,
    mut cosmic_edit_query: Query<(
        &mut CosmicEditor,
        &mut CosmicEditBuffer,
        &MaxLines,
        &MaxChars,
        Entity,
        Option<&ReadOnly>,
    )>,
    mut evw_changed: EventWriter<CosmicTextChanged>,
    mut font_system: ResMut<CosmicFontSystem>,
    mut is_deleting: Local<bool>,
) {
    let Some(active_editor_entity) = active_editor.0 else {
        return;
    };

    if let Ok((mut editor, buffer, max_lines, max_chars, entity, readonly_opt)) =
        cosmic_edit_query.get_mut(active_editor_entity)
    {
        let command = keypress_command(&keys);
        if keys.get_just_pressed().len() != 0 {
            editor.cursor_visible = true;
            editor.cursor_timer.reset();
        }
        let readonly = readonly_opt.is_some();

        if keys.just_pressed(KeyCode::Backspace) & !readonly {
            // fix for issue #8
            let select = editor.selection();
            match select {
                Selection::Line(cursor) => {
                    if editor.cursor().line == cursor.line && editor.cursor().index == cursor.index
                    {
                        editor.set_selection(Selection::None);
                    }
                }
                Selection::Normal(cursor) => {
                    if editor.cursor().line == cursor.line && editor.cursor().index == cursor.index
                    {
                        editor.set_selection(Selection::None);
                    }
                }
                Selection::Word(cursor) => {
                    if editor.cursor().line == cursor.line && editor.cursor().index == cursor.index
                    {
                        editor.set_selection(Selection::None);
                    }
                }
                Selection::None => {}
            }

            *is_deleting = true;
        }

        if keys.just_released(KeyCode::Backspace) {
            *is_deleting = false;
        }
        if keys.just_pressed(KeyCode::Delete) && !readonly {
            editor.action(&mut font_system.0, Action::Delete);
            editor.with_buffer_mut(|b| b.set_redraw(true));
        }

        if readonly {
            return;
        }

        let mut is_edit = false;
        let mut is_return = false;
        if keys.just_pressed(KeyCode::Enter) {
            is_return = true;
            if (max_lines.0 == 0 || buffer.lines.len() < max_lines.0)
                && (max_chars.0 == 0 || buffer.get_text().len() < max_chars.0)
            {
                // to have new line on wasm rather than E
                is_edit = true;
                editor.action(&mut font_system.0, Action::Insert('\n'));
            }
        }

        if !is_return {
            for char_ev in char_evr.read() {
                is_edit = true;
                if *is_deleting {
                    editor.action(&mut font_system.0, Action::Backspace);
                } else if !command
                    && (max_chars.0 == 0 || buffer.get_text().len() < max_chars.0)
                    && matches!(char_ev.state, bevy::input::ButtonState::Pressed)
                {
                    match &char_ev.logical_key {
                        Key::Character(char) => {
                            let b = char.as_bytes();
                            for c in b {
                                let c: char = (*c).into();
                                editor.action(&mut font_system.0, Action::Insert(c));
                            }
                        }
                        Key::Space => {
                            editor.action(&mut font_system.0, Action::Insert(' '));
                        }
                        _ => (),
                    }
                }
            }
        }

        if !is_edit {
            return;
        }

        evw_changed.send(CosmicTextChanged((
            entity,
            editor.with_buffer_mut(|b| b.get_text()),
        )));
    }
}
