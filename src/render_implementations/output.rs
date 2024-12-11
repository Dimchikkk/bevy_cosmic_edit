use bevy::ecs::query::QueryData;
use render_implementations::prelude::*;

use crate::prelude::*;

/// Will attempt to find a place on the receiving entity to place
/// a [`Handle<Image>`]
#[derive(QueryData)]
#[query_data(mutable)]
pub(in crate::render_implementations) struct OutputToEntity {
    scan: RenderTypeScan,

    sprite_target: Option<&'static mut Sprite>,
    image_node_target: Option<&'static mut ImageNode>,
    threed_target: Option<&'static mut MeshMaterial3d<StandardMaterial>>,
}

impl<'s> std::ops::Deref for OutputToEntityItem<'s> {
    type Target = RenderTypeScanItem<'s>;

    fn deref(&self) -> &Self::Target {
        &self.scan
    }
}

impl OutputToEntityItem<'_> {
    pub fn write_image_data(
        &mut self,
        image: Handle<Image>,
        mats: &mut Assets<StandardMaterial>,
    ) -> Result<()> {
        match self.scan()? {
            SourceType::Sprite => {
                let sprite = self
                    .sprite_target
                    .as_mut()
                    .ok_or(RenderTargetError::required_component_missing::<Sprite>())?;
                sprite.image = image;
                Ok(())
            }
            SourceType::Ui => {
                let image_node = self
                    .image_node_target
                    .as_mut()
                    .ok_or(RenderTargetError::required_component_missing::<ImageNode>())?;
                image_node.image = image;
                Ok(())
            }
            SourceType::ThreeD => {
                let material_handle = self
                    .threed_target
                    .as_mut()
                    .ok_or(RenderTargetError::required_component_missing::<
                        MeshMaterial3d<StandardMaterial>,
                    >())?;
                let mat = mats
                    .get_mut(material_handle.0.id())
                    .ok_or(RenderTargetError::Material3dDoesNotExist)?;

                mat.base_color_texture = Some(image);
                Ok(())
            }
        }
    }
}

/// Every frame updates the output (in [`CosmicRenderOutput`]) to its receiver
/// on the same entity, e.g. [`Sprite`]
pub(in crate::render_implementations) fn update_internal_target_handles(
    mut buffers_q: Query<(&CosmicRenderOutput, OutputToEntity), With<CosmicEditBuffer>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) -> render_implementations::Result<()> {
    for (CosmicRenderOutput(output_data), mut output_components) in buffers_q.iter_mut() {
        output_components.write_image_data(output_data.clone(), mats.deref_mut())?;
    }

    Ok(())
}
