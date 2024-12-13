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
    pub(super) use super::scan::{RenderTypeScan, RenderTypeScanItem, SourceType};
    pub(super) use super::RenderTargetError;
}

pub(crate) fn plugin(app: &mut App) {
    if !app.is_plugin_added::<MeshPickingPlugin>() {
        debug!("Adding MeshPickingPlugin manually as its not been added already");
        app.add_plugins(MeshPickingPlugin);
    }

    app.add_systems(PreUpdate, threed::sync_mesh_and_size)
        .add_systems(
            First,
            output::update_internal_target_handles
                .pipe(render_implementations::debug_error("update target handles")),
        )
        .register_type::<TextEdit3d>()
        .register_type::<output::CosmicRenderOutput>();
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

        UnexpectedNormal,

        ExpectedHitdataPosition,

        UiExpectedCursorPosition,

        Material3dDoesNotExist,
    }

    impl RenderTargetError {
        pub fn required_component_missing<C: bevy::prelude::Component>() -> Self {
            Self::RequiredComponentNotAvailable {
                debug_name: format!("{:?}", core::any::type_name::<C>()),
            }
        }
    }

    use crate::prelude::*;
    pub(crate) fn debug_error<T>(debug_name: &'static str) -> impl Fn(In<Result<T>>) {
        move |In(result): In<Result<T>>| match result {
            Ok(_) => {}
            Err(err) => debug!(?err, "Error in system {}", debug_name),
        }
    }
}

pub use size::WorldPixelRatio;

pub(crate) mod coords;
pub(crate) mod output;
pub(crate) mod scan;
pub(crate) mod size;
pub(crate) mod threed;

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

// #[cfg(feature = "3d")]
#[derive(Component, Reflect, Debug)]
#[require(Mesh3d, MeshMaterial3d::<StandardMaterial>, CosmicEditBuffer)]
pub struct TextEdit3d {
    pub size: Vec2,
}

impl TextEdit3d {
    pub fn new(size: Vec2) -> Self {
        Self { size }
    }
}
