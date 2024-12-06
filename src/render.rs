use crate::widget::CosmicPadding;
use crate::{cosmic_edit::ReadOnly, prelude::*, widget::WidgetSet};
use crate::{cosmic_edit::*, CosmicWidgetSize};
use bevy::render::render_resource::Extent3d;
use cosmic_text::{Color, Edit};
use image::{imageops::FilterType, GenericImageView};

/// System set for cosmic text rendering systems. Runs in [`PostUpdate`]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct RenderSet;

pub(crate) struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<SwashCache>() {
            app.insert_resource(SwashCache::default());
        } else {
            debug!("Skipping inserting `SwashCache` resource");
        }
        app.add_systems(Update, blink_cursor).add_systems(
            PostUpdate,
            (render_texture,).in_set(RenderSet).after(WidgetSet),
        );
    }
}

pub(crate) fn blink_cursor(mut q: Query<&mut CosmicEditor, Without<ReadOnly>>, time: Res<Time>) {
    for mut e in q.iter_mut() {
        e.cursor_timer.tick(time.delta());
        if e.cursor_timer.just_finished() {
            e.cursor_visible = !e.cursor_visible;
            e.set_redraw(true);
        }
    }
}

fn draw_pixel(buffer: &mut [u8], width: i32, height: i32, x: i32, y: i32, color: Color) {
    let a_a = color.a() as u32;
    if a_a == 0 {
        // Do not draw if alpha is zero
        return;
    }

    if y < 0 || y >= height {
        // Skip if y out of bounds
        return;
    }

    if x < 0 || x >= width {
        // Skip if x out of bounds
        return;
    }

    let offset = (y as usize * width as usize + x as usize) * 4;

    let bg = bevy::prelude::Color::srgba_u8(
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    );

    // TODO: if alpha is 100% or bg is empty skip blending

    let fg = Srgba::rgba_u8(color.r(), color.g(), color.b(), color.a());

    let premul = (fg * fg.alpha).with_alpha(color.a() as f32 / 255.0);

    let out = premul + (bg.to_srgba() * (1.0 - fg.alpha));

    buffer[offset + 2] = (out.blue * 255.0) as u8;
    buffer[offset + 1] = (out.green * 255.0) as u8;
    buffer[offset] = (out.red * 255.0) as u8;
    buffer[offset + 3] = (out.alpha * 255.0) as u8;
}

/// Renders to the [CosmicRenderOutput]
#[allow(unused_mut)] // for .set_redraw(false) commented out
fn render_texture(
    mut query: Query<(
        Option<&mut CosmicEditor>,
        &mut CosmicEditBuffer,
        &DefaultAttrs,
        &CosmicBackgroundImage,
        &CosmicBackgroundColor,
        &CursorColor,
        &SelectionColor,
        Option<&SelectedTextColor>,
        &CosmicRenderOutput,
        CosmicWidgetSize,
        &CosmicPadding,
        &XOffset,
        Option<&ReadOnly>,
        &CosmicTextAlign,
    )>,
    mut font_system: ResMut<CosmicFontSystem>,
    mut images: ResMut<Assets<Image>>,
    mut swash_cache_state: ResMut<SwashCache>,
) {
    for (
        editor,
        mut buffer,
        attrs,
        background_image,
        fill_color,
        cursor_color,
        selection_color,
        selected_text_color_option,
        canvas,
        size,
        padding,
        x_offset,
        readonly_opt,
        position,
    ) in query.iter_mut()
    {
        let Ok(size) = size.logical_size() else {
            continue;
        };

        // avoids a panic
        if size.x == 0. || size.y == 0. {
            debug!(
                message = "Size of buffer is zero, skipping",
                // once = "This log only appears once"
            );
            continue;
        }

        // Draw background
        let mut pixels = vec![0; size.x as usize * size.y as usize * 4];
        if let Some(bg_image) = background_image.0.clone() {
            if let Some(image) = images.get(&bg_image) {
                let mut dynamic_image = image.clone().try_into_dynamic().unwrap();
                if image.size() != size.as_uvec2() {
                    dynamic_image = dynamic_image.resize_to_fill(
                        size.x as u32,
                        size.y as u32,
                        FilterType::Triangle,
                    );
                }
                for (i, (_, _, rgba)) in dynamic_image.pixels().enumerate() {
                    if let Some(p) = pixels.get_mut(i * 4..(i + 1) * 4) {
                        p[0] = rgba[0];
                        p[1] = rgba[1];
                        p[2] = rgba[2];
                        p[3] = rgba[3];
                    }
                }
            }
        } else {
            let bg = fill_color.0.to_cosmic();
            for pixel in pixels.chunks_exact_mut(4) {
                pixel[0] = bg.r(); // Red component
                pixel[1] = bg.g(); // Green component
                pixel[2] = bg.b(); // Blue component
                pixel[3] = bg.a(); // Alpha component
            }
        }

        let font_color = attrs
            .0
            .color_opt
            .unwrap_or(cosmic_text::Color::rgb(0, 0, 0));

        let min_pad = match position {
            CosmicTextAlign::Center { padding } => *padding as f32,
            CosmicTextAlign::TopLeft { padding } => *padding as f32,
            CosmicTextAlign::Left { padding } => *padding as f32,
        };

        let draw_closure = |x, y, w, h, color| {
            for row in 0..h as i32 {
                for col in 0..w as i32 {
                    draw_pixel(
                        &mut pixels,
                        size.x as i32,
                        size.y as i32,
                        x + col + padding.x.max(min_pad) as i32 - x_offset.left as i32,
                        y + row + padding.y as i32,
                        color,
                    );
                }
            }
        };

        // Draw glyphs
        if let Some(mut editor) = editor {
            if !editor.redraw() {
                continue;
            }

            let cursor_color = cursor_color.0;
            let cursor_opacity = if editor.cursor_visible && readonly_opt.is_none() {
                cursor_color.alpha()
            } else {
                0.
            };

            let cursor_color = cursor_color.with_alpha(cursor_opacity).to_cosmic();

            let selection_color = selection_color.0.to_cosmic();

            let selected_text_color = selected_text_color_option
                .map(|selected_text_color| selected_text_color.0.to_cosmic())
                .unwrap_or(font_color);

            editor.draw(
                &mut font_system.0,
                &mut swash_cache_state.0,
                font_color,
                cursor_color,
                selection_color,
                selected_text_color,
                draw_closure,
            );
            // TODO: Performance optimization, read all possible render-input
            // changes and only redraw if necessary
            // editor.set_redraw(false);
        } else {
            if !buffer.redraw() {
                continue;
            }
            buffer.draw(
                &mut font_system.0,
                &mut swash_cache_state.0,
                font_color,
                draw_closure,
            );
            // TODO: Performance optimization, read all possible render-input
            // changes and only redraw if necessary
            // buffer.set_redraw(false);
        }

        if let Some(prev_image) = images.get_mut(&canvas.0) {
            prev_image.data.clear();
            // Updates the stored asset image with the computed pixels
            prev_image.data.extend_from_slice(pixels.as_slice());
            prev_image.resize(Extent3d {
                width: size.x as u32,
                height: size.y as u32,
                depth_or_array_layers: 1,
            });
        }
    }
}
