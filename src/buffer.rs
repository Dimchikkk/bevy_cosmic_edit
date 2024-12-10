use crate::{
    prelude::*, render_implementations::OutputToEntity, CosmicBackgroundColor,
    CosmicBackgroundImage, CosmicTextAlign, CosmicWrap, CursorColor, MaxChars, MaxLines,
    SelectionColor,
};
use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    window::PrimaryWindow,
};
use cosmic_text::{
    Attrs, AttrsOwned, BorrowedWithFontSystem, Buffer, Edit, FontSystem, Metrics, Shaping,
};

pub(crate) struct BufferPlugin;

impl Plugin for BufferPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            First,
            (
                add_font_system,
                // set_initial_scale,
                set_redraw,
                set_editor_redraw,
                update_internal_target_handles.pipe(render_implementations::debug_error),
            )
                .chain(),
        );
    }
}






/// Adds a [`FontSystem`] to a newly created [`CosmicEditBuffer`] if one was not provided
pub(crate) fn add_font_system(
    mut font_system: ResMut<CosmicFontSystem>,
    mut q: Query<&mut CosmicEditBuffer, Added<CosmicEditBuffer>>,
) {
    for mut b in q.iter_mut() {
        if !b.lines.is_empty() {
            continue;
        }
        b.0.set_text(&mut font_system, "", Attrs::new(), Shaping::Advanced);
        b.set_redraw(true);
    }
}

/// Initialises [`CosmicEditBuffer`] scale factor
pub(crate) fn set_initial_scale(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut cosmic_query: Query<&mut CosmicEditBuffer, Added<CosmicEditBuffer>>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    if let Ok(window) = window_q.get_single() {
        let w_scale = window.scale_factor();

        for mut b in &mut cosmic_query.iter_mut() {
            let m = b.metrics().scale(w_scale);
            b.set_metrics(&mut font_system, m);
        }
    }
}

/// Initialises new [`CosmicEditBuffer`] redraw flag to true
pub(crate) fn set_redraw(mut q: Query<&mut CosmicEditBuffer, Added<CosmicEditBuffer>>) {
    for mut b in q.iter_mut() {
        b.set_redraw(true);
    }
}

/// Initialises new [`CosmicEditor`] redraw flag to true
pub(crate) fn set_editor_redraw(mut q: Query<&mut CosmicEditor, Added<CosmicEditor>>) {
    for mut ed in q.iter_mut() {
        ed.set_redraw(true);
    }
}

/// Every frame updates the output (in [`CosmicRenderOutput`]) to its receiver
/// on the same entity, e.g. [`Sprite`]
pub(crate) fn update_internal_target_handles(
    mut buffers_q: Query<(&CosmicRenderOutput, OutputToEntity), With<CosmicEditBuffer>>,
) -> render_implementations::Result<()> {
    for (CosmicRenderOutput(output_data), mut output_components) in buffers_q.iter_mut() {
        output_components.write_image_data(output_data)?;
    }

    Ok(())
}
