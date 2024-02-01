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
    position: vec4<f32>,
    shape_data: vec4<f32>,
    shape_data2: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> drawables: array<Drawable, 256>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = uniforms.window_size.x / uniforms.window_size.y;

    //var uv = in.coord.xy;

    //uv.y = .5 - uv.y;
    //uv = uv * 2. - vec2<f32>(1., 0.5);
    //uv.x *= aspect;

    var uv = (vec2<f32>(in.coord.x, .5 - in.coord.y) * 2. - vec2<f32>(1., 0.5)) * aspect;

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
    for (var i = 0; i < 256; i = i + 1) {
        var d = drawables[i];
        if (d.shape_data2.w == 0.0) {
            return res;
        } else if (d.shape_data2.w == 1.0) {
            var sphere_pos = d.position.xyz;
            var radius = d.shape_data[0];
            var sphere = sdSphere(p - sphere_pos, radius, vec3<f32>(1.0, 0.0, 0.0));
            if (sphere.sd < res.sd) {
                res.sd = sphere.sd;
                res.col = vec3<f32>(1.0, 0., 0.);
            }
        } else if (d.shape_data2.w == 2.0) {
            var box_pos = tetris_pos_to_world_pos(d.position.xy);
            var box_size = vec3<f32>(0.125, 0.125, .05);
            var box = sdBox(p - box_pos, box_size, vec3<f32>(1.0, 0.0, 0.0));
            if (box.sd < res.sd) {
                res.sd = box.sd;
                res.col = vec3<f32>(d.shape_data2.x, d.shape_data2.y, d.shape_data2.z);
            }
        }

        if (res.sd < 0.001) {
            return res;
        }
    }

    return res;

}

fn board(p: vec3<f32>) -> Surface {
    var field_size = vec3<f32>(0.125, 0.125, 0.01);
    var b : Surface;
    b.sd = 1000.;
    for (var i = 0.; i < 10.; i = i + 1.) {
        for (var j = 0.; j < 20.; j = j + 1.) {
            var pos = vec2<f32>(i, j);
            var board_pos = tetris_pos_to_world_pos(pos);
            var board = sdBox(p - board_pos, field_size, vec3<f32>(.25, 0.3, 0.4));
            b = opUnion(b, board);
        }
    }

    return b;
}

/// 10 x 20 grid for tetris
/// algined bottom left corner at 0,0
/// each block is 0.125 x 0.125
/// small offset between blocks of 0.025
fn tetris_pos_to_world_pos(pos: vec2<f32>) -> vec3<f32> {
    // Centered - finally
    var board_origin = vec3<f32>(-1.5, -3.125, 5.0);
    var grid_field_size = vec3<f32>(0.125, 0.125, 0.0);
    var grid_field_offset = vec3<f32>(0.2, 0.2, 0.0);

    var world_pos = board_origin + vec3<f32>(pos.x, pos.y, 0.0) * grid_field_size + vec3<f32>(pos.x, pos.y, 0.0) * grid_field_offset;
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

// Surface Definition:
//
// struct Surface {
//     sd: f32,
//     col: vec3<f32>
// };

// Most of the SDF code is shamelessly stolen from:
// https://gist.github.com/rozgo/a3ed36377283ce329c759f8f3ace624b

fn sdBox(p : vec3 < f32>, b : vec3 < f32>, col: vec3<f32>) -> Surface {
    var surface: Surface;
    let q = abs(p) - b;
    surface.sd = length(max(q, vec3 < f32 > (0.))) + min(max(q.x, max(q.y, q.z)), 0.);
    surface.col = col;
    return surface;
}

fn sdSphere(p : vec3 < f32>, s : f32, col: vec3<f32>) -> Surface
{
    var surface: Surface;
    surface.sd = length(p) - s;
    surface.col = col;
    return surface;
}

fn sdHexPrism(p: vec3<f32>, h: vec2<f32>, col: vec3<f32>) -> Surface {
  let k = vec3<f32>(-0.8660254, 0.5, 0.57735);
  let a = abs(p);
  let v = a.xy - 2. * min(dot(k.xy, a.xy), 0.) * k.xy;
  let d1 = length(v - vec2<f32>(clamp(v.x, -k.z * h.x, k.z * h.x), h.x)) * sign(v.y - h.x);
  let d2 = a.z - h.y;
  var surface: Surface;
  surface.sd = min(max(d1, d2), 0.) + length(max(vec2<f32>(d1, d2), vec2<f32>(0.)));
  surface.col = col;
  return surface;
}

fn sdInfiniteCone(p: vec3<f32>, sincos: vec2<f32>, col: vec3<f32>) -> Surface {
  let q = vec2<f32>(length(p.xz), -p.y);
  let d = length(q - sincos * max(dot(q, sincos), 0.));
  var surface: Surface;
  surface.sd = d * select(-1., 1., q.x * sincos.y - q.y * sincos.x > 0.0);
  surface.col = col;
  return surface;
}

fn opSmoothUnion(d1 : Surface, d2 : Surface, k : f32) -> Surface {
    var h = clamp(0.5 + 0.5 * (d2.sd - d1.sd) / k, 0.0, 1.0);
    var surface: Surface;
    surface.sd = mix(d2.sd, d1.sd, h) - k * h * (1.0 - h);
    surface.col = mix(d2.col, d1.col, h);
    return surface;
}

fn opSmoothSubtraction(d1 : Surface, d2 : Surface, k : f32) -> Surface {
    var h = clamp(0.5 - 0.5 * (d2.sd + d1.sd) / k, 0.0, 1.0);
    var surface: Surface;
    surface.sd = mix(d2.sd, -d1.sd, h) + k * h * (1.0 - h);
    surface.col = mix(d2.col, d1.col, h);
    return surface;
}

fn opUnion(d1 : Surface, d2 : Surface) -> Surface {
    var surface: Surface;
    if (d1.sd < d2.sd) {
        surface.sd = d1.sd;
        surface.col = d1.col;
    } else {
        surface.sd = d2.sd;
        surface.col = d2.col;
    }
    return surface;
}

fn opSubtraction(d1 : Surface, d2 : Surface) -> Surface {
    var surface: Surface;
    if (d1.sd < -d2.sd) {
        surface.sd = -d1.sd;
        surface.col = d1.col;
    } else {
        surface.sd = d2.sd;
        surface.col = d2.col;
    }
    return surface;
}

fn opInfArray(p : vec3 < f32>, c : vec3 < f32>) -> vec3 < f32> {
    return p - c * round(p / c);
}

fn opLimArray(p : vec3 < f32>, c : f32, lim : vec3 < f32>) -> vec3 < f32> {
    return p - c * clamp(round(p / c), -lim, lim);
}

fn opTwist(p: vec3<f32>, k: f32) -> vec3<f32> {
  let s = sin(k * p.y);
  let c = cos(k * p.y);
  let m = mat2x2<f32>(vec2<f32>(c, s), vec2<f32>(-s, c));
  return vec3<f32>(m * p.xz, p.y);
}

fn udTriangle(p : vec3 < f32>, a : vec3 < f32>, b : vec3 < f32>, c : vec3 < f32>) -> f32 {
    let ba = b - a; let pa = p - a;
    let cb = c - b; let pb = p - b;
    let ac = a - c; let pc = p - c;
    let nor = cross(ba, ac);
    let d1 = ba * clamp(dot(ba, pa) / dot(ba, ba), 0., 1.) - pa;
    let d2 = cb * clamp(dot(cb, pb) / dot(cb, cb), 0., 1.) - pb;
    let d3 = ac * clamp(dot(ac, pc) / dot(ac, ac), 0., 1.) - pc;
    let k0 = min(min(dot(d1, d1), dot(d2, d2)), dot(d3, d3));
    let k1 = dot(nor, pa) * dot(nor, pa) / dot(nor, nor);
    let t = sign(dot(cross(ba, nor), pa)) + sign(dot(cross(cb, nor), pb)) +
    sign(dot(cross(ac, nor), pc));
    return sqrt(select(k0, k1, t < 2.));
}

fn sdPlane(p : vec3 < f32>, n : vec3 < f32>, h : f32, col: vec3<f32>) -> Surface {
    //n must be normalized
    var surface: Surface;
    surface.sd = dot(p, n) + h;
    surface.col = col;
    return surface;
}

fn sdGyroid(p : vec3 < f32>, h : f32, col: vec3<f32>) -> Surface {
    var surface: Surface;
    surface.sd = abs(dot(sin(p), cos(p.yxz))) - h;
    surface.col = col;
    return surface;
}

fn sdBoxFrame(p: vec3<f32>, b: vec3<f32>, e: f32, col: vec3<f32>) -> Surface {
    let q = abs(p) - b;
    let w = abs(q + e) - e;

    var surface: Surface;
    surface.sd = min(min(
                       length(max(vec3<f32>(q.x, w.y, w.z), vec3<f32>(0.))) + min(max(q.x, max(w.y, w.z)), 0.),
                       length(max(vec3<f32>(w.x, q.y, w.z), vec3<f32>(0.))) + min(max(w.x, max(q.y, w.z)), 0.)),
                       length(max(vec3<f32>(w.x, w.y, q.z), vec3<f32>(0.))) + min(max(w.x, max(w.y, q.z)), 0.));
    surface.col = col;
    return surface;
}

fn palette(t : f32) -> vec3 < f32> {
    return .3+.2 * cos(4.28318 * (t + vec3(.5, .216, .757)));
}

fn opRevolution(p: vec3<f32>, o: f32) -> vec2<f32> {
  return vec2<f32>(length(p.xz) - o, p.y);
}

fn opExtrusion(d: Surface, z: f32, h: f32) -> Surface {
    var surface: Surface;
    let w = vec2<f32>(d.sd, abs(z) - h);
    surface.sd = min(max(w.x, w.y), 0.) + length(max(w, vec2<f32>(0.)));
    surface.col = d.col;
    return surface;
}

fn sd2dBox(p: vec2 < f32>, b : vec2 < f32>, col: vec3<f32>) -> Surface {
    var surface: Surface;
    let q = abs(p) - b;
    surface.sd = length(max(q, vec2 < f32 > (0.))) + min(max(q.x, q.y), 0.);
    surface.col = col;
    return surface;
}

fn repeat_rect(e: vec2<f32>, size: vec2<f32>, s: f32 ) -> vec2<f32>
{
    var p = abs(e/s) - (size*0.5 - 0.5);
    if (p.x > p.y) { p = p.yx; }
    p.y -= min(0.0, round(p.y));
    return p*s;
}