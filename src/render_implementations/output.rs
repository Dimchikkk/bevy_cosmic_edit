use bevy::ecs::query::QueryData;

use crate::prelude::*;

/// Will attempt to find a place on the receiving entity to place
/// a [`Handle<Image>`]
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct OutputToEntity {
    sprite_target: Option<&'static mut Sprite>,
    image_node_target: Option<&'static mut ImageNode>,
}

impl OutputToEntityItem<'_> {
    pub fn write_image_data(&mut self, image: &Handle<Image>) {
        if let Some(sprite) = self.sprite_target.as_mut() {
            sprite.image = image.clone_weak();
        }
        if let Some(image_node) = self.image_node_target.as_mut() {
            image_node.image = image.clone_weak();
        }
    }
}
