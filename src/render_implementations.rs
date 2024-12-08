//! Generalizes over render target implementations.
//!
//! ## Sprite:
//! Requires [`Sprite`] component and requires [`Sprite.custom_size`] to be Some( non-zero )
//!
//! ## UI:
//! Requires [`ImageNode`] for rendering and [`Button`] for [`Interaction`]s
// TODO: Remove `CosmicWidgetSize`?

pub mod output;

use bevy::{
    ecs::query::{QueryData, QueryFilter},
    window::SystemCursorIcon,
    winit::cursor::CursorIcon,
};

use crate::{prelude::*, primary::CameraFilter, HoverCursor, TextHoverIn, TextHoverOut};

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
}

type Result<T> = core::result::Result<T, RenderTargetError>;

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

/// Query the size of a widget using any [`SourceType`]
#[derive(QueryData)]
pub struct CosmicWidgetSize {
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
    /// Automatically logs any errors
    pub fn logical_size(&self) -> Result<Vec2> {
        let ret = self._logical_size();
        if let Err(err) = &ret {
            debug!(message = "Finding the size of a widget failed", ?err);
        }
        ret
    }

    fn _logical_size(&self) -> Result<Vec2> {
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

pub(crate) fn hover_sprites(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut cosmic_edit_query: Query<
        (&mut Sprite, &Visibility, &GlobalTransform, &HoverCursor),
        With<CosmicEditBuffer>,
    >,
    camera_q: Query<(&Camera, &GlobalTransform), CameraFilter>,
    mut hovered: Local<bool>,
    mut last_hovered: Local<bool>,
    mut evw_hover_in: EventWriter<TextHoverIn>,
    mut evw_hover_out: EventWriter<TextHoverOut>,
) {
    *hovered = false;
    if windows.iter().len() == 0 {
        return;
    }
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    let mut icon = CursorIcon::System(SystemCursorIcon::Default);

    for (sprite, visibility, node_transform, hover) in &mut cosmic_edit_query.iter_mut() {
        if visibility == Visibility::Hidden {
            continue;
        }

        let size = sprite.custom_size.unwrap_or(Vec2::ONE);
        if crate::render_implementations::get_node_cursor_pos(
            window,
            node_transform,
            size,
            SourceType::Sprite,
            camera,
            camera_transform,
        )
        .is_some()
        {
            *hovered = true;
            icon = hover.0.clone();
        }
    }

    if *last_hovered != *hovered {
        if *hovered {
            evw_hover_in.send(TextHoverIn(icon));
        } else {
            evw_hover_out.send(TextHoverOut);
        }
    }

    *last_hovered = *hovered;
}
