use crate::{buffer::BufferExtras, placeholder::Placeholder};
use bevy::prelude::*;
use cosmic_text::{Cursor, Edit, Selection, Shaping};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    input::{input_mouse, kb_input_text, kb_move_cursor},
    CosmicBuffer, CosmicEditor, CosmicFontSystem, DefaultAttrs, Render,
};

pub struct PasswordPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PasswordSet;

impl Plugin for PasswordPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, (hide_password_text.before(input_mouse),))
            .add_systems(
                Update,
                (restore_password_text
                    .before(kb_input_text)
                    .after(kb_move_cursor),),
            )
            .add_systems(
                PostUpdate,
                (
                    hide_password_text.before(Render).in_set(PasswordSet),
                    restore_password_text.after(Render),
                ),
            );
    }
}

#[derive(Component)]
pub struct Password {
    real_text: String,
    glyph: char,
}

impl Default for Password {
    fn default() -> Self {
        Self {
            real_text: Default::default(),
            glyph: '●',
        }
    }
}

fn hide_password_text(
    mut q: Query<(
        &mut Password,
        &mut CosmicBuffer,
        &DefaultAttrs,
        Option<&mut CosmicEditor>,
        Option<&Placeholder>,
    )>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for (mut password, mut buffer, attrs, editor_opt, placeholder_opt) in q.iter_mut() {
        if let Some(placeholder) = placeholder_opt {
            if placeholder.is_active() {
                continue;
            }
        }
        if let Some(mut editor) = editor_opt {
            let mut cursor = editor.cursor();
            let mut selection = editor.selection();

            editor.with_buffer_mut(|buffer| {
                let text = buffer.get_text();

                // Translate cursor to correct position for blocker glyphs
                let translate_cursor = |c: &mut Cursor| {
                    let (pre, _post) = text.split_at(c.index);
                    let graphemes = pre.graphemes(true).count();
                    c.index = graphemes * password.glyph.len_utf8();
                };

                translate_cursor(&mut cursor);

                // Translate selection cursor
                match selection {
                    Selection::None => {}
                    Selection::Line(ref mut c) => {
                        translate_cursor(c);
                    }
                    Selection::Word(ref mut c) => {
                        translate_cursor(c);
                    }
                    Selection::Normal(ref mut c) => {
                        translate_cursor(c);
                    }
                }

                // Update text to blockers
                buffer.set_text(
                    &mut font_system,
                    password
                        .glyph
                        .to_string()
                        .repeat(text.graphemes(true).count())
                        .as_str(),
                    attrs.as_attrs(),
                    Shaping::Advanced,
                );

                password.real_text = text;
            });

            editor.set_cursor(cursor);
            editor.set_selection(selection);

            continue;
        }

        let text = buffer.get_text();

        buffer.set_text(
            &mut font_system,
            password.glyph.to_string().repeat(text.len()).as_str(),
            attrs.as_attrs(),
        );
        password.real_text = text;
    }
}

fn restore_password_text(
    mut q: Query<(
        &Password,
        &mut CosmicBuffer,
        &DefaultAttrs,
        Option<&mut CosmicEditor>,
        Option<&Placeholder>,
    )>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for (password, mut buffer, attrs, editor_opt, placeholder_opt) in q.iter_mut() {
        if let Some(placeholder) = placeholder_opt {
            if placeholder.is_active() {
                continue;
            }
        }
        if let Some(mut editor) = editor_opt {
            let mut cursor = editor.cursor();
            let mut selection = editor.selection();

            editor.with_buffer_mut(|buffer| {
                let text = buffer.get_text();

                // Find cursor position and translate back to correct position in real text
                let restore_cursor = |c: &mut Cursor| {
                    let (pre, _post) = text.split_at(c.index);
                    let graphemes = pre.graphemes(true).count();
                    let mut n_i = 0;
                    if let Some((i, _)) = password.real_text.grapheme_indices(true).nth(graphemes) {
                        n_i = i;
                    } else if c.index > 0 {
                        n_i = password.real_text.len();
                    }
                    c.index = n_i;
                };

                restore_cursor(&mut cursor);

                // Translate selection cursor
                match selection {
                    Selection::None => {}
                    Selection::Line(ref mut c) => {
                        restore_cursor(c);
                    }
                    Selection::Word(ref mut c) => {
                        restore_cursor(c);
                    }
                    Selection::Normal(ref mut c) => {
                        restore_cursor(c);
                    }
                }

                buffer.set_text(
                    &mut font_system,
                    password.real_text.as_str(),
                    attrs.as_attrs(),
                    Shaping::Advanced,
                );
            });

            editor.set_cursor(cursor);
            editor.set_selection(selection);

            continue;
        }

        buffer.set_text(
            &mut font_system,
            password.real_text.as_str(),
            attrs.as_attrs(),
        );
    }
}
