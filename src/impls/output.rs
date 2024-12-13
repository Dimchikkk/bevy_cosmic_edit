use bevy::ecs::query::QueryData;
use impls::prelude::*;

use crate::prelude::*;

/// Used to ferry data from a [`CosmicEditBuffer`]
#[derive(Component, Default, Reflect, Debug, Deref)]
#[component(on_add = new_image_from_default)]
pub(crate) struct CosmicRenderOutput(pub(crate) Handle<Image>);

/// Without this, multiple buffers will show the same image
/// as the focussed editor. IDK why
fn new_image_from_default(
    mut world: bevy::ecs::world::DeferredWorld,
    entity: Entity,
    _: bevy::ecs::component::ComponentId,
) {
    let mut images = world.resource_mut::<Assets<Image>>();
    let image = Image::default();
    let default_image = images.add(image);
    *world
        .entity_mut(entity)
        .get_mut::<CosmicRenderOutput>()
        .unwrap() = CosmicRenderOutput(default_image);
}

/// Will attempt to find a place on the receiving entity to place
/// a [`Handle<Image>`]
#[derive(QueryData)]
#[query_data(mutable)]
pub(in crate::impls) struct OutputToEntity {
    scan: RenderTypeScan,

    sprite_target: Option<&'static mut Sprite>,
    image_node_target: Option<&'static mut ImageNode>,
    #[cfg(feature = "3d")]
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
        #[cfg(feature = "3d")] mats: &mut Assets<StandardMaterial>,
        // imgs: &mut Assets<Image>,
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
            #[cfg(feature = "3d")]
            SourceType::ThreeD => {
                let material_handle = self
                    .threed_target
                    .as_mut()
                    .ok_or(RenderTargetError::required_component_missing::<
                        MeshMaterial3d<StandardMaterial>,
                    >())?;
                // let mat = mats
                //     .get_mut(material_handle.0.id())
                //     .ok_or(RenderTargetError::Material3dDoesNotExist)?;

                // mat.base_color_texture = Some(image);

                // material_handle.0 = mats.add(image);
                let mut old_material = mats
                    .get(material_handle.id())
                    .ok_or(RenderTargetError::Material3dDoesNotExist)?
                    .clone();
                old_material.base_color_texture = Some(image);
                material_handle.0 = mats.add(old_material);

                Ok(())
            }
        }
    }
}

/// Every frame updates the output (in [`CosmicRenderOutput`]) to its receiver
/// on the same entity, e.g. [`Sprite`]
pub(in crate::impls) fn update_internal_target_handles(
    mut buffers_q: Query<(&CosmicRenderOutput, OutputToEntity), With<CosmicEditBuffer>>,
    #[cfg(feature = "3d")] mut mats: ResMut<Assets<StandardMaterial>>,
    // mut imgs: ResMut<Assets<Image>>,
) -> impls::Result<()> {
    for (CosmicRenderOutput(output_data), mut output_components) in buffers_q.iter_mut() {
        output_components.write_image_data(
            output_data.clone(),
            #[cfg(feature = "3d")]
            mats.deref_mut(),
            // imgs.deref_mut(),
        )?;
    }

    Ok(())
}
