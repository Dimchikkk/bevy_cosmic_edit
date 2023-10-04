#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::time::Duration;

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};
use cosmic_text::{Action, AttrsList, BufferLine, Cursor, Edit, Shaping};

use crate::{
    get_node_cursor_pos, get_timestamp, get_x_offset_center, get_y_offset_center,
    save_edit_history, CosmicAttrs, CosmicEditHistory, CosmicEditor, CosmicFontSystem,
    CosmicMaxChars, CosmicMaxLines, CosmicTextChanged, CosmicTextPosition, Focus, ReadOnly,
    XOffset,
};

pub(crate) fn input_mouse(
    windows: Query<&Window, With<PrimaryWindow>>, // Mouse
    active_editor: Res<Focus>,                    // Both
    keys: Res<Input<KeyCode>>,                    // Both
    buttons: Res<Input<MouseButton>>,             // Mouse
    mut cosmic_edit_query: Query<(
        &mut CosmicEditor,   // Both
        &GlobalTransform,    // Mouse
        &CosmicTextPosition, // Mouse, to determine point
        Entity,              // Both
        &XOffset,            // Mouse
        Option<&mut Node>,
        Option<&mut Sprite>,
    )>,
    mut font_system: ResMut<CosmicFontSystem>,    // Both
    mut scroll_evr: EventReader<MouseWheel>,      // Mouse
    camera_q: Query<(&Camera, &GlobalTransform)>, // Mouse
) {
    if active_editor.0.is_none() {
        return;
    }

    let primary_window = windows.single();
    let scale_factor = primary_window.scale_factor() as f32;
    let (camera, camera_transform) = camera_q.iter().find(|(c, _)| c.is_active).unwrap();
    for (mut editor, node_transform, text_position, entity, x_offset, node_opt, sprite_opt) in
        &mut cosmic_edit_query.iter_mut()
    {
        if active_editor.0 != Some(entity) {
            continue;
        }

        let (width, height, is_ui_node) = match node_opt {
            Some(node) => (node.size().x, node.size().y, true),
            None => {
                let sprite = sprite_opt.unwrap();
                let size = sprite.custom_size.unwrap();
                (size.x, size.y, false)
            }
        };

        let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

        // if shift key is pressed
        let already_has_selection = editor.0.select_opt().is_some();
        if shift && !already_has_selection {
            let cursor = editor.0.cursor();
            editor.0.set_select_opt(Some(cursor));
        }

        let (padding_x, padding_y) = match text_position {
            CosmicTextPosition::Center => (
                get_x_offset_center(width * scale_factor, editor.0.buffer()),
                get_y_offset_center(height * scale_factor, editor.0.buffer()),
            ),
            CosmicTextPosition::TopLeft { padding } => (*padding, *padding),
            CosmicTextPosition::Left { padding } => (
                *padding,
                get_y_offset_center(height * scale_factor, editor.0.buffer()),
            ),
        };
        let point = |node_cursor_pos: (f32, f32)| {
            (
                (node_cursor_pos.0 * scale_factor) as i32 - padding_x,
                (node_cursor_pos.1 * scale_factor) as i32 - padding_y,
            )
        };

        if buttons.just_pressed(MouseButton::Left) {
            if let Some(node_cursor_pos) = get_node_cursor_pos(
                primary_window,
                node_transform,
                (width, height),
                is_ui_node,
                camera,
                camera_transform,
            ) {
                let (mut x, y) = point(node_cursor_pos);
                x += x_offset.0.unwrap_or((0., 0.)).0 as i32;
                if shift {
                    editor.0.action(&mut font_system.0, Action::Drag { x, y });
                } else {
                    editor.0.action(&mut font_system.0, Action::Click { x, y });
                }
            }
            return;
        }
        if buttons.pressed(MouseButton::Left) {
            if let Some(node_cursor_pos) = get_node_cursor_pos(
                primary_window,
                node_transform,
                (width, height),
                is_ui_node,
                camera,
                camera_transform,
            ) {
                let (mut x, y) = point(node_cursor_pos);
                x += x_offset.0.unwrap_or((0., 0.)).0 as i32;
                if active_editor.is_changed() && !shift {
                    editor.0.action(&mut font_system.0, Action::Click { x, y });
                } else {
                    editor.0.action(&mut font_system.0, Action::Drag { x, y });
                }
            }
            return;
        }
        for ev in scroll_evr.iter() {
            match ev.unit {
                MouseScrollUnit::Line => {
                    editor.0.action(
                        &mut font_system.0,
                        Action::Scroll {
                            lines: -ev.y as i32,
                        },
                    );
                }
                MouseScrollUnit::Pixel => {
                    let line_height = editor.0.buffer().metrics().line_height;
                    editor.0.action(
                        &mut font_system.0,
                        Action::Scroll {
                            lines: -(ev.y / line_height) as i32,
                        },
                    );
                }
            }
        }
    }
}

/// Handles undo/redo, copy/paste and char input
pub(crate) fn input_kb(
    active_editor: Res<Focus>,                    // Both
    keys: Res<Input<KeyCode>>,                    // Both
    mut char_evr: EventReader<ReceivedCharacter>, // Kb
    mut cosmic_edit_query: Query<(
        &mut CosmicEditor,      // Both
        &mut CosmicEditHistory, // Kb - Undo
        &CosmicAttrs,           // Kb - Undo
        &CosmicMaxLines,        // Kb
        &CosmicMaxChars,        // Kb
        Entity,                 // Both
        Option<&ReadOnly>,
    )>,
    mut evw_changed: EventWriter<CosmicTextChanged>, // Kb
    mut font_system: ResMut<CosmicFontSystem>,       // Both
    mut is_deleting: Local<bool>,                    // Kb
    mut edits_duration: Local<Option<Duration>>,     // Kb - Undo
    mut undoredo_duration: Local<Option<Duration>>,  // Kb - Undo
) {
    for (mut editor, mut edit_history, attrs, max_lines, max_chars, entity, readonly_opt) in
        &mut cosmic_edit_query.iter_mut()
    {
        if active_editor.0 != Some(entity) {
            continue;
        }

        let readonly = readonly_opt.is_some();

        let attrs = &attrs.0;

        let now_ms = get_timestamp();

        #[cfg(target_os = "macos")]
        let command = keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);

        #[cfg(not(target_os = "macos"))]
        let command = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

        let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

        #[cfg(target_os = "macos")]
        let option = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);

        // if shift key is pressed
        let already_has_selection = editor.0.select_opt().is_some();
        if shift && !already_has_selection {
            let cursor = editor.0.cursor();
            editor.0.set_select_opt(Some(cursor));
        }

        #[cfg(target_os = "macos")]
        let should_jump = command && option;
        #[cfg(not(target_os = "macos"))]
        let should_jump = command;

        if should_jump && keys.just_pressed(KeyCode::Left) {
            editor.0.action(&mut font_system.0, Action::PreviousWord);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if should_jump && keys.just_pressed(KeyCode::Right) {
            editor.0.action(&mut font_system.0, Action::NextWord);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if should_jump && keys.just_pressed(KeyCode::Home) {
            editor.0.action(&mut font_system.0, Action::BufferStart);
            // there's a bug with cosmic text where it doesn't update the visual cursor for this action
            // TODO: fix upstream
            editor.0.buffer_mut().set_redraw(true);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if should_jump && keys.just_pressed(KeyCode::End) {
            editor.0.action(&mut font_system.0, Action::BufferEnd);
            // there's a bug with cosmic text where it doesn't update the visual cursor for this action
            // TODO: fix upstream
            editor.0.buffer_mut().set_redraw(true);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }

        if keys.just_pressed(KeyCode::Left) {
            editor.0.action(&mut font_system.0, Action::Left);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::Right) {
            editor.0.action(&mut font_system.0, Action::Right);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::Up) {
            editor.0.action(&mut font_system.0, Action::Up);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::Down) {
            editor.0.action(&mut font_system.0, Action::Down);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }

        if keys.just_pressed(KeyCode::Back) {
            #[cfg(target_arch = "wasm32")]
            editor.0.action(&mut font_system.0, Action::Backspace);
            *is_deleting = true;
        }
        if keys.just_released(KeyCode::Back) {
            *is_deleting = false;
        }
        if keys.just_pressed(KeyCode::Delete) {
            editor.0.action(&mut font_system.0, Action::Delete);
        }
        if keys.just_pressed(KeyCode::Escape) {
            editor.0.action(&mut font_system.0, Action::Escape);
        }
        if command && keys.just_pressed(KeyCode::A) {
            editor.0.action(&mut font_system.0, Action::BufferEnd);
            let current_cursor = editor.0.cursor();
            editor.0.set_select_opt(Some(Cursor {
                line: 0,
                index: 0,
                affinity: current_cursor.affinity,
                color: current_cursor.color,
            }));
            return;
        }
        if keys.just_pressed(KeyCode::Home) {
            editor.0.action(&mut font_system.0, Action::Home);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::End) {
            editor.0.action(&mut font_system.0, Action::End);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::PageUp) {
            editor.0.action(&mut font_system.0, Action::PageUp);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::PageDown) {
            editor.0.action(&mut font_system.0, Action::PageDown);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }

        // redo
        #[cfg(not(target_os = "windows"))]
        let requested_redo = command && shift && keys.just_pressed(KeyCode::Z) && !readonly;
        #[cfg(target_os = "windows")]
        let requested_redo = command && keys.just_pressed(KeyCode::Y);

        if requested_redo {
            let edits = &edit_history.edits;
            if edits.is_empty() {
                return;
            }
            if edit_history.current_edit == edits.len() - 1 {
                return;
            }
            let idx = edit_history.current_edit + 1;
            if let Some(current_edit) = edits.get(idx) {
                editor.0.buffer_mut().lines.clear();
                for line in current_edit.lines.iter() {
                    let mut line_text = String::new();
                    let mut attrs_list = AttrsList::new(attrs.as_attrs());
                    for (text, attrs) in line.iter() {
                        let start = line_text.len();
                        line_text.push_str(text);
                        let end = line_text.len();
                        attrs_list.add_span(start..end, attrs.as_attrs());
                    }
                    editor.0.buffer_mut().lines.push(BufferLine::new(
                        line_text,
                        attrs_list,
                        Shaping::Advanced,
                    ));
                }
                editor.0.set_cursor(current_edit.cursor);
                editor.0.buffer_mut().set_redraw(true);
                edit_history.current_edit += 1;
            }
            *undoredo_duration = Some(Duration::from_millis(now_ms as u64));
            evw_changed.send(CosmicTextChanged((entity, editor.get_text())));
            return;
        }
        // undo
        let requested_undo = command && keys.just_pressed(KeyCode::Z) && !readonly;

        if requested_undo {
            let edits = &edit_history.edits;
            if edits.is_empty() {
                return;
            }
            if edit_history.current_edit <= 1 {
                return;
            }
            let idx = edit_history.current_edit - 1;
            if let Some(current_edit) = edits.get(idx) {
                editor.0.buffer_mut().lines.clear();
                for line in current_edit.lines.iter() {
                    let mut line_text = String::new();
                    let mut attrs_list = AttrsList::new(attrs.as_attrs());
                    for (text, attrs) in line.iter() {
                        let start = line_text.len();
                        line_text.push_str(text);
                        let end = line_text.len();
                        attrs_list.add_span(start..end, attrs.as_attrs());
                    }
                    editor.0.buffer_mut().lines.push(BufferLine::new(
                        line_text,
                        attrs_list,
                        Shaping::Advanced,
                    ));
                }
                editor.0.set_cursor(current_edit.cursor);
                editor.0.buffer_mut().set_redraw(true);
                edit_history.current_edit -= 1;
            }
            *undoredo_duration = Some(Duration::from_millis(now_ms as u64));
            evw_changed.send(CosmicTextChanged((entity, editor.get_text())));
            return;
        }

        let mut is_clipboard = false;
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if command && keys.just_pressed(KeyCode::C) {
                    if let Some(text) = editor.0.copy_selection() {
                        clipboard.set_text(text).unwrap();
                        return;
                    }
                }
                if command && keys.just_pressed(KeyCode::X) && !readonly {
                    if let Some(text) = editor.0.copy_selection() {
                        clipboard.set_text(text).unwrap();
                        editor.0.delete_selection();
                    }
                    is_clipboard = true;
                }
                if command && keys.just_pressed(KeyCode::V) && !readonly {
                    if let Ok(text) = clipboard.get_text() {
                        for c in text.chars() {
                            if max_chars.0 == 0 || editor.get_text().len() < max_chars.0 {
                                if c == 0xA as char {
                                    if max_lines.0 == 0
                                        || editor.0.buffer().lines.len() < max_lines.0
                                    {
                                        editor.0.action(&mut font_system.0, Action::Insert(c));
                                    }
                                } else {
                                    editor.0.action(&mut font_system.0, Action::Insert(c));
                                }
                            }
                        }
                    }
                    is_clipboard = true;
                }
            }
        }

        // fix for issue #8
        if let Some(select) = editor.0.select_opt() {
            if editor.0.cursor().line == select.line && editor.0.cursor().index == select.index {
                editor.0.set_select_opt(None);
            }
        }

        let mut is_edit = is_clipboard;
        let mut is_return = false;
        if keys.just_pressed(KeyCode::Return) && !readonly {
            is_return = true;
            if (max_lines.0 == 0 || editor.0.buffer().lines.len() < max_lines.0)
                && (max_chars.0 == 0 || editor.get_text().len() < max_chars.0)
            {
                // to have new line on wasm rather than E
                is_edit = true;
                editor.0.action(&mut font_system.0, Action::Insert('\n'));
            }
        }

        if !(is_clipboard || is_return || readonly) {
            for char_ev in char_evr.iter() {
                is_edit = true;
                if *is_deleting {
                    editor.0.action(&mut font_system.0, Action::Backspace);
                } else if max_chars.0 == 0 || editor.get_text().len() < max_chars.0 {
                    editor
                        .0
                        .action(&mut font_system.0, Action::Insert(char_ev.char));
                }
            }
        }

        if !is_edit || readonly {
            return;
        }

        evw_changed.send(CosmicTextChanged((entity, editor.get_text())));

        if let Some(last_edit_duration) = *edits_duration {
            if Duration::from_millis(now_ms as u64) - last_edit_duration
                > Duration::from_millis(150)
            {
                save_edit_history(&mut editor.0, attrs, &mut edit_history);
                *edits_duration = Some(Duration::from_millis(now_ms as u64));
            }
        } else {
            save_edit_history(&mut editor.0, attrs, &mut edit_history);
            *edits_duration = Some(Duration::from_millis(now_ms as u64));
        }
    }
}
