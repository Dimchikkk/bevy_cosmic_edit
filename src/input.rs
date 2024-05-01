#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use crate::*;
use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};
use cosmic_text::{Action, Cursor, Edit, Motion, Selection};

#[cfg(target_arch = "wasm32")]
use crate::DefaultAttrs;
#[cfg(target_arch = "wasm32")]
use bevy::tasks::AsyncComputeTaskPool;
#[cfg(target_arch = "wasm32")]
use js_sys::Promise;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

/// System set for mouse and keyboard input events. Runs in [`PreUpdate`] and [`Update`]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputSet;

pub(crate) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, input_mouse.in_set(InputSet))
            .add_systems(
                Update,
                (kb_move_cursor, kb_input_text, kb_clipboard)
                    .chain()
                    .in_set(InputSet),
            )
            .insert_resource(ClickTimer(Timer::from_seconds(0.5, TimerMode::Once)));
    }
}

/// Timer for double / triple clicks
#[derive(Resource)]
pub struct ClickTimer(pub Timer);

// TODO: hide this behind #cfg wasm, depends on wasm having own copy/paste fn
/// Crossbeam channel struct for Wasm clipboard data
#[allow(dead_code)]
pub struct WasmPaste {
    text: String,
    entity: Entity,
}

/// Async channel for receiving from the clipboard in Wasm
#[derive(Resource)]
pub struct WasmPasteAsyncChannel {
    pub tx: crossbeam_channel::Sender<WasmPaste>,
    pub rx: crossbeam_channel::Receiver<WasmPaste>,
}

pub(crate) fn input_mouse(
    windows: Query<&Window, With<PrimaryWindow>>,
    active_editor: Res<FocusedWidget>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut editor_q: Query<(
        &mut CosmicEditor,
        &GlobalTransform,
        &CosmicTextAlign,
        Entity,
        &XOffset,
        &mut Sprite,
    )>,
    node_q: Query<(&Node, &GlobalTransform, &CosmicSource)>,
    mut font_system: ResMut<CosmicFontSystem>,
    mut scroll_evr: EventReader<MouseWheel>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut click_timer: ResMut<ClickTimer>,
    mut click_count: Local<usize>,
    time: Res<Time>,
    evr_mouse_motion: EventReader<MouseMotion>,
) {
    click_timer.0.tick(time.delta());

    let Some(active_editor_entity) = active_editor.0 else {
        return;
    };

    if click_timer.0.finished() || !evr_mouse_motion.is_empty() {
        *click_count = 0;
    }

    if buttons.just_pressed(MouseButton::Left) {
        click_timer.0.reset();
        *click_count += 1;
    }

    if *click_count > 3 {
        *click_count = 0;
    }

    let Ok(primary_window) = windows.get_single() else {
        return;
    };

    let scale_factor = primary_window.scale_factor();
    let Some((camera, camera_transform)) = camera_q.iter().find(|(c, _)| c.is_active) else {
        return;
    };

    if let Ok((mut editor, sprite_transform, text_position, entity, x_offset, sprite)) =
        editor_q.get_mut(active_editor_entity)
    {
        let buffer = editor.with_buffer(|b| b.clone());

        let mut is_ui_node = false;
        let mut transform = sprite_transform;
        let (mut width, mut height) =
            (sprite.custom_size.unwrap().x, sprite.custom_size.unwrap().y);

        // TODO: this is bad loop nesting, rethink system with relationships in mind
        for (node, node_transform, source) in node_q.iter() {
            if source.0 != entity {
                continue;
            }
            is_ui_node = true;
            transform = node_transform;
            width = node.size().x;
            height = node.size().y;
        }

        let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

        // if shift key is pressed
        let already_has_selection = editor.selection() != Selection::None;
        if shift && !already_has_selection {
            let cursor = editor.cursor();
            editor.set_selection(Selection::Normal(cursor));
        }

        let (padding_x, padding_y) = match text_position {
            CosmicTextAlign::Center { padding: _ } => (
                get_x_offset_center(width * scale_factor, &buffer),
                get_y_offset_center(height * scale_factor, &buffer),
            ),
            CosmicTextAlign::TopLeft { padding } => (*padding, *padding),
            CosmicTextAlign::Left { padding } => (
                *padding,
                get_y_offset_center(height * scale_factor, &buffer),
            ),
        };
        let point = |node_cursor_pos: (f32, f32)| {
            (
                (node_cursor_pos.0 * scale_factor) as i32 - padding_x,
                (node_cursor_pos.1 * scale_factor) as i32 - padding_y,
            )
        };

        if buttons.just_pressed(MouseButton::Left) {
            editor.cursor_visible = true;
            editor.cursor_timer.reset();

            if let Some(node_cursor_pos) = get_node_cursor_pos(
                primary_window,
                transform,
                (width, height),
                is_ui_node,
                camera,
                camera_transform,
            ) {
                let (mut x, y) = point(node_cursor_pos);
                x += x_offset.left as i32;
                if shift {
                    editor.action(&mut font_system.0, Action::Drag { x, y });
                } else {
                    match *click_count {
                        1 => {
                            editor.action(&mut font_system.0, Action::Click { x, y });
                        }
                        2 => {
                            // select word
                            editor.action(&mut font_system.0, Action::Motion(Motion::LeftWord));
                            let cursor = editor.cursor();
                            editor.set_selection(Selection::Normal(cursor));
                            editor.action(&mut font_system.0, Action::Motion(Motion::RightWord));
                        }
                        3 => {
                            // select paragraph
                            editor
                                .action(&mut font_system.0, Action::Motion(Motion::ParagraphStart));
                            let cursor = editor.cursor();
                            editor.set_selection(Selection::Normal(cursor));
                            editor.action(&mut font_system.0, Action::Motion(Motion::ParagraphEnd));
                        }
                        _ => {}
                    }
                }
            }
            return;
        }

        if buttons.pressed(MouseButton::Left) && *click_count == 0 {
            if let Some(node_cursor_pos) = get_node_cursor_pos(
                primary_window,
                transform,
                (width, height),
                is_ui_node,
                camera,
                camera_transform,
            ) {
                let (mut x, y) = point(node_cursor_pos);
                x += x_offset.left as i32;
                if active_editor.is_changed() && !shift {
                    editor.action(&mut font_system.0, Action::Click { x, y });
                } else {
                    editor.action(&mut font_system.0, Action::Drag { x, y });
                }
            }
            return;
        }

        for ev in scroll_evr.read() {
            match ev.unit {
                MouseScrollUnit::Line => {
                    editor.action(
                        &mut font_system.0,
                        Action::Scroll {
                            lines: -ev.y as i32,
                        },
                    );
                }
                MouseScrollUnit::Pixel => {
                    let line_height = buffer.metrics().line_height;
                    editor.action(
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

pub fn kb_move_cursor(
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
    mut char_evr: EventReader<ReceivedCharacter>,
    mut cosmic_edit_query: Query<(
        &mut CosmicEditor,
        &mut CosmicBuffer,
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
            #[cfg(target_arch = "wasm32")]
            editor.action(&mut font_system.0, Action::Backspace);
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
                } else if !command && (max_chars.0 == 0 || buffer.get_text().len() < max_chars.0) {
                    let b = char_ev.char.as_bytes();
                    for c in b {
                        let c: char = (*c).into();
                        editor.action(&mut font_system.0, Action::Insert(c));
                    }
                }
            }
        }

        if !is_edit {
            return;
        }

        evw_changed.send(CosmicTextChanged((entity, buffer.get_text())));
    }
}

pub fn kb_clipboard(
    active_editor: Res<FocusedWidget>,
    keys: Res<ButtonInput<KeyCode>>,
    mut evw_changed: EventWriter<CosmicTextChanged>,
    mut font_system: ResMut<CosmicFontSystem>,
    mut cosmic_edit_query: Query<(
        &mut CosmicEditor,
        &mut CosmicBuffer,
        &MaxLines,
        &MaxChars,
        Entity,
        Option<&ReadOnly>,
    )>,
    _channel: Option<Res<WasmPasteAsyncChannel>>,
) {
    let Some(active_editor_entity) = active_editor.0 else {
        return;
    };

    if let Ok((mut editor, buffer, max_lines, max_chars, entity, readonly_opt)) =
        cosmic_edit_query.get_mut(active_editor_entity)
    {
        let command = keypress_command(&keys);

        let readonly = readonly_opt.is_some();

        let mut is_clipboard = false;
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if command && keys.just_pressed(KeyCode::KeyC) {
                    if let Some(text) = editor.copy_selection() {
                        clipboard.set_text(text).unwrap();
                        return;
                    }
                }
                if command && keys.just_pressed(KeyCode::KeyX) && !readonly {
                    if let Some(text) = editor.copy_selection() {
                        clipboard.set_text(text).unwrap();
                        editor.delete_selection();
                    }
                    is_clipboard = true;
                }
                if command && keys.just_pressed(KeyCode::KeyV) && !readonly {
                    if let Ok(text) = clipboard.get_text() {
                        for c in text.chars() {
                            if max_chars.0 == 0 || buffer.get_text().len() < max_chars.0 {
                                if c == 0xA as char {
                                    if max_lines.0 == 0 || buffer.lines.len() < max_lines.0 {
                                        editor.action(&mut font_system.0, Action::Insert(c));
                                    }
                                } else {
                                    editor.action(&mut font_system.0, Action::Insert(c));
                                }
                            }
                        }
                    }
                    is_clipboard = true;
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            if command && keys.just_pressed(KeyCode::KeyC) {
                if let Some(text) = editor.copy_selection() {
                    write_clipboard_wasm(text.as_str());
                    return;
                }
            }

            if command && keys.just_pressed(KeyCode::KeyX) && !readonly {
                if let Some(text) = editor.copy_selection() {
                    write_clipboard_wasm(text.as_str());
                    editor.delete_selection();
                }
                is_clipboard = true;
            }
            if command && keys.just_pressed(KeyCode::KeyV) && !readonly {
                let tx = _channel.unwrap().tx.clone();
                let _task = AsyncComputeTaskPool::get().spawn(async move {
                    let promise = read_clipboard_wasm();

                    let result = JsFuture::from(promise).await;

                    if let Ok(js_text) = result {
                        if let Some(text) = js_text.as_string() {
                            let _ = tx.try_send(WasmPaste { text, entity });
                        }
                    }
                });

                return;
            }
        }

        if !is_clipboard {
            return;
        }

        evw_changed.send(CosmicTextChanged((entity, buffer.get_text())));
    }
}

fn keypress_command(keys: &ButtonInput<KeyCode>) -> bool {
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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn write_clipboard_wasm(text: &str) {
    let clipboard = web_sys::window()
        .unwrap()
        .navigator()
        .clipboard()
        .expect("Clipboard not found!");
    let _result = clipboard.write_text(text);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn read_clipboard_wasm() -> Promise {
    let clipboard = web_sys::window()
        .unwrap()
        .navigator()
        .clipboard()
        .expect("Clipboard not found!");
    clipboard.read_text()
}

#[cfg(target_arch = "wasm32")]
pub fn poll_wasm_paste(
    channel: Res<WasmPasteAsyncChannel>,
    mut editor_q: Query<
        (
            &mut CosmicEditor,
            &mut CosmicBuffer,
            &crate::DefaultAttrs,
            &MaxChars,
            &MaxChars,
        ),
        Without<ReadOnly>,
    >,
    mut evw_changed: EventWriter<CosmicTextChanged>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    let inlet = channel.rx.try_recv();
    match inlet {
        Ok(inlet) => {
            let entity = inlet.entity;
            if let Ok((mut editor, mut buffer, attrs, max_chars, max_lines)) =
                editor_q.get_mut(entity)
            {
                let text = inlet.text;
                let attrs = &attrs.0;
                for c in text.chars() {
                    if max_chars.0 == 0 || buffer.get_text().len() < max_chars.0 {
                        if c == 0xA as char {
                            if max_lines.0 == 0 || buffer.lines.len() < max_lines.0 {
                                editor.action(&mut font_system.0, Action::Insert(c));
                            }
                        } else {
                            editor.action(&mut font_system.0, Action::Insert(c));
                        }
                    }
                }

                evw_changed.send(CosmicTextChanged((entity, buffer.get_text())));
            }
        }
        Err(_) => {}
    }
}
