use std::time::Duration;

use bevy::{
    asset::HandleId,
    prelude::*,
    render::render_resource::Extent3d,
    utils::HashMap,
    window::{PrimaryWindow, WindowScaleFactorChanged},
};
use cosmic_text::{Affinity, Edit, Metrics, SwashCache};
use image::{imageops::FilterType, GenericImageView};

use crate::{
    get_text_size, get_x_offset_center, get_y_offset_center, CosmicAttrs, CosmicBackground,
    CosmicCanvas, CosmicEditor, CosmicFontSystem, CosmicMetrics, CosmicMode, CosmicText,
    CosmicTextPosition, FillColor, Focus, PasswordInput, Placeholder, ReadOnly, XOffset,
    DEFAULT_SCALE_PLACEHOLDER,
};

#[derive(Resource)]
pub(crate) struct SwashCacheState {
    pub swash_cache: SwashCache,
}

#[derive(Resource)]
pub(crate) struct CursorBlinkTimer(pub Timer);

#[derive(Resource)]
pub(crate) struct CursorVisibility(pub bool);

pub(crate) fn cosmic_edit_redraw_buffer(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut swash_cache_state: ResMut<SwashCacheState>,
    mut cosmic_edit_query: Query<(
        &mut CosmicEditor,
        &CosmicAttrs,
        &CosmicBackground,
        &FillColor,
        &mut CosmicCanvas,
        &CosmicTextPosition,
        Option<&Node>,
        Option<&mut Style>,
        Option<&mut Sprite>,
        &mut XOffset,
        &CosmicMode,
        Option<&mut Placeholder>,
    )>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    let primary_window = windows.single();
    let scale = primary_window.scale_factor() as f32;

    for (
        mut cosmic_editor,
        attrs,
        background_image,
        fill_color,
        mut canvas,
        text_position,
        node_opt,
        style_opt,
        sprite_opt,
        mut x_offset,
        mode,
        mut placeholder_opt,
    ) in &mut cosmic_edit_query.iter_mut()
    {
        if !cosmic_editor.0.buffer().redraw() {
            continue;
        }

        let current_text = cosmic_editor.get_text();

        // Check for placeholder, replace editor if found and buffer is empty
        let editor = if current_text.is_empty() && placeholder_opt.is_some() {
            let placeholder = &mut placeholder_opt.as_mut().unwrap().0 .0;
            placeholder.buffer_mut().set_redraw(true);

            cosmic_editor.0.buffer_mut().set_redraw(true);

            let mut cursor = placeholder.cursor();
            cursor.index = 0;
            placeholder.set_cursor(cursor);
            *x_offset = XOffset(None);
            placeholder
        } else {
            &mut cosmic_editor.0
        };

        editor.shape_as_needed(&mut font_system.0);

        // Get numbers, do maths to find and set cursor
        //
        let (base_width, mut base_height) = match node_opt {
            Some(node) => (node.size().x.ceil(), node.size().y.ceil()),
            None => (
                sprite_opt.as_ref().unwrap().custom_size.unwrap().x.ceil(),
                sprite_opt.as_ref().unwrap().custom_size.unwrap().y.ceil(),
            ),
        };

        let widget_width = base_width * scale;
        let widget_height = base_height * scale;

        let padding_x = match text_position {
            CosmicTextPosition::Center => 0.,
            CosmicTextPosition::TopLeft { padding } => *padding as f32,
            CosmicTextPosition::Left { padding } => *padding as f32,
        };

        let (buffer_width, buffer_height) = match mode {
            CosmicMode::InfiniteLine => (f32::MAX, widget_height),
            CosmicMode::AutoHeight => (widget_width - padding_x, (i32::MAX / 2) as f32),
            CosmicMode::Wrap => (widget_width - padding_x, widget_height),
        };

        editor
            .buffer_mut()
            .set_size(&mut font_system.0, buffer_width, buffer_height);

        if mode == &CosmicMode::AutoHeight {
            let text_size = get_text_size(editor.buffer());
            let text_height = (text_size.1 + 30.) / primary_window.scale_factor() as f32;
            if text_height > base_height {
                base_height = text_height.ceil();
                match style_opt {
                    Some(mut style) => style.height = Val::Px(base_height),
                    None => sprite_opt.unwrap().custom_size.unwrap().y = base_height,
                }
            }
        }

        let mut cursor_x = 0.;
        if mode == &CosmicMode::InfiniteLine {
            if let Some(line) = editor.buffer().layout_runs().next() {
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
            let padding_x = match text_position {
                CosmicTextPosition::Center => get_x_offset_center(widget_width, editor.buffer()),
                CosmicTextPosition::TopLeft { padding } => *padding,
                CosmicTextPosition::Left { padding } => *padding,
            };
            *x_offset = XOffset(Some((0., widget_width - 2. * padding_x as f32)));
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

        // Draw background
        let mut pixels = vec![0; widget_width as usize * widget_height as usize * 4];
        if let Some(bg_image) = background_image.0.clone() {
            if let Some(image) = images.get(&bg_image) {
                let mut dynamic_image = image.clone().try_into_dynamic().unwrap();
                if image.size().x != widget_width || image.size().y != widget_height {
                    dynamic_image = dynamic_image.resize_to_fill(
                        widget_width as u32,
                        widget_height as u32,
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
            let bg = fill_color.0;
            for pixel in pixels.chunks_exact_mut(4) {
                pixel[0] = (bg.r() * 255.) as u8; // Red component
                pixel[1] = (bg.g() * 255.) as u8; // Green component
                pixel[2] = (bg.b() * 255.) as u8; // Blue component
                pixel[3] = (bg.a() * 255.) as u8; // Alpha component
            }
        }

        // Get values for glyph draw step
        let (padding_x, padding_y) = match text_position {
            CosmicTextPosition::Center => (
                get_x_offset_center(widget_width, editor.buffer()),
                get_y_offset_center(widget_height, editor.buffer()),
            ),
            CosmicTextPosition::TopLeft { padding } => (*padding, *padding),
            CosmicTextPosition::Left { padding } => (
                *padding,
                get_y_offset_center(widget_height, editor.buffer()),
            ),
        };

        let font_color = attrs
            .0
            .color_opt
            .unwrap_or(cosmic_text::Color::rgb(0, 0, 0));

        // Draw glyphs
        editor.draw(
            &mut font_system.0,
            &mut swash_cache_state.swash_cache,
            font_color,
            |x, y, w, h, color| {
                for row in 0..h as i32 {
                    for col in 0..w as i32 {
                        draw_pixel(
                            &mut pixels,
                            widget_width as i32,
                            widget_height as i32,
                            x + col + padding_x - x_offset.0.unwrap_or((0., 0.)).0 as i32,
                            y + row + padding_y,
                            color,
                        );
                    }
                }
            },
        );

        let canvas = &mut canvas.0;

        if let Some(prev_image) = images.get_mut(canvas) {
            if *canvas == bevy::render::texture::DEFAULT_IMAGE_HANDLE.typed() {
                let mut prev_image = prev_image.clone();
                prev_image.data.clear();
                prev_image.data.extend_from_slice(pixels.as_slice());
                prev_image.resize(Extent3d {
                    width: widget_width as u32,
                    height: widget_height as u32,
                    depth_or_array_layers: 1,
                });
                let handle_id: HandleId = HandleId::random::<Image>();
                let new_handle: Handle<Image> = Handle::weak(handle_id);
                let new_handle = images.set(new_handle, prev_image);
                *canvas = new_handle;
            } else {
                prev_image.data.clear();
                prev_image.data.extend_from_slice(pixels.as_slice());
                prev_image.resize(Extent3d {
                    width: widget_width as u32,
                    height: widget_height as u32,
                    depth_or_array_layers: 1,
                });
            }
        }

        editor.buffer_mut().set_redraw(false);
    }
}

fn draw_pixel(
    buffer: &mut [u8],
    width: i32,
    height: i32,
    x: i32,
    y: i32,
    color: cosmic_text::Color,
) {
    // TODO: perftest this fn against previous iteration
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

    let bg = Color::rgba_u8(
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    );

    // TODO: if alpha is 100% or bg is empty skip blending

    let fg = Color::rgba_u8(color.r(), color.g(), color.b(), color.a());

    let premul = fg * Vec3::splat(color.a() as f32 / 255.0);

    let out = premul + bg * (1.0 - fg.a());

    buffer[offset + 2] = (out.b() * 255.0) as u8;
    buffer[offset + 1] = (out.g() * 255.0) as u8;
    buffer[offset] = (out.r() * 255.0) as u8;
    buffer[offset + 3] = (out.a() * 255.0) as u8;
}

pub(crate) fn blink_cursor(
    mut visibility: ResMut<CursorVisibility>,
    mut timer: ResMut<CursorBlinkTimer>,
    time: Res<Time>,
    active_editor: ResMut<Focus>,
    mut cosmic_editor_q: Query<&mut CosmicEditor, Without<ReadOnly>>,
    mut placeholder_editor_q: Query<&mut Placeholder, Without<ReadOnly>>,
) {
    if let Some(e) = active_editor.0 {
        timer.0.tick(time.delta());
        if !timer.0.just_finished() && !active_editor.is_changed() {
            return;
        }
        visibility.0 = !visibility.0;

        // always start cursor visible on focus
        if active_editor.is_changed() {
            visibility.0 = true;
            timer.0.set_elapsed(Duration::ZERO);
        }

        let new_color = if visibility.0 {
            None
        } else {
            Some(cosmic_text::Color::rgba(0, 0, 0, 0))
        };

        if let Ok(mut editor) = cosmic_editor_q.get_mut(e) {
            let editor = &mut editor.0;
            let mut cursor = editor.cursor();
            cursor.color = new_color;
            editor.set_cursor(cursor);
            editor.buffer_mut().set_redraw(true);
        }

        if let Ok(mut placeholder) = placeholder_editor_q.get_mut(e) {
            let placeholder = &mut placeholder.0 .0;
            let mut cursor_p = placeholder.cursor();
            cursor_p.color = new_color;
            placeholder.set_cursor(cursor_p);
            placeholder.buffer_mut().set_redraw(true);
        }
    }
}

pub(crate) fn freeze_cursor_blink(
    mut visibility: ResMut<CursorVisibility>,
    mut timer: ResMut<CursorBlinkTimer>,
    active_editor: Res<Focus>,
    keys: Res<Input<KeyCode>>,
    char_evr: EventReader<ReceivedCharacter>,
    mut editor_q: Query<&mut CosmicEditor, Without<ReadOnly>>,
) {
    let inputs = [
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Back,
        KeyCode::Return,
    ];
    if !keys.any_pressed(inputs) && char_evr.is_empty() {
        return;
    }

    if let Some(e) = active_editor.0 {
        if let Ok(mut editor) = editor_q.get_mut(e) {
            timer.0.set_elapsed(Duration::ZERO);
            visibility.0 = true;
            let mut cursor = editor.0.cursor();
            cursor.color = None;
            editor.0.set_cursor(cursor);
            editor.0.buffer_mut().set_redraw(true);
        }
    }
}

pub(crate) fn hide_inactive_or_readonly_cursor(
    mut cosmic_editor_q_readonly: Query<&mut CosmicEditor, With<ReadOnly>>,
    mut cosmic_editor_q_placeholder: Query<(Entity, &mut Placeholder, Option<&ReadOnly>)>,
    mut cosmic_editor_q_editable: Query<(Entity, &mut CosmicEditor), Without<ReadOnly>>,
    active_editor: Res<Focus>,
) {
    for mut editor in &mut cosmic_editor_q_readonly.iter_mut() {
        let mut cursor = editor.0.cursor();
        cursor.color = Some(cosmic_text::Color::rgba(0, 0, 0, 0));
        editor.0.set_cursor(cursor);
        editor.0.buffer_mut().set_redraw(true);
    }

    for (e, mut editor, readonly_opt) in &mut cosmic_editor_q_placeholder.iter_mut() {
        // filthy short circuiting instead of correct unwrapping
        if active_editor.is_none() || e != active_editor.0.unwrap() || readonly_opt.is_some() {
            let editor = &mut editor.0;
            let mut cursor = editor.0.cursor();
            if cursor.color == Some(cosmic_text::Color::rgba(0, 0, 0, 0)) {
                return;
            }
            cursor.color = Some(cosmic_text::Color::rgba(0, 0, 0, 0));
            editor.0.set_cursor(cursor);
            editor.0.buffer_mut().set_redraw(true);
        }
    }

    for (e, mut editor) in &mut cosmic_editor_q_editable.iter_mut() {
        if active_editor.is_none() || e != active_editor.0.unwrap() {
            let mut cursor = editor.0.cursor();
            if cursor.color == Some(cosmic_text::Color::rgba(0, 0, 0, 0)) {
                return;
            }
            cursor.color = Some(cosmic_text::Color::rgba(0, 0, 0, 0));
            editor.0.set_cursor(cursor);
            editor.0.buffer_mut().set_redraw(true);
        }
    }
}

pub(crate) fn set_initial_scale(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut metrics_q: Query<&mut CosmicMetrics, Added<CosmicMetrics>>,
) {
    let scale = window_q.single().scale_factor() as f32;

    for mut metrics in metrics_q.iter_mut() {
        if metrics.scale_factor == DEFAULT_SCALE_PLACEHOLDER {
            metrics.scale_factor = scale;
        }
    }
}

pub(crate) fn on_scale_factor_change(
    mut scale_factor_changed: EventReader<WindowScaleFactorChanged>,
    mut cosmic_query: Query<(&mut CosmicEditor, &CosmicMetrics, &mut XOffset)>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    if !scale_factor_changed.is_empty() {
        let new_scale_factor = scale_factor_changed.iter().last().unwrap().scale_factor as f32;
        for (mut editor, metrics, mut x_offset) in &mut cosmic_query.iter_mut() {
            let font_system = &mut font_system.0;
            let metrics =
                Metrics::new(metrics.font_size, metrics.line_height).scale(new_scale_factor);

            editor.0.buffer_mut().set_metrics(font_system, metrics);
            editor.0.buffer_mut().set_redraw(true);

            *x_offset = XOffset(None);
        }
    }
}

pub(crate) fn cosmic_ui_to_canvas(
    mut added_ui_images: Query<(&mut UiImage, &CosmicCanvas), Added<UiImage>>,
) {
    for (mut ui_image, canvas) in added_ui_images.iter_mut() {
        ui_image.texture = canvas.0.clone_weak();
    }
}

pub(crate) fn update_handle_ui(
    mut changed_handles: Query<(&mut UiImage, &CosmicCanvas), Changed<CosmicCanvas>>,
) {
    for (mut ui_image, canvas) in changed_handles.iter_mut() {
        ui_image.texture = canvas.0.clone_weak();
    }
}

pub(crate) fn cosmic_sprite_to_canvas(
    mut added_sprite_textures: Query<(&mut Handle<Image>, &CosmicCanvas), Added<Handle<Image>>>,
) {
    for (mut handle, canvas) in added_sprite_textures.iter_mut() {
        *handle = canvas.0.clone_weak();
    }
}

pub(crate) fn update_handle_sprite(
    mut changed_handles: Query<(&mut Handle<Image>, &CosmicCanvas), Changed<CosmicCanvas>>,
) {
    for (mut handle, canvas) in changed_handles.iter_mut() {
        *handle = canvas.0.clone_weak();
    }
}

#[derive(Resource, Default)]
pub(crate) struct PasswordStates(pub HashMap<Entity, (String, usize)>);

pub(crate) fn hide_password_text(
    mut editor_q: Query<(Entity, &mut CosmicEditor, &CosmicAttrs, &PasswordInput)>,
    mut font_system: ResMut<CosmicFontSystem>,
    mut password_input_states: ResMut<PasswordStates>,
    active_editor: Res<Focus>,
) {
    for (entity, mut cosmic_editor, attrs, password) in editor_q.iter_mut() {
        let text = cosmic_editor.get_text();
        let select_opt = cosmic_editor.0.select_opt();
        let mut cursor = cosmic_editor.0.cursor();

        if !text.is_empty() {
            cosmic_editor.set_text(
                CosmicText::OneStyle(format!("{}", password.0).repeat(text.chars().count())),
                attrs.0.clone(),
                &mut font_system.0,
            );

            // multiply cursor idx and select_opt end point by password char length
            // the actual char length cos '●' is 3x as long as 'a'
            // This operation will need to be undone when resetting.
            //
            // Currently breaks entering multi-byte chars

            let char_len = password.0.len_utf8();

            let select_opt = match select_opt {
                Some(mut select) => {
                    select.index *= char_len;
                    Some(select)
                }
                None => None,
            };

            cursor.index *= char_len;

            cosmic_editor.0.set_select_opt(select_opt);

            // Fixes stuck cursor on password inputs
            if let Some(active) = active_editor.0 {
                if entity != active {
                    cursor.color = Some(cosmic_text::Color::rgba(0, 0, 0, 0));
                }
            }

            cosmic_editor.0.set_cursor(cursor);
        }

        let glyph_idx = match cosmic_editor.0.buffer().lines[0].layout_opt() {
            Some(_) => cosmic_editor.0.buffer().layout_cursor(&cursor).glyph,
            None => 0,
        };

        password_input_states.0.insert(entity, (text, glyph_idx));
    }
}

pub(crate) fn restore_password_text(
    mut editor_q: Query<(Entity, &mut CosmicEditor, &CosmicAttrs, &PasswordInput)>,
    mut font_system: ResMut<CosmicFontSystem>,
    password_input_states: Res<PasswordStates>,
) {
    for (entity, mut cosmic_editor, attrs, password) in editor_q.iter_mut() {
        if let Some((text, _glyph_idx)) = password_input_states.0.get(&entity) {
            // reset intercepted text
            if !text.is_empty() {
                let char_len = password.0.len_utf8();

                let mut cursor = cosmic_editor.0.cursor();
                let select_opt = match cosmic_editor.0.select_opt() {
                    Some(mut select) => {
                        select.index /= char_len;
                        Some(select)
                    }
                    None => None,
                };

                cursor.index /= char_len;

                cosmic_editor.set_text(
                    crate::CosmicText::OneStyle(text.clone()),
                    attrs.0.clone(),
                    &mut font_system.0,
                );

                cosmic_editor.0.set_select_opt(select_opt);
                cosmic_editor.0.set_cursor(cursor);
            }
        }
    }
}
