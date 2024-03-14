use crate::*;
use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowScaleFactorChanged},
};
use cosmic_text::Affinity;

#[derive(Component, Default)]
pub struct CosmicPadding(pub Vec2);

#[derive(Component, Default)]
pub struct CosmicWidgetSize(pub Vec2);

pub fn reshape(mut query: Query<&mut CosmicEditor>, mut font_system: ResMut<CosmicFontSystem>) {
    for mut cosmic_editor in query.iter_mut() {
        cosmic_editor.shape_as_needed(&mut font_system.0, false);
    }
}

pub fn set_padding(
    mut query: Query<(
        &mut CosmicPadding,
        &CosmicTextPosition,
        &CosmicBuffer,
        &CosmicWidgetSize,
        Option<&CosmicEditor>,
    )>,
) {
    for (mut padding, position, buffer, size, editor_opt) in query.iter_mut() {
        // TODO: At least one of these clones is uneccessary
        let mut buffer = buffer.0.clone();

        if let Some(editor) = editor_opt {
            buffer = editor.with_buffer(|b| b.clone());
        }

        if !buffer.redraw() {
            continue;
        }

        padding.0 = match position {
            CosmicTextPosition::Center => Vec2::new(
                get_x_offset_center(size.0.x, &buffer) as f32,
                get_y_offset_center(size.0.y, &buffer) as f32,
            ),
            CosmicTextPosition::TopLeft { padding } => Vec2::new(*padding as f32, *padding as f32),
            CosmicTextPosition::Left { padding } => Vec2::new(
                *padding as f32,
                get_y_offset_center(size.0.y, &buffer) as f32,
            ),
        }
    }
}

pub fn set_widget_size(
    mut query: Query<(&mut CosmicWidgetSize, &Sprite), Changed<Sprite>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if windows.iter().len() == 0 {
        return;
    }
    let scale = windows.single().scale_factor();
    for (mut size, sprite) in query.iter_mut() {
        size.0 = sprite.custom_size.unwrap().ceil() * scale;
    }
}

pub fn set_buffer_size(
    mut query: Query<(
        &mut CosmicBuffer,
        &CosmicMode,
        &CosmicWidgetSize,
        &CosmicTextPosition,
    )>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for (mut buffer, mode, size, position) in query.iter_mut() {
        let padding_x = match position {
            CosmicTextPosition::Center => 0.,
            CosmicTextPosition::TopLeft { padding } => *padding as f32,
            CosmicTextPosition::Left { padding } => *padding as f32,
        };

        let (buffer_width, buffer_height) = match mode {
            CosmicMode::InfiniteLine => (f32::MAX, size.0.y),
            CosmicMode::AutoHeight => (size.0.x - padding_x, (i32::MAX / 2) as f32),
            CosmicMode::Wrap => (size.0.x - padding_x, size.0.y),
        };

        buffer.set_size(&mut font_system.0, buffer_width, buffer_height);
    }
}

pub fn new_image_from_default(
    mut query: Query<&mut Handle<Image>, Added<CosmicBuffer>>,
    mut images: ResMut<Assets<Image>>,
) {
    for mut canvas in query.iter_mut() {
        *canvas = images.add(Image::default());
    }
}

pub fn set_cursor(
    mut query: Query<(
        &mut XOffset,
        &CosmicMode,
        &CosmicEditor,
        &CosmicBuffer,
        &CosmicWidgetSize,
        &CosmicPadding,
    )>,
) {
    for (mut x_offset, mode, editor, buffer, size, padding) in query.iter_mut() {
        let mut cursor_x = 0.;
        if mode == &CosmicMode::InfiniteLine {
            if let Some(line) = buffer.layout_runs().next() {
                for (idx, glyph) in line.glyphs.iter().enumerate() {
                    if editor.cursor().affinity == Affinity::Before {
                        if idx <= editor.cursor().index {
                            cursor_x += glyph.w;
                        }
                    } else if idx < editor.cursor().index {
                        cursor_x += glyph.w;
                    } else {
                        break;
                    }
                }
            }
        }

        if mode == &CosmicMode::InfiniteLine && x_offset.0.is_none() {
            *x_offset = XOffset(Some((0., size.0.x - 2. * padding.0.x)));
        }

        if let Some((x_min, x_max)) = x_offset.0 {
            if cursor_x > x_max {
                let diff = cursor_x - x_max;
                *x_offset = XOffset(Some((x_min + diff, cursor_x)));
            }
            if cursor_x < x_min {
                let diff = x_min - cursor_x;
                *x_offset = XOffset(Some((cursor_x, x_max - diff)));
            }
        }
    }
}

pub fn auto_height(
    mut query: Query<(
        Entity,
        &mut Sprite,
        &CosmicMode,
        &mut CosmicBuffer,
        &CosmicWidgetSize,
    )>,
    mut style_q: Query<(&mut Style, &CosmicSource)>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if windows.iter().len() == 0 {
        return;
    }

    let scale = windows.single().scale_factor();

    for (entity, mut sprite, mode, mut buffer, size) in query.iter_mut() {
        if mode == &CosmicMode::AutoHeight {
            let text_size = get_text_size(&buffer);
            let text_height = (text_size.1 + 30.) / scale;
            if text_height > size.0.y / scale {
                let mut new_size = sprite.custom_size.unwrap();
                new_size.y = text_height.ceil();
                // TODO this gets set automatically in UI cases but needs to be done for all other cases.
                // redundant work but easier to just set on all sprites
                sprite.custom_size = Some(new_size);

                buffer.set_redraw(true);

                // TODO: bad loop nesting
                for (mut style, source) in style_q.iter_mut() {
                    if source.0 != entity {
                        continue;
                    }
                    style.height = Val::Px(text_height.ceil());
                }
            }
        }
    }
}

pub fn set_sprite_size_from_ui(
    mut source_q: Query<&mut Sprite, With<CosmicBuffer>>,
    dest_q: Query<(&Node, &CosmicSource)>,
) {
    for (node, source) in dest_q.iter() {
        if let Ok(mut sprite) = source_q.get_mut(source.0) {
            sprite.custom_size = Some(node.size().ceil().max(Vec2::ONE));
        }
    }
}

pub fn on_scale_factor_change(
    mut scale_factor_changed: EventReader<WindowScaleFactorChanged>,
    mut cosmic_query: Query<(&mut CosmicBuffer, &mut XOffset)>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    if !scale_factor_changed.is_empty() {
        let new_scale_factor = scale_factor_changed.read().last().unwrap().scale_factor as f32;
        for (mut buffer, mut x_offset) in &mut cosmic_query.iter_mut() {
            let metrics = buffer.metrics();
            let font_system = &mut font_system.0;
            let metrics =
                Metrics::new(metrics.font_size, metrics.line_height).scale(new_scale_factor);

            buffer.set_metrics(font_system, metrics);
            buffer.set_redraw(true);

            *x_offset = XOffset(None);
        }
    }
}
