use bevy::prelude::*;
use cosmic_text::{Buffer, Edit, Shaping};

use crate::{
    input::{input_mouse, kb_input_text, kb_move_cursor},
    CosmicBuffer, CosmicEditor, CosmicFontSystem, DefaultAttrs, Render,
};

pub struct PasswordPlugin;

impl Plugin for PasswordPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                hide_password_text.before(input_mouse),
                restore_password_text.after(input_mouse),
            ),
        )
        .add_systems(
            Update,
            (
                hide_password_text.before(kb_move_cursor),
                restore_password_text
                    .before(kb_input_text)
                    .after(kb_move_cursor),
            ),
        )
        .add_systems(
            PostUpdate,
            (
                hide_password_text.before(Render),
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
    )>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for (mut password, mut buffer, attrs, editor_opt) in q.iter_mut() {
        if let Some(mut editor) = editor_opt {
            editor.with_buffer_mut(|buffer| {
                fn get_text(buffer: &mut Buffer) -> String {
                    let mut text = String::new();
                    let line_count = buffer.lines.len();

                    for (i, line) in buffer.lines.iter().enumerate() {
                        text.push_str(line.text());

                        if i < line_count - 1 {
                            text.push('\n');
                        }
                    }

                    text
                }

                let text = get_text(buffer);
                buffer.set_text(
                    &mut font_system,
                    password.glyph.to_string().repeat(text.len()).as_str(),
                    attrs.as_attrs(),
                    Shaping::Advanced,
                );

                for (i, c) in text.char_indices() {
                    if !text.is_char_boundary(i) || c.len_utf8() > 1 {
                        panic!("Widechars (like {c}) are not yet supported in password fields.")
                    }
                }

                password.real_text = text;
            });

            let mut cursor = editor.cursor();
            cursor.index *= password.glyph.len_utf8(); // HACK: multiply cursor position assuming no widechars are input
                                                       // TODO: Count characters until cursor and set new position accordingly,
                                                       // noting the previous location for restoring
                                                       // TODO: Look into unicode graphemes
            editor.set_cursor(cursor);

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
    )>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for (password, mut buffer, attrs, editor_opt) in q.iter_mut() {
        if let Some(mut editor) = editor_opt {
            editor.with_buffer_mut(|buffer| {
                buffer.set_text(
                    &mut font_system,
                    password.real_text.as_str(),
                    attrs.as_attrs(),
                    Shaping::Advanced,
                )
            });

            let mut cursor = editor.cursor();
            cursor.index /= password.glyph.len_utf8(); // HACK: restore cursor position assuming no widechars are input
            editor.set_cursor(cursor);

            continue;
        }

        buffer.set_text(
            &mut font_system,
            password.real_text.as_str(),
            attrs.as_attrs(),
        );
    }
}
