use bevy::prelude::*;
use cosmic_text::{Edit, Editor};

use crate::{CosmicBuffer, CosmicEditor};

/// Resource struct that keeps track of the currently active editor entity.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct FocusedWidget(pub Option<Entity>);

pub(crate) fn add_editor_to_focused(
    mut commands: Commands,
    active_editor: Res<FocusedWidget>,
    q: Query<&CosmicBuffer, Without<CosmicEditor>>,
) {
    if let Some(e) = active_editor.0 {
        let Ok(b) = q.get(e) else {
            return;
        };
        let mut editor = Editor::new(b.0.clone());
        editor.set_redraw(true);
        commands.entity(e).insert(CosmicEditor(editor));
    }
}

pub(crate) fn drop_editor_unfocused(
    mut commands: Commands,
    active_editor: Res<FocusedWidget>,
    mut q: Query<(Entity, &mut CosmicBuffer), With<CosmicEditor>>,
) {
    if active_editor.0 == None {
        for (e, mut b) in q.iter_mut() {
            b.set_redraw(true);
            commands.entity(e).remove::<CosmicEditor>();
        }
    } else if let Some(focused) = active_editor.0 {
        for (e, mut b) in q.iter_mut() {
            if e != focused {
                b.set_redraw(true);
                commands.entity(e).remove::<CosmicEditor>();
            }
        }
    }
}
