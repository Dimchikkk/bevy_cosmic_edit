#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use crate::{
    cosmic_edit::{MaxChars, MaxLines, ReadOnly, ScrollEnabled},
    events::CosmicTextChanged,
    prelude::*,
    render::WidgetBufferCoordTransformation,
    CosmicTextAlign, CosmicWidgetSize,
};
use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    input::{
        keyboard::{Key, KeyboardInput},
        mouse::{MouseScrollUnit, MouseWheel},
    },
};
use cosmic_text::{Action, BorrowedWithFontSystem, Cursor, Edit, Motion, Selection};

#[cfg(target_arch = "wasm32")]
use bevy::tasks::AsyncComputeTaskPool;
#[cfg(target_arch = "wasm32")]
#[allow(unused_imports)]
use js_sys::Promise;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

/// System set for mouse and keyboard input events. Runs in [`PreUpdate`] and [`Update`]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputSet {
    PreUpdate,
    Update,
}

pub(crate) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, scroll.in_set(InputSet::PreUpdate))
            .add_systems(
                Update,
                (kb_move_cursor, kb_input_text, kb_clipboard)
                    .chain()
                    .in_set(InputSet::Update),
            )
            .insert_resource(ClickTimer(Timer::from_seconds(0.5, TimerMode::Once)));

        #[cfg(target_arch = "wasm32")]
        {
            let (tx, rx) = crossbeam_channel::bounded::<WasmPaste>(1);
            app.insert_resource(WasmPasteAsyncChannel { tx, rx })
                .add_systems(Update, poll_wasm_paste);
        }
    }
}

/// Timer for double / triple clicks
#[derive(Resource)]
pub(crate) struct ClickTimer(pub(crate) Timer);

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

#[derive(Component, Default)]
#[require(ScrollEnabled)]
#[component(on_add = add_event_handlers)]
pub struct InputState;

fn add_event_handlers(
    mut world: DeferredWorld,
    targeted_entity: Entity,
    _component_id: ComponentId,
) {
    let mut observers = [
        picking_event_sprite(handle_click_sprite),
        Observer::new(handle_drag),
    ];
    for observer in &mut observers {
        observer.watch_entity(targeted_entity);
    }
    world.commands().spawn_batch(observers);
}

struct SpritePickingEvent<'s, E>
where
    E: Reflect + std::fmt::Debug + Clone + HitDataEvent,
{
    event: &'s E,
    buffer_coord: Vec2,
    editor: BorrowedWithFontSystem<'s, cosmic_text::Editor<'static>>,
}

trait HitDataEvent {
    fn hit(&self) -> &bevy::picking::backend::HitData;
}

impl HitDataEvent for bevy::picking::events::Click {
    fn hit(&self) -> &bevy::picking::backend::HitData {
        &self.hit
    }
}

fn picking_event_sprite<E>(
    mut callback: impl FnMut(SpritePickingEvent<E>) + Send + Sync + 'static,
) -> Observer
where
    E: Reflect + std::fmt::Debug + Clone + HitDataEvent,
{
    Observer::new(
        move |mut trigger: Trigger<Pointer<E>>,
              mut editor: Query<(
            &mut InputState,
            &mut CosmicEditor,
            &GlobalTransform,
            &CosmicTextAlign,
            CosmicWidgetSize,
        )>,
              mut font_system: ResMut<CosmicFontSystem>| {
            trigger.propagate(false);

            let font_system = &mut font_system.0;
            let target = trigger.target;
            let (input_state, mut editor, global_transform, text_align, size) =
                editor.get_mut(target).unwrap();
            let event = &trigger.event().event;

            if event.hit().normal != Some(Vec3::Z) {
                warn!(?event, "Normal is not out of screen, skipping");
                return;
            }

            let Some(world_position) = event.hit().position else {
                return;
            };

            let position_transform =
                GlobalTransform::from(Transform::from_translation(world_position));
            let relative_transform = position_transform.reparented_to(global_transform);
            let relative_position = relative_transform.translation.xy();

            let Ok(render_target_size) = size.logical_size() else {
                return;
            };
            let buffer_size = editor.with_buffer_mut(|b| b.borrow_with(font_system).logical_size());
            let transformation = WidgetBufferCoordTransformation::new(
                text_align.vertical,
                render_target_size,
                buffer_size,
            );
            // .xy swizzle depends on normal vector being perfectly out of screen
            let buffer_coord = transformation.widget_origined_to_buffer_topleft(relative_position);
            let Some(cursor_hit) =
                editor.with_buffer(|buffer| buffer.hit(buffer_coord.x, buffer_coord.y))
            else {
                return;
            };

            let data = SpritePickingEvent {
                event,
                buffer_coord,
                editor: editor.borrow_with(font_system),
            };

            callback(data);
        },
    )
}

fn handle_click_sprite(sprite_picking_event: SpritePickingEvent<'_, Click>) {
    let SpritePickingEvent {
        event: click,
        mut editor,
        buffer_coord,
    } = sprite_picking_event;

    editor.action(Action::Click {
        x: buffer_coord.x as i32,
        y: buffer_coord.y as i32,
    });
}

fn handle_drag(trigger: Trigger<Pointer<Drag>>) {
    // debug!(?trigger, "drag");
}

// let (padding_x, padding_y) = match text_position {
//             CosmicTextAlign::Center { padding: _ } => (
//                 get_x_offset_center(width * scale_factor, &buffer),
//                 get_y_offset_center(height * scale_factor, &buffer),
//             ),
//             CosmicTextAlign::TopLeft { padding } => (*padding, *padding),
//             CosmicTextAlign::Left { padding } => (
//                 *padding,
//                 get_y_offset_center(height * scale_factor, &buffer),
//             ),
//         };
//         // Converts a node-relative space coordinate to an i32 glyph position
//         let glyph_coord = |node_cursor_pos: Vec2| {
//             (
//                 (node_cursor_pos.x * scale_factor) as i32 - padding_x + x_offset.left as i32,
//                 (node_cursor_pos.y * scale_factor) as i32 - padding_y,
//             )
//         };

//         if click_state == ClickState::Single {
//             editor.cursor_visible = true;
//             editor.cursor_timer.reset();

//             if let Some(node_cursor_pos) = crate::render_targets::get_node_cursor_pos(
//                 primary_window,
//                 transform,
//                 Vec2::new(width, height),
//                 source_type,
//                 camera,
//                 camera_transform,
//             ) {
//                 let (x, y) = glyph_coord(node_cursor_pos);

pub(crate) fn scroll(
    mut editor: Query<(&mut CosmicEditor, &ScrollEnabled)>,
    mut font_system: ResMut<CosmicFontSystem>,
    mut scroll_evr: EventReader<MouseWheel>,
) {
    for (mut editor, scroll_enabled) in editor.iter_mut() {
        let buffer = editor.with_buffer(|b| b.clone());
        if scroll_enabled.should_scroll() {
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
