use crate::prelude::*;

pub(in crate::render_implementations) fn sync_mesh_and_size(
    mut meshs: Query<(&TextEdit3d, &mut Mesh3d), (With<CosmicEditBuffer>, Changed<TextEdit3d>)>,
    // mut asset_server: ResMut<Assets<Image>>,
    mut mesh_server: ResMut<Assets<Mesh>>,
) {
    for (size, mut mesh_component) in meshs.iter_mut() {
        let size = size.size;
        let mesh = Plane3d::new(Vec3::Z, size / 2.0);
        mesh_component.0 = mesh_server.add(mesh);
    }
}
