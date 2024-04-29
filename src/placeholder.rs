use crate::{buffer::BufferExtras, input::InputSet, render::RenderSet};
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

    pub fn is_active(&self) -> bool {
        self.active
    }
}

pub struct PlaceholderPlugin;

impl Plugin for PlaceholderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                add_placeholder_to_buffer,
                add_placeholder_to_editor,
                move_cursor_to_start_of_placeholder,
                remove_placeholder_on_input,
            )
                .chain()
                .after(InputSet)
                .before(RenderSet),
        );
    }
}

fn add_placeholder_to_buffer(
    mut q: Query<(&mut CosmicBuffer, &mut Placeholder)>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for (mut buffer, mut placeholder) in q.iter_mut() {
        if placeholder.active {
            return;
        }

        if buffer.get_text().is_empty() {
            buffer.set_text(&mut font_system, placeholder.text, placeholder.attrs);
            placeholder.active = true;
        }
    }
}

fn add_placeholder_to_editor(
    mut q: Query<(&mut CosmicEditor, &mut Placeholder)>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for (mut editor, mut placeholder) in q.iter_mut() {
        if placeholder.active {
            // PERF: Removing this guard fixes single char placeholder deletion
            // BUT makes the check and buffer update run every frame
            // return;
        }

        editor.with_buffer_mut(|buffer| {
            if buffer.lines.len() > 1 {
                return;
            }

            if buffer.lines[0].clone().into_text().is_empty() {
                buffer.set_text(
                    &mut font_system,
                    placeholder.text,
                    placeholder.attrs,
                    cosmic_text::Shaping::Advanced,
                );
                placeholder.active = true;
                buffer.set_redraw(true);
            }
        })
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

        let mut lines = 0;

        let last_line = editor.with_buffer_mut(|b| {
            lines = b.lines.len();

            if lines > 1 {
                let mut full_text: String = b
                    .lines
                    .iter()
                    .map(|l| {
                        let mut s = l.clone().into_text().replace(placeholder.text, "");
                        // Extra newline on enter to prevent reading as an empty buffer
                        s.push('\n');
                        s
                    })
                    .collect();

                if lines > 2 {
                    // for pasted text, remove trailing newline
                    full_text = full_text
                        .strip_suffix('\n')
                        .expect("oop something broke in multiline placeholder removal")
                        .to_string();
                }

                b.set_text(
                    &mut font_system,
                    full_text.as_str(),
                    attrs.0.as_attrs(),
                    cosmic_text::Shaping::Advanced,
                );

                let last_line = full_text.lines().last();

                return last_line.map(|line| line.to_string());
            }

            let single_line = b.lines[0].clone().into_text().replace(placeholder.text, "");

            if single_line.is_empty() {
                return None;
            }

            {
                // begin hacky fix for delete key in empty placeholder widget

                let p = placeholder
                    .text
                    .chars()
                    .next()
                    .expect("Couldn't get first char of placeholder.");

                let laceholder = placeholder
                    .text
                    .strip_prefix(p)
                    .expect("Couldn't remove first char of placeholder.");

                if single_line.as_str() == laceholder {
                    b.set_text(
                        &mut font_system,
                        placeholder.text,
                        placeholder.attrs,
                        cosmic_text::Shaping::Advanced,
                    );
                    return None;
                }
            } // end hacky fix

            b.set_text(
                &mut font_system,
                single_line.as_str(),
                attrs.0.as_attrs(),
                cosmic_text::Shaping::Advanced,
            );

            Some(single_line)
        });

        let Some(last_line) = last_line else {
            return;
        };

        editor.set_cursor(cosmic_text::Cursor::new(
            (lines - 1).max(0),
            last_line.bytes().len(),
        ));

        placeholder.active = false;
    }
}
