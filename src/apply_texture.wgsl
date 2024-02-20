
struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) coord: vec2<f32>,
};


@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    //out.coord = vertices[model.vertex_index];
    out.coord = fma(model.position, vec2<f32>(0.5, -0.5), vec2<f32>(0.5, 0.5));
    //out.position = vec4<f32>(model.position.xyz, 1.0);
    out.position = vec4<f32>(model.position.xy, 0.0, 1.0);
    return out;
}

@group(1) @binding(0)
var tex_sampler: sampler;

@group(1) @binding(1)
var tex_coords: texture_2d<f32>;


struct Uniforms {
    mouse: vec2<f32>,
    time: f32,
    glitch: f32,
    window_size: vec2<f32>,
    scale: f32,
    window_scale: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = (in.position.xy) / (uniforms.window_size.xy);
    var col = textureSample(tex_coords, tex_sampler, uv.xy);

    if (uniforms.glitch > 0.0) {
        var rand = (sin(uniforms.time) * 43758.5453 + cos(uniforms.time) * 76938.3243) - floor((sin(uniforms.time) * 43758.5453 + cos(uniforms.time) * 76938.3243));
        var enable_shift = 0.0;
        if (rand < 0.2) { enable_shift = 1.5*uniforms.glitch/3.; };
        col.r = mix(col.r, textureSample(tex_coords, tex_sampler, uv + vec2<f32>( 0.025*rand,  0.025*rand)).r, enable_shift);
        col.b = mix(col.b, textureSample(tex_coords, tex_sampler, uv + vec2<f32>(-0.025*rand, -0.025*rand)).b, enable_shift);
        col.g = mix(col.g, textureSample(tex_coords, tex_sampler, uv + vec2<f32>( 0.025*rand, -0.025*rand)).g, enable_shift);
    }

    return col;
}