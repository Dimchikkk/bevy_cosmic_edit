//! Generalizes over render target implementations.
//!
//! ## UI:
//! Requires [`Sprite`] component and requires [`Sprite.custom_size`] to be Some( non-zero )
// TODO: Remove `CosmicWidgetSize`?

use bevy::ecs::query::{QueryData, QueryFilter};

use crate::prelude::*;

/// TODO: Generalize implementations depending on this
/// and add 3D
pub(crate) enum SourceType {
    Ui,
    Sprite,
}

#[derive(Debug)]
pub enum RenderTargetError {
    /// When no recognized [SourceType] could be found
    NoTargetsAvailable,

    /// When more than one [SourceType] was detected.
    ///
    /// This will always be thrown if more than one target type is available,
    /// there is no propritisation procedure as this should be considered a
    /// logic error.
    MoreThanOneTargetAvailable,

    /// When using [SourceType::Sprite], you must set [Sprite.custom_size]
    SpriteCustomSizeNotSet,
}

type Result<T> = core::result::Result<T, RenderTargetError>;

#[derive(QueryData)]
pub(crate) struct CosmicWidgetSize {
    sprite: Option<&'static Sprite>,
    ui: Option<&'static ComputedNode>,
}

#[derive(QueryFilter)]
pub(crate) struct ChangedCosmicWidgetSize {
    sprite: Changed<Sprite>,
    ui: Changed<ComputedNode>,
}

impl CosmicWidgetSizeItem<'_> {
    pub fn source_type(&self) -> Result<SourceType> {
        match (self.sprite, self.ui) {
            (Some(_), Some(_)) => Err(RenderTargetError::MoreThanOneTargetAvailable),
            (None, None) => Err(RenderTargetError::NoTargetsAvailable),
            (Some(_), None) => Ok(SourceType::Sprite),
            (None, Some(_)) => Ok(SourceType::Ui),
        }
    }

    pub fn logical_size(&self) -> Result<Vec2> {
        match (self.sprite, self.ui) {
            (Some(_), Some(_)) => Err(RenderTargetError::MoreThanOneTargetAvailable),
            (None, None) => Err(RenderTargetError::NoTargetsAvailable),
            (Some(sprite), None) => {
                let sprite_size = sprite
                    .custom_size
                    .ok_or(RenderTargetError::SpriteCustomSizeNotSet)?;
                Ok(sprite_size)
            }
            (None, Some(ui)) => Ok(ui.logical_size()),
        }
    }
}
