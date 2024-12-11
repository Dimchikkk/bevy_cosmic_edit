use bevy::ecs::query::QueryData;

use crate::prelude::*;
use crate::render_implementations::prelude::*;

/// TODO: Generalize implementations depending on this
/// and add 3D
#[non_exhaustive]
pub(in crate::render_implementations) enum SourceType {
    Ui,
    Sprite,
    ThreeD,
}

#[derive(QueryData)]
pub struct RenderTypeScan {
    is_sprite: Has<TextEdit2d>,
    is_ui: Has<TextEdit>,
    is_3d: Has<TextEdit3d>,
}

impl RenderTypeScanItem<'_> {
    pub fn confirm_conformance(&self) -> Result<()> {
        match self.scan() {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub(in crate::render_implementations) fn scan(&self) -> Result<SourceType> {
        let flags = [self.is_ui, self.is_sprite, self.is_3d];
        let count_true = flags.iter().filter(|x| **x).count();
        match count_true {
            0 => Err(RenderTargetError::NoTargetsAvailable),
            1 => match flags {
                [true, false, false] => Ok(SourceType::Sprite),
                [false, true, false] => Ok(SourceType::Ui),
                [false, false, true] => Ok(SourceType::ThreeD),
                _ => unreachable!(),
            },
            _ => Err(RenderTargetError::MoreThanOneTargetAvailable),
        }
    }
}

pub(crate) fn debug_error<T>(In(result): In<Result<T>>) {
    match result {
        Ok(_) => {}
        Err(err) => debug!(message = "Error in render target", ?err),
    }
}
