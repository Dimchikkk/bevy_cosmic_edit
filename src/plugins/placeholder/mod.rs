use bevy::prelude::*;
use cosmic_text::{Attrs, Edit};

use crate::{CosmicBuffer, CosmicEditor, CosmicFontSystem, DefaultAttrs};

#[derive(Component)]
pub struct Placeholder {
    pub text: &'static str,
    pub attrs: Attrs<'static>,
    active: bool,
}

impl Placeholder {
    pub fn new(text: impl Into<&'static str>, attrs: Attrs<'static>) -> Self {
        Self {
            active: false,
            text: text.into(),
            attrs,
        }
    }
}

pub struct PlaceholderPlugin;

impl Plugin for PlaceholderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (add_placeholder_to_buffer, empty_editor_buffer_on_focus),
        );
    }
}

fn add_placeholder_to_buffer(
    mut q: Query<(&mut CosmicBuffer, &mut Placeholder)>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for (mut buffer, mut placeholder) in q.iter_mut() {
        if buffer.get_text().is_empty() {
            buffer.set_text(&mut font_system, placeholder.text, placeholder.attrs);
            placeholder.active = true;
        }
    }
}

fn empty_editor_buffer_on_focus(
    mut q: Query<(&mut CosmicEditor, &mut Placeholder, &DefaultAttrs), Added<CosmicEditor>>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    // TODO: In order to hold placeholder until input, move XOffset to 0 and empty buffer on input
    for (mut editor, mut placeholder, attrs) in q.iter_mut() {
        if placeholder.active {
            placeholder.active = false;
            editor.with_buffer_mut(|b| {
                b.set_text(
                    &mut font_system,
                    "",
                    attrs.0.as_attrs(),
                    cosmic_text::Shaping::Advanced,
                );
            });
            editor.set_cursor(cosmic_text::Cursor::new(0, 0));
        }
    }
}
