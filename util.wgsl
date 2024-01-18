// Surface Definition:
//
// struct Surface {
//     sd: f32,
//     col: vec3<f32>
// };

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
