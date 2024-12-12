//! Generalizes over render target implementations. All code that
//! depends on the specific render target implementation should
//! live in this module.
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
        RequiredComponentNotAvailable {
            debug_name: String,
        },

        /// When using [`SourceType::Sprite`], you must set [`Sprite.custom_size`]
        SpriteCustomSizeNotSet,

        SpriteUnexpectedNormal,

        SpriteExpectedHitdataPosition,

        UiExpectedCursorPosition,
    }

    impl RenderTargetError {
        pub fn required_component_missing<C: bevy::prelude::Component>() -> Self {
            Self::RequiredComponentNotAvailable {
                debug_name: format!("{:?}", core::any::type_name::<C>()),
            }
        }
    }
}

pub(crate) use coords::*;
mod coords;
pub(crate) use output::*;
mod output;
pub(crate) use widget_size::*;
mod widget_size;
pub(crate) use scan::*;
mod scan;

use crate::prelude::*;

/// The top level UI text edit component
///
/// Adding [`TextEdit`] will pull in the required components for setting up
/// a text edit widget, notably [`CosmicEditBuffer`]
///
/// Hopefully this API will eventually mirror [`bevy::prelude::Text`].
/// See [`CosmicEditBuffer`] for more information.
#[derive(Component)]
#[require(ImageNode, Button, bevy::ui::RelativeCursorPosition, CosmicEditBuffer)]
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
