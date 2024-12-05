use crate::{
    cosmic_edit::DefaultAttrs, focus::FocusSet, placeholder::Placeholder, prelude::*,
    render::RenderSet,
};
use cosmic_text::{Cursor, Edit, Selection, Shaping};
use unicode_segmentation::UnicodeSegmentation;

pub(crate) struct PasswordPlugin;

/// System set for password blocking systems. Runs in [`PostUpdate`]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PasswordSet;

impl Plugin for PasswordPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (hide_password_text.before(crate::input::input_mouse),),
        )
        .add_systems(
            Update,
            (restore_password_text
                .before(crate::input::kb_input_text)
                .after(crate::input::kb_move_cursor),),
        )
        .add_systems(
            PostUpdate,
            (
                hide_password_text.before(RenderSet).in_set(PasswordSet),
                restore_password_text.before(FocusSet).after(RenderSet),
            ),
        );
    }
}

/// Component to be added to an entity with a [`CosmicEditBuffer`] to block contents with a
/// password blocker glyph
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_cosmic_edit::*;
/// #
/// # fn setup(mut commands: Commands) {
/// // Create a new cosmic bundle
/// commands.spawn((
///     CosmicEditBuffer::default(),
///     Sprite {
///         custom_size: Some(Vec2::new(300.0, 40.0)),
///         ..default()
///     },
///     Password::default()
/// ));
/// # }
/// #
/// # fn main() {
/// #     App::new()
/// #         .add_plugins(MinimalPlugins)
/// #         .add_plugins(CosmicEditPlugin::default())
/// #         .add_systems(Startup, setup);
/// # }
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

impl Password {
    /// New password component with custom blocker glyph
    pub fn new(glyph: char) -> Self {
        Self { glyph, ..default() }
    }
}

fn hide_password_text(
    mut q: Query<(
        &mut Password,
        &mut CosmicEditBuffer,
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
        &mut CosmicEditBuffer,
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
