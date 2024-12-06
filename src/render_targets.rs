//! Generalizes over render target implementations.
//!
//! ## Sprite:
//! Requires [`Sprite`] component and requires [`Sprite.custom_size`] to be Some( non-zero )
//!
//! ## UI:
//! Requires [`ImageNode`] for rendering and [`Button`] for [`Interaction`]s
// TODO: Remove `CosmicWidgetSize`?

use bevy::ecs::query::{QueryData, QueryFilter};

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
pub(crate) enum SourceType {
    Ui,
    Sprite,
}

#[derive(Debug)]
pub(crate) enum RenderTargetError {
    /// When no recognized [SourceType] could be found
    NoTargetsAvailable,

    /// When more than one [SourceType] was detected.
    ///
    /// This will always be thrown if more than one target type is available,
    /// there is no propritisation procedure as this should be considered a
    /// logic error.
    MoreThanOneTargetAvailable,

    /// When a [`RenderTypeScan`] was conducted yet the expected
    /// [required component/s](https://docs.rs/bevy/latest/bevy/ecs/prelude/trait.Component.html#required-components)
    /// were found
    RequiredComponentNotAvailable,

    /// When using [SourceType::Sprite], you must set [Sprite.custom_size]
    SpriteCustomSizeNotSet,
}

type Result<T> = core::result::Result<T, RenderTargetError>;

#[derive(QueryData)]
pub(crate) struct RenderTypeScan {
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

/// Query the size of a widget using any [`SourceType`]
#[derive(QueryData)]
pub(crate) struct CosmicWidgetSize {
    scan: RenderTypeScan,
    sprite: Option<&'static Sprite>,
    ui: Option<&'static ComputedNode>,
}

/// Allows `.scan()` to be called on a [`CosmicWidgetSize`] through deref
impl<'s> std::ops::Deref for CosmicWidgetSizeItem<'s> {
    type Target = RenderTypeScanItem<'s>;

    fn deref(&self) -> &Self::Target {
        &self.scan
    }
}

/// An optimization [`QueryFilter`](bevy::ecs::query::QueryFilter)
#[derive(QueryFilter)]
pub(crate) struct ChangedCosmicWidgetSize {
    sprite: Changed<Sprite>,
    ui: Changed<ComputedNode>,
}

pub(crate) trait NodeSizeExt {
    fn logical_size(&self) -> Vec2;
}

impl NodeSizeExt for ComputedNode {
    fn logical_size(&self) -> Vec2 {
        self.size() * self.inverse_scale_factor()
    }
}

impl CosmicWidgetSizeItem<'_> {
    pub fn logical_size(&self) -> Result<Vec2> {
        let source_type = self.scan.scan()?;
        match source_type {
            SourceType::Ui => {
                let ui = self
                    .ui
                    .ok_or(RenderTargetError::RequiredComponentNotAvailable)?;
                Ok(ui.logical_size())
            }
            SourceType::Sprite => {
                let sprite = self
                    .sprite
                    .ok_or(RenderTargetError::RequiredComponentNotAvailable)?;
                Ok(sprite
                    .custom_size
                    .ok_or(RenderTargetError::SpriteCustomSizeNotSet)?)
            }
        }
    }
}
