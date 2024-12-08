use crate::{prelude::*, CosmicTextChanged, MaxChars, MaxLines};

#[cfg(target_arch = "wasm32")]
use bevy::tasks::AsyncComputeTaskPool;
use cosmic_text::{Action, Edit};
#[cfg(target_arch = "wasm32")]
#[allow(unused_imports)]
use js_sys::Promise;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

// TODO: hide this behind #cfg wasm, depends on wasm having own copy/paste fn
/// Crossbeam channel struct for Wasm clipboard data
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) struct WasmPaste {
    text: String,
    entity: Entity,
}

/// Async channel for receiving from the clipboard in Wasm
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[derive(Resource)]
pub(crate) struct WasmPasteAsyncChannel {
    pub tx: crossbeam_channel::Sender<WasmPaste>,
    pub rx: crossbeam_channel::Receiver<WasmPaste>,
}

pub(crate) fn kb_clipboard(
    active_editor: Res<FocusedWidget>,
    keys: Res<ButtonInput<KeyCode>>,
    mut evw_changed: EventWriter<CosmicTextChanged>,
    mut font_system: ResMut<CosmicFontSystem>,
    mut cosmic_edit_query: Query<(
        &mut CosmicEditor,
        &mut CosmicEditBuffer,
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
        let command = crate::input::keyboard::keypress_command(&keys);

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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn write_clipboard_wasm(text: &str) {
    let clipboard = web_sys::window().unwrap().navigator().clipboard();
    let _result = clipboard.write_text(text);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn read_clipboard_wasm() -> Promise {
    let clipboard = web_sys::window().unwrap().navigator().clipboard();
    clipboard.read_text()
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn poll_wasm_paste(
    channel: Res<WasmPasteAsyncChannel>,
    mut editor_q: Query<
        (
            &mut CosmicEditor,
            &mut CosmicEditBuffer,
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
            if let Ok((mut editor, buffer, max_chars, max_lines)) = editor_q.get_mut(entity) {
                let text = inlet.text;
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
