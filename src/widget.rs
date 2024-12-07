use crate::buffer::get_x_offset_center;
use crate::buffer::get_y_offset_center;
use crate::cosmic_edit::CosmicTextAlign;
use crate::cosmic_edit::CosmicWrap;
use crate::cosmic_edit::XOffset;
use crate::input::InputSet;
use crate::prelude::*;
use crate::ChangedCosmicWidgetSize;
use crate::CosmicWidgetSize;
use cosmic_text::Affinity;
use cosmic_text::Edit;

/// System set for cosmic text layout systems. Runs in [`PostUpdate`]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct WidgetSet;

pub(crate) struct WidgetPlugin;

impl Plugin for WidgetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, reshape.in_set(WidgetSet).after(InputSet::Update))
            .add_systems(
                PostUpdate,
                (
                    new_image_from_default,
                    set_buffer_size,
                    set_padding,
                    set_x_offset,
                )
                    .chain()
                    .in_set(WidgetSet)
                    .after(TransformSystem::TransformPropagate),
            )
            .register_type::<CosmicPadding>();
    }
}

/// Wrapper for a [`Vec2`] describing the horizontal and vertical padding of a widget.
/// This is set programatically, not for user modification.
/// To set a widget's padding, use [`CosmicTextAlign`]
#[derive(Component, Reflect, Default, Deref, DerefMut, Debug)]
pub(crate) struct CosmicPadding(pub(crate) Vec2);

// /// Wrapper for a [`Vec2`] describing the horizontal and vertical size of a widget.
// /// This is set programatically, not for user modification.
// /// To set a widget's size, use either it's [`Sprite`] dimensions or modify the target UI element's
// /// size.
// #[derive(Component, Reflect, Default, Deref, DerefMut)]
// pub(crate) struct CosmicWidgetSize(pub(crate) Vec2);

/// Reshapes text in a [`CosmicEditor`]
fn reshape(mut query: Query<&mut CosmicEditor>, mut font_system: ResMut<CosmicFontSystem>) {
    for mut cosmic_editor in query.iter_mut() {
        cosmic_editor.shape_as_needed(&mut font_system.0, false);
    }
}

/// Programatically sets the [`CosmicPadding`] of a widget based on it's [`CosmicTextAlign`]
fn set_padding(
    mut query: Query<
        (
            &mut CosmicPadding,
            &CosmicTextAlign,
            &CosmicEditBuffer,
            CosmicWidgetSize,
            Option<&CosmicEditor>,
        ),
        Or<(
            With<CosmicEditor>,
            Changed<CosmicTextAlign>,
            Changed<CosmicEditBuffer>,
            ChangedCosmicWidgetSize,
        )>,
    >,
) {
    for (mut padding, position, buffer, size, editor_opt) in query.iter_mut() {
        let Ok(size) = size.logical_size() else {
            continue;
        };

        // TODO: At least one of these clones is uneccessary
        let mut buffer = buffer.0.clone();

        if let Some(editor) = editor_opt {
            buffer = editor.with_buffer(|b| b.clone());
        }

        if !buffer.redraw() {
            continue;
        }

        padding.0 = match position {
            CosmicTextAlign::Center { padding: _ } => Vec2::new(
                get_x_offset_center(size.x, &buffer) as f32,
                get_y_offset_center(size.y, &buffer) as f32,
            ),
            CosmicTextAlign::TopLeft { padding } => Vec2::new(*padding as f32, *padding as f32),
            CosmicTextAlign::Left { padding } => {
                Vec2::new(*padding as f32, get_y_offset_center(size.y, &buffer) as f32)
            }
        }
    }
}

/// Sets the internal [`Buffer`]'s size according to the [`CosmicWidgetSize`] and [`CosmicTextAlign`]
fn set_buffer_size(
    mut query: Query<
        (
            &mut CosmicEditBuffer,
            &CosmicWrap,
            CosmicWidgetSize,
            &CosmicTextAlign,
        ),
        Or<(
            Changed<CosmicWrap>,
            ChangedCosmicWidgetSize,
            Changed<CosmicTextAlign>,
        )>,
    >,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for (mut buffer, mode, size, position) in query.iter_mut() {
        let Ok(size) = size.logical_size() else {
            continue;
        };
        let padding_x = match position {
            CosmicTextAlign::Center { padding: _ } => 0.,
            CosmicTextAlign::TopLeft { padding } => *padding as f32,
            CosmicTextAlign::Left { padding } => *padding as f32,
        };

        let (buffer_width, buffer_height) = match mode {
            CosmicWrap::InfiniteLine => (f32::MAX, size.y),
            CosmicWrap::Wrap => (size.x - padding_x, size.y),
        };

        buffer.set_size(&mut font_system.0, Some(buffer_width), Some(buffer_height));
    }
}

/// Instantiates a new image for a [`CosmicEditBuffer`]
fn new_image_from_default(
    mut query: Query<&mut CosmicRenderOutput, Added<CosmicEditBuffer>>,
    mut images: ResMut<Assets<Image>>,
) {
    for mut canvas in query.iter_mut() {
        debug!(message = "Initializing a new canvas");
        *canvas = CosmicRenderOutput(images.add(Image::default()));
    }
}

fn set_x_offset(
    mut query: Query<(
        &mut XOffset,
        &CosmicWrap,
        &CosmicEditor,
        CosmicWidgetSize,
        &CosmicTextAlign,
    )>,
) {
    for (mut x_offset, mode, editor, size, position) in query.iter_mut() {
        let Ok(size) = size.logical_size() else {
            continue;
        };
        if mode != &CosmicWrap::InfiniteLine {
            continue; // this used to be `return` though I feel like it should be continue
        }

        let mut cursor_x = 0.;
        let cursor = editor.cursor();

        // counts the width of the glyphs up to the cursor in `cursor_x`
        if let Some(line) = editor.with_buffer(|b| b.clone()).layout_runs().next() {
            for (idx, glyph) in line.glyphs.iter().enumerate() {
                match cursor.affinity {
                    Affinity::Before => {
                        if idx <= cursor.index {
                            cursor_x += glyph.w;
                        } else {
                            break;
                        }
                    }
                    Affinity::After => {
                        if idx < cursor.index {
                            cursor_x += glyph.w;
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        let padding_x = match position {
            CosmicTextAlign::Center { padding } => *padding as f32,
            CosmicTextAlign::TopLeft { padding } => *padding as f32,
            CosmicTextAlign::Left { padding } => *padding as f32,
        };

        if x_offset.width.is_none() {
            x_offset.width = Some(size.x - padding_x * 2.);
        }

        let right = x_offset.width.unwrap() + x_offset.left;

        if cursor_x > right {
            let diff = cursor_x - right;
            x_offset.left += diff;
        }
        if cursor_x < x_offset.left {
            let diff = x_offset.left - cursor_x;
            x_offset.left -= diff;
        }
    }
}
