use bevy::ecs::query::QueryData;

use crate::impls::prelude::*;
use crate::prelude::*;

/// TODO: Generalize implementations depending on this
/// and add 3D
#[non_exhaustive]
pub enum SourceType {
    Ui,
    Sprite,
    #[cfg(feature = "3d")]
    ThreeD,
}

/// An internal type to work out what [`SourceType`] a widget is
#[derive(QueryData)]
pub struct RenderTypeScan {
    is_sprite: Has<TextEdit2d>,
    is_ui: Has<TextEdit>,
    #[cfg(feature = "3d")]
    is_3d: Has<TextEdit3d>,
}

impl RenderTypeScanItem<'_> {
    pub fn confirm_conformance(&self) -> Result<()> {
        match self.scan() {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn is_3d(&self) -> bool {
        #[cfg(feature = "3d")]
        let ret = self.is_3d;

        #[cfg(not(feature = "3d"))]
        let ret = false;

        ret
    }

    pub(in crate::impls) fn scan(&self) -> Result<SourceType> {
        let flags = [self.is_ui, self.is_sprite, self.is_3d()];
        let count_true = flags.iter().filter(|x| **x).count();
        match count_true {
            0 => Err(RenderTargetError::NoTargetsAvailable),
            1 => match flags {
                [true, false, false] => Ok(SourceType::Ui),
                [false, true, false] => Ok(SourceType::Sprite),
                #[cfg(feature = "3d")]
                [false, false, true] => Ok(SourceType::ThreeD),
                #[cfg(not(feature = "3d"))]
                [false, false, false] => unreachable!(),
                _ => unreachable!(),
            },
            _ => Err(RenderTargetError::MoreThanOneTargetAvailable),
        }
    }
}
