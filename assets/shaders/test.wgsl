#import bevy_sprite::mesh2d_view_bindings  globals
#import bevy_sprite::mesh2d_vertex_output  MeshVertexOutput

struct CustomMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;
@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;

@fragment
fn fragment(
    mesh: MeshVertexOutput,
) -> @location(0) vec4<f32> {

let t_1 = sin(globals.time*2.0)*0.5+0.5;
let t_2 = cos(globals.time*2.0);

let color: vec4<f32> = vec4(t_1, t_2, t_1, 1.0);

    return color * textureSample(base_color_texture, base_color_sampler, mesh.uv);
}
