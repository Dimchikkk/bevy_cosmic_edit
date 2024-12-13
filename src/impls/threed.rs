use crate::prelude::*;

pub(in crate::impls) fn sync_mesh_and_size(
    mut meshs: Query<(&TextEdit3d, &mut Mesh3d), (With<CosmicEditBuffer>, Changed<TextEdit3d>)>,
    // mut asset_server: ResMut<Assets<Image>>,
    mut mesh_server: ResMut<Assets<Mesh>>,
) {
    for (size, mut mesh_component) in meshs.iter_mut() {
        if !size.auto_manage_mesh {
            continue;
        }

        let size = size.world_size;
        // let mesh = Plane3d::new(Vec3::Z, size / 2.0);
        let mesh = Rectangle::new(size.x, size.y);
        mesh_component.0 = mesh_server.add(mesh);
    }
}

impl TextEdit3d {
    pub fn new(rendering_size: Vec2) -> Self {
        Self {
            world_size: rendering_size,
            auto_manage_mesh: true,
        }
    }
}

pub(super) fn default_3d_material(
    mut world: bevy::ecs::world::DeferredWorld,
    target: Entity,
    _: bevy::ecs::component::ComponentId,
) {
    let current_handle = world
        .get::<MeshMaterial3d<StandardMaterial>>(target)
        .unwrap()
        .0
        .clone();
    if current_handle == Handle::default() {
        debug!("It appears no customization of a `TextEdit3d` material has been done, overwriting with a default");
        let default_material = StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        };
        let default_handle = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(default_material);
        world
            .get_mut::<MeshMaterial3d<StandardMaterial>>(target)
            .unwrap()
            .0 = default_handle;
    }
}
