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
//!
//! ## 3D: [`TextEdit3d`]
//! Automatically initializes the [`Mesh3d`] to a plane centered at the origin (default [`Transform`])
//! with its face normal being [`Vec3::Z`] and size being [`world_size`](crate::TextEdit3d.world_size)
//!
//! Continuously updates the `MeshMaterial3d<StandardMaterial>` to the latest [`CosmicEditBuffer`]
//! [`Handle<Image>`]. It clones the previous material and only touches the `base_color_texture` field.
//!

pub use scan::{RenderTypeScan, SourceType};
pub use size::WorldPixelRatio;

pub(crate) mod coords;
pub(crate) mod output;
pub(crate) mod scan;
pub(crate) mod size;
pub(crate) mod threed;

mod prelude {
    pub(super) use super::error::Result;
    pub(super) use super::scan::{RenderTypeScan, RenderTypeScanItem, SourceType};
    pub(super) use super::RenderTargetError;
}
#[cfg(doc)]
use prelude::*;

pub(crate) fn plugin(app: &mut App) {
    if !app.is_plugin_added::<bevy::picking::mesh_picking::MeshPickingPlugin>() {
        debug!("Adding MeshPickingPlugin manually as its not been added already");
        app.add_plugins(bevy::picking::mesh_picking::MeshPickingPlugin);
    }

    app.add_systems(PreUpdate, threed::sync_mesh_and_size)
        .add_systems(
            First,
            output::update_internal_target_handles
                .pipe(impls::debug_error("update target handles")),
        )
        .register_type::<TextEdit3d>()
        .register_type::<output::CosmicRenderOutput>();
}

pub use error::*;
mod error {
    #[cfg(doc)]
    use impls::prelude::*;

    pub type Error = crate::impls::RenderTargetError;
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

/// The top-level driving component for 3D text editing
// #[cfg(feature = "3d")]
#[derive(Component, Reflect, Debug)]
#[require(Mesh3d, MeshMaterial3d::<StandardMaterial>, CosmicEditBuffer)]
#[component(on_add = default_3d_material)]
pub struct TextEdit3d {
    /// The size in world pixels of the text editor.
    ///
    /// See [`WorldPixelRatio`] for more information.
    pub world_size: Vec2,

    /// Recommended, defaults to `true`.
    /// See [crate::impls] for more information.
    pub auto_manage_mesh: bool,
}

impl TextEdit3d {
    pub fn new(rendering_size: Vec2) -> Self {
        Self {
            world_size: rendering_size,
            auto_manage_mesh: true,
        }
    }
}

fn default_3d_material(
    mut world: bevy::ecs::world::DeferredWorld,
    target: Entity,
    _: bevy::ecs::component::ComponentId,
) {
    let current_handle = world
        .get::<MeshMaterial3d<StandardMaterial>>(target)
        .unwrap()
        .0
        .clone();
    if current_handle == Handle::default() {
        debug!("It appears no customization of a `TextEdit3d` material has been done, overwriting with a default");
        let default_material = StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        };
        let default_handle = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(default_material);
        world
            .get_mut::<MeshMaterial3d<StandardMaterial>>(target)
            .unwrap()
            .0 = default_handle;
    }
}
