use bevy::prelude::*;
use cosmic_text::{Buffer, Edit, Shaping};
use unicode_segmentation::UnicodeSegmentation;

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

// TODO: impl this on buffer
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
            let mut cursor = editor.cursor();

            editor.with_buffer_mut(|buffer| {
                let text = get_text(buffer);

                let (pre, _post) = text.split_at(cursor.index);

                let graphemes = pre.graphemes(true).count();

                cursor.index = graphemes * password.glyph.len_utf8();

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
            let mut cursor = editor.cursor();
            let mut index = 0;

            editor.with_buffer_mut(|buffer| {
                let text = get_text(buffer);
                let (pre, _post) = text.split_at(cursor.index);

                let grapheme_count = pre.graphemes(true).count();

                let mut g_idx = 0;
                for (i, _c) in password.real_text.grapheme_indices(true) {
                    if g_idx == grapheme_count {
                        index = i;
                    }
                    g_idx += 1;
                }

                // TODO: save/restore with selection bounds

                if cursor.index > 0 && index == 0 {
                    index = password.real_text.len();
                }

                buffer.set_text(
                    &mut font_system,
                    password.real_text.as_str(),
                    attrs.as_attrs(),
                    Shaping::Advanced,
                );
            });

            cursor.index = index;

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
