#import bevy_pbr::forward_io::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> tile_width: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> tile_height: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> color_a: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> color_b: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let pos = in.world_position.xy;

    let checker = (i32(floor(pos.x / tile_width)) + i32(floor(pos.y / tile_height))) & 1;

    if checker == 0 {
        return color_a;
    } else {
        return color_b;
    }
}
