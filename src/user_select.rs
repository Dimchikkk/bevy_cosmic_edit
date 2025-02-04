use crate::{input::InputSet, prelude::*};
use cosmic_text::Edit;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, clear_selection.after(InputSet));
}

/// Tag component to disable user selection
/// Like CSS `user-select: none` <https://developer.mozilla.org/en-US/docs/Web/CSS/user-select>
#[derive(Component, Default)]
pub struct UserSelectNone;

fn clear_selection(mut q: Query<&mut CosmicEditor, With<UserSelectNone>>) {
    for mut editor in q.iter_mut() {
        editor.set_selection(cosmic_text::Selection::None);
    }
}
