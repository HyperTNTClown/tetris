//!include util.wgsl

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
    out.coord = fma(model.position, vec2<f32>(0.5, -0.5), vec2<f32>(0.5, 0.5));
    out.position = vec4<f32>(model.position.xy, 0.0, 1.0);
    out.position.y -= 0.5;
    out.position.y *= 2.0;
    return out;
}

struct Surface {
    sd: f32,
    col: vec3<f32>
};

struct Overstep {
    os: f32,
    sf: Surface,
};

struct Uniforms {
    mouse: vec2<f32>,
    time: f32,
    window_size: vec2<f32>,
    scale: f32,
    window_scale: f32
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct Drawable {
    position: vec3<f32>,
    shape_data: array<f32, 8>,
    shape: u32,
}

@group(1) @binding(0)
var<storage, read> drawables: array<Drawable, 256>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = uniforms.window_size.x / uniforms.window_size.y;

    var uv = in.coord.xy;

    uv.y = .5 - uv.y;
    uv = uv * 2. - vec2<f32>(1., 0.5);
    uv.x *= aspect;

    // uv.y = .5 - uv.y;
    // uv = uv * 2. - vec2<f32>(1., 0.5);
    // uv.x *= aspect;
    // finally works...

    let ray_origin = vec3 < f32 > (0., 0., -3.);
    let ray_direction = normalize(vec3 < f32 > (uv, 1.));
    var col = vec3 < f32 > (0.);

    var t : Surface;
    t.sd = 0.;


    for (var i = 0; i < 60; i = i + 1)
    {
        var p = ray_origin + ray_direction * t.sd;
        var d = scene(p);
        var board = board(p);
        d = opUnion(d, board);

        t.col = d.col;
        t.sd += d.sd;
        if (t.sd > 1000.0 || d.sd < 0.001) {
            if (t.sd > 1000.0) {
                t.col = vec3<f32>(0.0, 0.0, 0.0);
            }
            col = vec3 < f32 > (t.col * (1.0 - f32(i) / 60.0));
            break;
        }
    }

    var rand = (fract(sin(dot(uv, vec2<f32>(12.9898, 78.233))*uniforms.time) * 43758.5453) - 0.5) * 2.0;

    col = mix(col, vec3<f32>(rand), 0.01*rand);

    var vignette = 1.0 - length(uv * vec2<f32>(1.0, aspect));
    col = mix(vec3<f32>(0.0), col, vignette);

    var moving_scalines = sin(uv.y * 100.0 + uniforms.time * 10.0) * 0.01;
    col = mix(col, vec3<f32>(.75), (moving_scalines)*sin(rand*100.0 + uniforms.time * 10.0));


    //return vec4<f32>(uv.xy, col.x, 1.0);
    return vec4<f32>(col, 1.0);
}

fn scene(p: vec3<f32>) -> Surface {
    var res: Surface;
    res.sd = 1000.;
    for (var i = 0; i < 32; i = i + 1) {
        var d = drawables[i];
        if (d.shape == u32(0)) {
            return res;
        } else if (d.shape == u32(1)) {
            var sphere_pos = d.position;
            var radius = d.shape_data[0];
            var sphere = sdSphere(p - sphere_pos, radius, vec3<f32>(1.0, 0.0, 0.0));
            if (sphere.sd < res.sd) {
                res.sd = sphere.sd;
                res.col = vec3<f32>(1.0, 0., 0.);
            }
        } else if (d.shape == u32(2)) {
            var box_pos = d.position;
            var box_size = vec3<f32>(d.shape_data[0], d.shape_data[1], d.shape_data[2]);
            var box = sdBox(p - box_pos, box_size, vec3<f32>(1.0, 0.0, 0.0));
            if (box.sd < res.sd) {
                res.sd = box.sd;
                res.col = vec3<f32>(d.shape_data[3], d.shape_data[4], d.shape_data[5]);
            }
        }

        if (res.sd < 0.001) {
            return res;
        }
    }

    return res;

}

fn board(p: vec3<f32>) -> Surface {
    var board_pos = vec3<f32>(0.0, -0.5, 5.0);
    var board_size = vec3<f32>(1.0, 0.5, 1.0);
    var board = sdBox(p - tetris_pos_to_world_pos(vec2<f32>(4.0, 5.)), board_size, vec3<f32>(1.0, 0.0, 0.0));

    return board;
}

fn tetris_pos_to_world_pos(pos: vec2<f32>) -> vec3<f32> {
    var world_pos = vec3<f32>(0.0, 0.0, 0.0);
    world_pos.x = pos.x * 0.5;
    world_pos.y = pos.y * 0.5;
    world_pos.z = 6.0;
    return world_pos;
}


/// Function for having two surfaces saved and blending them using smooth union,
/// will currently not be using it
/*
// fn scene(p: vec3<f32>) -> Surface {
//     var res: Surface;
//     res.sd = 1000.;
//     var res2: Surface;
//     res2.sd = 1000.;
//
//     for (var i = 0; i < 32; i = i + 1) {
//         var d = drawables[i];
//         if (d.shape == u32(0)) {
//             return opSmoothUnion(res, res2, 1.0);
//         } else if (d.shape == u32(1)) {
//             var sphere_pos = d.position;
//             var radius = d.shape_data[0];
//             var sphere = sdSphere(p - sphere_pos, radius, vec3<f32>(1.0, 0.0, 0.0));
//             if (sphere.sd < res.sd) {
//                 res2 = res;
//                 res = sphere;
//             } else if (sphere.sd < res2.sd) {
//                 res2 = sphere;
//             }
//         } else if (d.shape == u32(2)) {
//             var box_pos = d.position;
//             var box_size = vec3<f32>(d.shape_data[0], d.shape_data[1], d.shape_data[2]);
//             var box = sdBox(p - box_pos, box_size, vec3<f32>(1.0, 0.0, 0.0));
//             if (box.sd < res.sd) {
//                 res2 = res;
//                 res = box;
//             } else if (box.sd < res2.sd) {
//                 res2 = box;
//             }
//         }
//
//         if (res.sd < 0.001) {
//             return opSmoothUnion(res, res2, 1.0);
//         }
//     }
//
//     return opSmoothUnion(res, res2, 1.0);
// }
*/