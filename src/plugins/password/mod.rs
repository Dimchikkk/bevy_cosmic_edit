use bevy::prelude::*;
use cosmic_text::{Buffer, Edit, Shaping};

use crate::{
    input::input_mouse, CosmicBuffer, CosmicEditor, CosmicFontSystem, DefaultAttrs, Render,
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
            glyph: '*',
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
                password.real_text = text;
            });

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
            continue;
        }

        buffer.set_text(
            &mut font_system,
            password.real_text.as_str(),
            attrs.as_attrs(),
        );
    }
}
