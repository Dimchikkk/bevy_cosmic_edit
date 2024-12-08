#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use crate::{
    cosmic_edit::ScrollEnabled,
    double_click::{ClickCount, ClickState},
    prelude::*,
    render_implementations::RelativeQuery,
};
use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    input::mouse::{MouseScrollUnit, MouseWheel},
};
use cosmic_text::{Action, Edit, Motion, Selection};

pub(crate) mod clipboard;
pub(crate) mod hover;
pub(crate) mod keyboard;
// mod click;
pub(crate) mod drag;

/// System set for mouse and keyboard input events. Runs in [`PreUpdate`] and [`Update`]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputSet;

pub(crate) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, scroll.in_set(InputSet))
            .add_systems(
                Update,
                (
                    keyboard::kb_move_cursor,
                    keyboard::kb_input_text,
                    clipboard::kb_clipboard,
                )
                    .chain()
                    .in_set(InputSet),
            );

        #[cfg(target_arch = "wasm32")]
        {
            let (tx, rx) = crossbeam_channel::bounded::<WasmPaste>(1);
            app.insert_resource(WasmPasteAsyncChannel { tx, rx })
                .add_systems(Update, poll_wasm_paste);
        }
    }
}

#[derive(Component, Default, Debug)]
#[require(ScrollEnabled)]
#[component(on_add = add_event_handlers)]
pub(crate) enum InputState {
    #[default]
    Idle,
    Hovering,
    Dragging {
        initial_buffer_coord: Vec2,
    },
}

fn add_event_handlers(
    mut world: DeferredWorld,
    targeted_entity: Entity,
    _component_id: ComponentId,
) {
    let mut observers = [
        Observer::new(handle_click),
        Observer::new(drag::handle_dragstart),
        Observer::new(drag::handle_dragend),
        Observer::new(drag::handle_drag),
    ];
    for observer in &mut observers {
        observer.watch_entity(targeted_entity);
    }
    world.commands().spawn_batch(observers);
}

fn warn_no_editor_on_picking_event() {
    warn!(
        message = "Failed to get editor from picking event",
        note = "Please only use the `InputState` component on entities with a `CosmicEditor` component",
        note = "`CosmicEditor` components should be automatically added to focussed `CosmicEditBuffer` entities"
    );
}

fn handle_click(
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
            // select word
            editor.action(Action::Motion(Motion::LeftWord));
            let cursor = editor.cursor();
            editor.set_selection(Selection::Normal(cursor));
            editor.action(Action::Motion(Motion::RightWord));
        }
        ClickCount::Triple => {
            // select paragraph
            editor.action(Action::Motion(Motion::ParagraphStart));
            let cursor = editor.cursor();
            editor.set_selection(Selection::Normal(cursor));
            editor.action(Action::Motion(Motion::ParagraphEnd));
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
