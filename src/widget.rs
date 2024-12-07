use crate::prelude::*;

/// System set for cosmic text layout systems. Runs in [`PostUpdate`]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct WidgetSet;

pub(crate) struct WidgetPlugin;

impl Plugin for WidgetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (new_image_from_default,)
                .chain()
                .in_set(WidgetSet)
                .after(TransformSystem::TransformPropagate),
        );
    }
}

/// Instantiates a new image for a [`CosmicEditBuffer`]
fn new_image_from_default(
    mut query: Query<&mut CosmicRenderOutput, Added<CosmicEditBuffer>>,
    mut images: ResMut<Assets<Image>>,
) {
    for mut canvas in query.iter_mut() {
        debug!(message = "Initializing a new canvas");
        *canvas = CosmicRenderOutput(images.add(Image::default()));
    }
}
