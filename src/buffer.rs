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







