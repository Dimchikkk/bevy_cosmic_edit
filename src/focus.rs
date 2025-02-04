//! Manages the [`FocusedWidget`] resource
//!
//! Makes sure that the focused widget has a [`CosmicEditor`] component
//! if its focused

use crate::prelude::*;

/// System set for focus systems. Runs in `PostUpdate`
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FocusSet;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (drop_editor_unfocused, add_editor_to_focused)
            .chain()
            .in_set(FocusSet),
    )
    .init_resource::<FocusedWidget>()
    .register_type::<FocusedWidget>();
}

/// Resource struct that keeps track of the currently active editor entity.
///
/// The focussed entity must have a [`CosmicEditBuffer`], and should have a
/// [`CosmicEditor`] component as well if it can be mutated (i.e. isn't [`Readonly`]).
#[derive(Resource, Reflect, Default, Deref, DerefMut)]
#[reflect(Resource)]
pub struct FocusedWidget(pub Option<Entity>);

/// Adds [`CosmicEditor`] by copying from existing [`CosmicEditBuffer`].
pub(crate) fn add_editor_to_focused(
    mut commands: Commands,
    active_editor: Res<FocusedWidget>,
    q: Query<&CosmicEditBuffer, Without<CosmicEditor>>,
) {
    if let Some(e) = active_editor.0 {
        let Ok(buffer) = q.get(e) else {
            return;
        };
        let editor = CosmicEditor::clone_from_buffer(buffer);
        trace!("Adding editor to focused widget");
        commands.entity(e).insert(editor);
    }
}

/// Removes [`CosmicEditor`]
pub(crate) fn drop_editor_unfocused(
    mut commands: Commands,
    active_editor: Res<FocusedWidget>,
    mut q: Query<(Entity, &mut CosmicEditBuffer, &CosmicEditor)>,
) {
    match active_editor.0 {
        None => {
            for (e, mut buffer, editor) in q.iter_mut() {
                *buffer = CosmicEditBuffer::from_downgrading_editor(editor);
                trace!("Removing editor from all entities as there is no focussed widget",);
                commands.entity(e).remove::<CosmicEditor>();
            }
        }
        Some(focused) => {
            for (e, mut b, editor) in q.iter_mut() {
                if e != focused {
                    *b = CosmicEditBuffer::from_downgrading_editor(editor);
                    trace!("Removing editor from entity as its not focussed anymore",);
                    commands.entity(e).remove::<CosmicEditor>();
                }
            }
        }
    }
}

/// Placed as on_remove hook for [`CosmicEditBuffer`] and [`CosmicEditor`]
pub(crate) fn remove_focus_from_entity(
    mut world: bevy::ecs::world::DeferredWorld,
    entity: Entity,
    _: bevy::ecs::component::ComponentId,
) {
    if let Some(mut focused_widget) = world.get_resource_mut::<FocusedWidget>() {
        if let Some(focused) = focused_widget.0 {
            if focused == entity {
                focused_widget.0 = None;
            }
        }
    }
}
