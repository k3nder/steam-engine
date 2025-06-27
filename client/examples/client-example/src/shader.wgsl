// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normals: vec3<f32>,
    @location(5) model_0: vec4<f32>,
    @location(6) model_1: vec4<f32>,
    @location(7) model_2: vec4<f32>,
    @location(8) model_3: vec4<f32>,
    @location(9) color: vec4<f32>,
    @location(10) uv_offset: vec2<f32>,
    @location(11) uv_scale: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) uv_offset: vec2<f32>,
    @location(3) uv_scale: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    let model = mat4x4<f32>(
        input.model_0,
        input.model_1,
        input.model_2,
        input.model_3,
    );

    var out: VertexOutput;
    out.color = input.color;
    out.tex_coords = input.tex_coords;
    out.uv_offset = input.uv_offset;
    out.uv_scale = input.uv_scale;
    out.clip_position = camera * model * vec4<f32>(input.position, 1.0);
    return out;
}

// Fragment

@group(1) @binding(0)
var atlas_texture: texture_2d<f32>;

@group(1) @binding(1)
var atlas_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv_offset + in.tex_coords * in.uv_scale;
    return textureSample(atlas_texture, atlas_sampler, uv);
}
