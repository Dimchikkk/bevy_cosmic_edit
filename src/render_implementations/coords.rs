use bevy::ecs::query::QueryData;
use bevy::picking::backend::HitData;
use bevy::ui::RelativeCursorPosition;
use render_implementations::{RenderTargetError, SourceType};

use crate::render::WidgetBufferCoordTransformation;
use crate::render_implementations::CosmicWidgetSize;
use crate::{prelude::*, CosmicTextAlign};

/// Responsible for translating a world coordinate to a buffer coordinate
#[derive(QueryData)]
pub(crate) struct RelativeQuery {
    /// Widget size
    size: CosmicWidgetSize,
    text_align: &'static CosmicTextAlign,
    sprite_global_transform: &'static GlobalTransform,
    ui_cursor_position: Option<&'static RelativeCursorPosition>,
}

impl<'s> std::ops::Deref for RelativeQueryItem<'s> {
    type Target = render_implementations::RenderTypeScanItem<'s>;

    fn deref(&self) -> &Self::Target {
        self.size.deref()
    }
}

impl RelativeQueryItem<'_> {
    pub fn compute_buffer_coord(
        &self,
        hit_data: &HitData,
        buffer_size: Vec2,
    ) -> Result<Vec2, render_implementations::RenderTargetError> {
        match self.scan()? {
            SourceType::Sprite => {
                if hit_data.normal != Some(Vec3::Z) {
                    warn!(?hit_data, "Normal is not out of screen, skipping");
                    return Err(RenderTargetError::SpriteUnexpectedNormal);
                }

                let world_position = hit_data
                    .position
                    .ok_or(RenderTargetError::SpriteExpectedHitdataPosition)?;
                let RelativeQueryItem {
                    sprite_global_transform,
                    text_align,
                    size,
                    ..
                } = self;

                let position_transform =
                    GlobalTransform::from(Transform::from_translation(world_position));
                let relative_transform = position_transform.reparented_to(sprite_global_transform);
                let relative_position = relative_transform.translation.xy();

                let render_target_size = size.logical_size()?;
                let transformation = WidgetBufferCoordTransformation::new(
                    text_align.vertical,
                    render_target_size,
                    buffer_size,
                );
                // .xy swizzle depends on normal vector being perfectly out of screen
                let buffer_coord =
                    transformation.widget_origined_to_buffer_topleft(relative_position);

                Ok(buffer_coord)
            }
            SourceType::Ui => {
                let RelativeQueryItem {
                    size,
                    text_align,
                    ui_cursor_position,
                    ..
                } = self;
                let cursor_position_normalized = ui_cursor_position
                    .ok_or(RenderTargetError::RequiredComponentNotAvailable)?
                    .normalized
                    .ok_or(RenderTargetError::UiExpectedCursorPosition)?;

                let widget_size = size.logical_size()?;
                let relative_position = cursor_position_normalized * widget_size;

                let transformation = WidgetBufferCoordTransformation::new(
                    text_align.vertical,
                    widget_size,
                    buffer_size,
                );

                let buffer_coord =
                    transformation.widget_topleft_to_buffer_topleft(relative_position);

                Ok(buffer_coord)
            }
        }
    }
}
