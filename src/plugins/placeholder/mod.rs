use bevy::prelude::*;
use cosmic_text::{Attrs, Edit};

use crate::{CosmicBuffer, CosmicEditor, CosmicFontSystem, CosmicTextChanged, DefaultAttrs};

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
            (
                add_placeholder_to_buffer,
                move_cursor_to_start_of_placeholder,
                remove_placeholder_on_input,
            ),
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

fn move_cursor_to_start_of_placeholder(mut q: Query<(&mut CosmicEditor, &mut Placeholder)>) {
    for (mut editor, placeholder) in q.iter_mut() {
        if placeholder.active {
            editor.set_cursor(cosmic_text::Cursor::new(0, 0));
        }
    }
}

fn remove_placeholder_on_input(
    mut q: Query<(&mut CosmicEditor, &mut Placeholder, &DefaultAttrs)>,
    evr: EventReader<CosmicTextChanged>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for (mut editor, mut placeholder, attrs) in q.iter_mut() {
        if !placeholder.active {
            return;
        }
        if evr.is_empty() {
            return;
        }

        editor.with_buffer_mut(|b| {
            let new_string = b.lines[0].clone().into_text().replace(placeholder.text, "");
            b.set_text(
                &mut font_system,
                new_string.as_str(),
                attrs.0.as_attrs(),
                cosmic_text::Shaping::Advanced,
            );
        });
        // TODO: multi byte char test
        editor.set_cursor(cosmic_text::Cursor::new(0, 1));

        placeholder.active = false;
    }
}
