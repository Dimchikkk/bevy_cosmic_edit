//! Generalizes over render target implementations.
//!
//! All implementations should use [`bevy::picking`] for interactions,
//! even [`SourceType::Ui`], for consistency.
//!
//! ## Sprite: [`TextEdit2d`]
//! Requires [`Sprite`] component and requires [`Sprite.custom_size`] to be Some( non-zero )
//!
//! ## UI: [`TextEdit`]
//! Requires [`ImageNode`] for rendering
// TODO: Remove `CosmicWidgetSize`?

mod prelude {
    pub(super) use super::error::Result;
    pub(super) use super::{RenderTargetError, SourceType};
    pub(super) use super::{RenderTypeScan, RenderTypeScanItem};
}

pub use error::*;
mod error {
    pub type Error = crate::render_implementations::RenderTargetError;
    pub type Result<T> = core::result::Result<T, RenderTargetError>;

    #[derive(Debug)]
    pub enum RenderTargetError {
        /// When no recognized [`SourceType`] could be found
        NoTargetsAvailable,

        /// When more than one [`SourceType`] was detected.
        ///
        /// This will always be thrown if more than one target type is available,
        /// there is no propritisation procedure as this should be considered a
        /// logic error.
        MoreThanOneTargetAvailable,

        /// When a [`RenderTypeScan`] was successfully conducted yet the expected
        /// [required component/s](https://docs.rs/bevy/latest/bevy/ecs/prelude/trait.Component.html#required-components)
        /// were not found
        RequiredComponentNotAvailable,

        /// When using [`SourceType::Sprite`], you must set [`Sprite.custom_size`]
        SpriteCustomSizeNotSet,

        SpriteUnexpectedNormal,

        SpriteExpectedHitdataPosition,

        UiExpectedCursorPosition,
    }
}

pub(crate) use coords::*;
mod coords;
pub(crate) use output::*;
mod output;
pub(crate) use widget_size::*;
mod widget_size;

use bevy::ecs::query::QueryData;

use crate::prelude::*;

/// The top level UI text edit component
///
/// Adding [`TextEdit`] will pull in the required components for setting up
/// a text edit widget, notably [`CosmicEditBuffer`]
///
/// Hopefully this API will eventually mirror [`bevy::prelude::Text`].
/// See [`CosmicEditBuffer`] for more information.
#[derive(Component)]
#[require(ImageNode, Button, CosmicEditBuffer)]
pub struct TextEdit;

/// The top-level 2D text edit component
///
/// Adding [`TextEdit2d`] will pull in the required components for setting up
/// a 2D text editor using a [`Sprite`] with [`Sprite.custom_size`] set,
/// to set the size of the text editor add a [`Sprite`] component with
/// [`Sprite.custom_size`] set.
///
/// Hopefully this API will eventually mirror [`bevy::prelude::Text2d`].
/// See [`CosmicEditBuffer`] for more information.
#[derive(Component)]
#[require(Sprite, CosmicEditBuffer)]
pub struct TextEdit2d;

/// TODO: Generalize implementations depending on this
/// and add 3D
pub enum SourceType {
    Ui,
    Sprite,
}

#[derive(QueryData)]
pub struct RenderTypeScan {
    is_sprite: Has<TextEdit2d>,
    is_ui: Has<TextEdit>,
}

impl RenderTypeScanItem<'_> {
    pub fn scan(&self) -> Result<SourceType> {
        match (self.is_sprite, self.is_ui) {
            (true, false) => Ok(SourceType::Sprite),
            (false, true) => Ok(SourceType::Ui),
            (true, true) => Err(RenderTargetError::MoreThanOneTargetAvailable),
            (false, false) => Err(RenderTargetError::NoTargetsAvailable),
        }
    }
}

/// Function to find the location of the mouse cursor in a cosmic widget.
/// Returns in logical pixels
// TODO: Change this to use builtin `bevy::picking` instead
pub(crate) fn get_node_cursor_pos(
    window: &Window,
    node_transform: &GlobalTransform,
    size: Vec2,
    source_type: SourceType,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    let node_translation = node_transform.affine().translation;
    let node_bounds = Rect::new(
        node_translation.x - size.x / 2.,
        node_translation.y - size.y / 2.,
        node_translation.x + size.x / 2.,
        node_translation.y + size.y / 2.,
    );

    window.cursor_position().and_then(|pos| match source_type {
        SourceType::Ui => {
            if node_bounds.contains(pos) {
                Some(Vec2::new(
                    pos.x - node_bounds.min.x,
                    pos.y - node_bounds.min.y,
                ))
            } else {
                None
            }
        }
        SourceType::Sprite => camera
            .viewport_to_world_2d(camera_transform, pos)
            .ok()
            .and_then(|pos| {
                if node_bounds.contains(pos) {
                    Some(Vec2::new(
                        pos.x - node_bounds.min.x,
                        node_bounds.max.y - pos.y,
                    ))
                } else {
                    None
                }
            }),
    })
}
