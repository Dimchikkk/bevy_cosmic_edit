use bevy::ecs::query::QueryData;

use crate::prelude::*;
use crate::render_implementations::prelude::*;

/// TODO: Generalize implementations depending on this
/// and add 3D
#[non_exhaustive]
pub(in crate::render_implementations) enum SourceType {
    Ui,
    Sprite,
}

#[derive(QueryData)]
pub struct RenderTypeScan {
    is_sprite: Has<TextEdit2d>,
    is_ui: Has<TextEdit>,
}

impl RenderTypeScanItem<'_> {
    pub fn confirm_conformance(&self) -> Result<()> {
        match self.scan() {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub(in crate::render_implementations) fn scan(&self) -> Result<SourceType> {
        match (self.is_sprite, self.is_ui) {
            (true, false) => Ok(SourceType::Sprite),
            (false, true) => Ok(SourceType::Ui),
            (true, true) => Err(RenderTargetError::MoreThanOneTargetAvailable),
            (false, false) => Err(RenderTargetError::NoTargetsAvailable),
        }
    }
}

pub(crate) fn debug_error<T>(In(result): In<Result<T>>) {
    match result {
        Ok(_) => {}
        Err(err) => debug!(message = "Error in render target", ?err),
    }
}
