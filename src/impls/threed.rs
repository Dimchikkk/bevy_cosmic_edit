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
