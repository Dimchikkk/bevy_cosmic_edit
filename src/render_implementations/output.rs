use bevy::ecs::query::QueryData;
use render_implementations::prelude::*;

use crate::prelude::*;

/// Will attempt to find a place on the receiving entity to place
/// a [`Handle<Image>`]
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct OutputToEntity {
    scan: RenderTypeScan,

    sprite_target: Option<&'static mut Sprite>,
    image_node_target: Option<&'static mut ImageNode>,
}

impl<'s> std::ops::Deref for OutputToEntityItem<'s> {
    type Target = RenderTypeScanItem<'s>;

    fn deref(&self) -> &Self::Target {
        &self.scan
    }
}

impl OutputToEntityItem<'_> {
    pub fn write_image_data(&mut self, image: &Handle<Image>) -> Result<()> {
        match self.scan()? {
            SourceType::Sprite => {
                let sprite = self
                    .sprite_target
                    .as_mut()
                    .ok_or(RenderTargetError::required_component_missing::<Sprite>())?;
                sprite.image = image.clone_weak();
                Ok(())
            }
            SourceType::Ui => {
                let image_node = self
                    .image_node_target
                    .as_mut()
                    .ok_or(RenderTargetError::required_component_missing::<ImageNode>())?;
                image_node.image = image.clone_weak();
                Ok(())
            }
        }
    }
}
