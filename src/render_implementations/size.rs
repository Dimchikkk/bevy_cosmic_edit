use bevy::ecs::query::{QueryData, QueryFilter};
use render_implementations::scan::{RenderTypeScan, RenderTypeScanItem};

use crate::prelude::*;
use render_implementations::prelude::*;

/// Pixel / World ratio
/// E.g. 20 => 20 text pixels are rendered = 1 world pixel
#[derive(Component, Deref, DerefMut, Debug, Clone, Copy)]
pub struct WorldPixelRatio(pub f32);

impl Default for WorldPixelRatio {
    fn default() -> Self {
        WorldPixelRatio(1.0)
    }
}

impl WorldPixelRatio {
    pub fn from_one_world_pixel_equals(text_pixels: f32) -> Self {
        WorldPixelRatio(text_pixels)
    }
}

/// Query the (logical) size of a widget
#[derive(QueryData)]
pub struct CosmicWidgetSize {
    scan: RenderTypeScan,
    ratio: &'static WorldPixelRatio,

    sprite: Option<&'static Sprite>,
    ui: Option<&'static ComputedNode>,
    threed: Option<&'static TextEdit3d>,
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
    scan: Or<(Changed<TextEdit>, Changed<TextEdit2d>, Changed<TextEdit3d>)>,
    ratio: Changed<WorldPixelRatio>,
    sprite: Changed<Sprite>,
    ui: Changed<ComputedNode>,
    threed: Changed<TextEdit3d>,
}

pub(in crate::render_implementations) trait NodeSizeExt {
    fn world_size(&self) -> Vec2;
}

impl NodeSizeExt for ComputedNode {
    fn world_size(&self) -> Vec2 {
        self.size() * self.inverse_scale_factor()
    }
}

impl CosmicWidgetSizeItem<'_> {
    pub fn world_size(&self) -> Result<Vec2> {
        let source_type = self.scan.scan()?;
        match source_type {
            SourceType::Ui => {
                let ui = self
                    .ui
                    .ok_or(RenderTargetError::required_component_missing::<ComputedNode>())?;
                Ok(ui.world_size())
            }
            SourceType::Sprite => {
                let sprite = self
                    .sprite
                    .ok_or(RenderTargetError::required_component_missing::<Sprite>())?;
                Ok(sprite
                    .custom_size
                    .ok_or(RenderTargetError::SpriteCustomSizeNotSet)?)
            }
            SourceType::ThreeD => {
                let threed = self
                    .threed
                    .ok_or(RenderTargetError::required_component_missing::<TextEdit3d>())?;
                Ok(threed.size)
            }
        }
    }

    pub fn pixel_render_size(&self) -> Result<Vec2> {
        let world_size = self.world_size()?;
        let ratio = self.ratio.deref();

        Ok(world_size * ratio)
    }
}
