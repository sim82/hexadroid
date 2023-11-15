struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
#ifdef VERTEX_TANGENTS
    [[location(3)]] world_tangent: vec4<f32>;
#endif
};

struct MyMaterial {
    alpha: f32;
    color: vec4<f32>;
};

[[group(1), binding(0)]]
var<uniform> uniform_data: MyMaterial;

[[stage(fragment)]]
fn fragment(input: VertexOutput) -> [[location(0)]] vec4<f32> {
    var output_color = vec4<f32>(input.uv, 0.0, uniform_data.alpha);
    output_color = output_color * uniform_data.color;
    return output_color;
}