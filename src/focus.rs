use crate::prelude::*;
use cosmic_text::{Edit, Editor};

/// System set for focus systems. Runs in `PostUpdate`
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FocusSet;

pub(crate) struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (drop_editor_unfocused, add_editor_to_focused)
                .chain()
                .in_set(FocusSet),
        )
        .init_resource::<FocusedWidget>()
        .register_type::<FocusedWidget>();
    }
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
        let Ok(CosmicEditBuffer(b)) = q.get(e) else {
            return;
        };
        let mut editor = Editor::new(b.clone());
        editor.set_redraw(true);
        trace!("Adding editor to focused widget");
        commands.entity(e).insert(CosmicEditor::new(editor));
    }
}

/// Removes [`CosmicEditor`]
pub(crate) fn drop_editor_unfocused(
    mut commands: Commands,
    active_editor: Res<FocusedWidget>,
    mut q: Query<(Entity, &mut CosmicEditBuffer, &CosmicEditor)>,
) {
    if active_editor.0.is_none() {
        for (e, mut b, editor) in q.iter_mut() {
            b.lines = editor.with_buffer(|buf| buf.lines.clone());
            b.set_redraw(true);
            trace!("Removing editor from all entities as there is no focussed widget",);
            commands.entity(e).remove::<CosmicEditor>();
        }
    } else if let Some(focused) = active_editor.0 {
        for (e, mut b, editor) in q.iter_mut() {
            if e != focused {
                b.lines = editor.with_buffer(|buf| buf.lines.clone());
                b.set_redraw(true);
                trace!("Removing editor from entity as its not focussed anymore",);
                commands.entity(e).remove::<CosmicEditor>();
            }
        }
    }
}
