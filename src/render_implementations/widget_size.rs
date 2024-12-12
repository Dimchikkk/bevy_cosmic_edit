use bevy::ecs::query::{QueryData, QueryFilter};
use render_implementations::{RenderTypeScan, RenderTypeScanItem};

use crate::prelude::*;
use render_implementations::prelude::*;

/// Query the (logical) size of a widget
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

trait NodeSizeExt {
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
                    .ok_or(RenderTargetError::required_component_missing::<ComputedNode>())?;
                Ok(ui.logical_size())
            }
            SourceType::Sprite => {
                let sprite = self
                    .sprite
                    .ok_or(RenderTargetError::required_component_missing::<Sprite>())?;
                Ok(sprite
                    .custom_size
                    .ok_or(RenderTargetError::SpriteCustomSizeNotSet)?)
            }
        }
    }
}
