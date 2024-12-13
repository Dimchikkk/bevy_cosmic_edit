#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use crate::{cosmic_edit::ScrollEnabled, prelude::*};
use bevy::ecs::{component::ComponentId, world::DeferredWorld};

pub mod click;
pub mod clipboard;
pub mod cursor_icon;
pub mod cursor_visibility;
pub mod drag;
pub mod hover;
pub mod keyboard;
pub mod scroll;

/// System set for mouse and keyboard input events. Runs in [`PreUpdate`] and [`Update`]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputSet;

pub(crate) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, scroll::scroll.in_set(InputSet))
            .add_systems(
                Update,
                (
                    keyboard::kb_move_cursor,
                    keyboard::kb_input_text,
                    clipboard::kb_clipboard,
                    (
                        cursor_icon::update_cursor_icon,
                        cursor_visibility::update_cursor_visibility,
                    ),
                )
                    .chain()
                    .in_set(InputSet),
            )
            .add_event::<hover::TextHoverIn>()
            .add_event::<hover::TextHoverOut>()
            .add_event::<CosmicTextChanged>()
            .register_type::<hover::TextHoverIn>()
            .register_type::<hover::TextHoverOut>()
            .register_type::<CosmicTextChanged>();

        #[cfg(target_arch = "wasm32")]
        {
            let (tx, rx) = crossbeam_channel::bounded::<clipboard::WasmPaste>(1);
            app.insert_resource(clipboard::WasmPasteAsyncChannel { tx, rx })
                .add_systems(Update, clipboard::poll_wasm_paste);
        }
    }
}

/// Text change events
///
/// Sent when text is changed in a cosmic buffer
/// Contains the entity on which the text was changed, and the new text as a [`String`]
#[derive(Event, Reflect, Debug)]
pub struct CosmicTextChanged(pub (Entity, String));

/// First variant is least important, last is most important
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
        Observer::new(
            click::handle_focussed_click.pipe(render_implementations::debug_error("handle_click")),
        ),
        Observer::new(
            drag::handle_dragstart.pipe(render_implementations::debug_error("handle dragstart")),
        ),
        Observer::new(drag::handle_drag_continue),
        Observer::new(drag::handle_dragend),
        Observer::new(hover::handle_hover_start),
        Observer::new(hover::handle_hover_continue),
        Observer::new(hover::handle_hover_end),
        Observer::new(cancel::handle_cancel),
    ];
    for observer in &mut observers {
        observer.watch_entity(targeted_entity);
    }
    world.commands().spawn_batch(observers);
}

// todo: avoid these warnings on ReadOnly
fn warn_no_editor_on_picking_event(job: &'static str) {
    debug!(
        note = "This is a false alarm for ReadOnly buffers",
        note = "Please only use the `InputState` component on entities with a `CosmicEditor` component",
        note = "`CosmicEditor` components should be automatically added to focussed `CosmicEditBuffer` entities",
        "Failed to get editor from picking event while {job}"
    );
}

pub mod cancel {
    use crate::prelude::*;

    use super::{warn_no_editor_on_picking_event, InputState};

    impl InputState {
        /// `Cancel` event handler
        pub fn cancel(&mut self) {
            trace!("Cancelling a pointer");
            *self = InputState::Idle;
        }
    }

    pub(super) fn handle_cancel(
        trigger: Trigger<Pointer<Cancel>>,
        mut editor: Query<&mut InputState, With<CosmicEditBuffer>>,
    ) {
        let Ok(mut input_state) = editor.get_mut(trigger.target) else {
            warn_no_editor_on_picking_event("handling cursor `Cancel` event");
            return;
        };

        input_state.cancel();
    }
}
