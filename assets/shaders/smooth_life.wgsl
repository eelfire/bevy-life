@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, read_write>;

fn hash(value: u32) -> u32 {
    var state = value;
    state = state ^ 2747636419u;
    state = state * 2654435769u;
    state = state ^ (state >> 16u);
    state = state * 2654435769u;
    state = state ^ (state >> 16u);
    state = state * 2654465769u;
    return state;
}

fn random_float(value: u32) -> f32 {
    let x: f32 = f32(hash(value)) / 4294967295.0;
    return clamp(x, 0.0, 1.0);
}

// const WIDTH: usize = 100;
// const HEIGHT: usize = 100;

// const LEVEL: [char; 10] = [' ', '.', '-', '=', 'c', 'o', 'a', 'A', '@', '#'];
// const LEVEL_COUNT: usize = LEVEL.len() - 1;

const alpha_n: f32 = 0.028;
const alpha_m: f32 = 0.147;

const b1: f32 = 0.278;
const b2: f32 = 0.365;
const d1: f32 = 0.267;
const d2: f32 = 0.445;

const dt: f32 = 0.59;

const ra: f32 = 12.0;
const ri: f32 = 4.0;

fn sigma(x: f32, a: f32, alpha: f32) -> f32 {
    return 1.0 / (1.0 + exp(-(x - a) * 4.0 / alpha));
}

fn sigma_n(x: f32, a: f32, b: f32) -> f32 {
    return sigma(x, a, alpha_n) * (1.0 - sigma(x, b, alpha_n));
}

fn sigma_m(x: f32, y: f32, m: f32) -> f32 {
    return x * (1.0 - sigma(m, 0.5, alpha_m)) + y * sigma(m, 0.5, alpha_m);
}

fn s(n: f32, m: f32) -> f32 {
    return sigma_n(n, sigma_m(b1, d1, m), sigma_m(b2, d2, m));
}

fn grid(location: vec2<f32>) -> f32 {
    // var tx: i32 = i32(f32(location.x) / 6.0);
    // var ty: i32 = i32(f32(location.y) / 6.0);
    // var t: vec4<f32> = textureLoad(texture, vec2<i32>(tx, ty));
    var tx: f32 = location.x;
    var ty: f32 = location.y;
    var t: vec4<f32> = textureLoad(texture, vec2<i32>(i32(tx), i32(ty)));
    return max(max(t.x, t.y), t.z);
}

@compute @workgroup_size(8, 8, 1)
fn init(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
    @builtin(num_workgroups) num_workgroups: vec3<u32>,
) {
    let location = vec2<i32>(invocation_id.xy);

    let random_num = random_float(invocation_id.y * num_workgroups.x + invocation_id.x);
    // let alive = random_num > 0.9;
    let color = vec4<f32>(random_num, random_num, random_num, 1.0);

    let perlin = perlinNoise2(vec2<f32>(f32(location.x), f32(location.y)));
    let perlin_num = clamp(perlin, 0.0, 1.0);
    let perlin_color = vec4<f32>(perlin_num, perlin_num, perlin_num, 1.0);

    textureStore(texture, location, color);
    // textureStore(texture, location, perlin_color);
}

@compute @workgroup_size(8, 8, 1)
fn update(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let location = vec2<i32>(invocation_id.xy);

    var cx = f32(location.x);
    var cy = f32(location.y);
    // var cy = (1.0 - f32(location.y));
    var m: f32 = 0.0;
    var M: f32 = 0.0;
    var n: f32 = 0.0;
    var N: f32 = 0.0;

    for (var dy: f32 = -(ra - 1.0); dy <= (ra - 1.0); dy = dy + 1.0) {
        for (var dx: f32 = -(ra - 1.0); dx <= (ra - 1.0); dx = dx + 1.0) {
            var x: f32 = cx + dx;
            var y: f32 = cy + dy;
            if (dx * dx) + (dy * dy) <= (ri * ri) {
                m += grid(vec2<f32>(x, y));
                M += 1.0;
            } else if (dx * dx) + (dy * dy) <= (ra * ra) {
                n += grid(vec2<f32>(x, y));
                N += 1.0;
            }
        }
    }
    m /= M;
    n /= N;
    var q: f32 = s(n, m);
    var diff: f32 = 2.0 * q - 1.0;
    var v = clamp(grid(vec2<f32>(cx, cy)) + dt * diff, 0.0, 1.0);

    let color = vec4<f32>(v, v, v, 1.0);

    // let new_location = 6 * location;

    textureStore(texture, location, color);
    // textureStore(texture, new_location, color);
}

// MIT License. © Stefan Gustavson, Munrocket
//
fn permute4(x: vec4<f32>) -> vec4<f32> { return ((x * 34. + 1.) * x) % vec4<f32>(289.); }
fn fade2(t: vec2<f32>) -> vec2<f32> { return t * t * t * (t * (t * 6. - 15.) + 10.); }

fn perlinNoise2(P: vec2<f32>) -> f32 {
    var Pi: vec4<f32> = floor(P.xyxy) + vec4<f32>(0., 0., 1., 1.);
    let Pf = fract(P.xyxy) - vec4<f32>(0., 0., 1., 1.);
    Pi = Pi % vec4<f32>(289.); // To avoid truncation effects in permutation
    let ix = Pi.xzxz;
    let iy = Pi.yyww;
    let fx = Pf.xzxz;
    let fy = Pf.yyww;
    let i = permute4(permute4(ix) + iy);
    var gx: vec4<f32> = 2. * fract(i * 0.0243902439) - 1.; // 1/41 = 0.024...
    let gy = abs(gx) - 0.5;
    let tx = floor(gx + 0.5);
    gx = gx - tx;
    var g00: vec2<f32> = vec2<f32>(gx.x, gy.x);
    var g10: vec2<f32> = vec2<f32>(gx.y, gy.y);
    var g01: vec2<f32> = vec2<f32>(gx.z, gy.z);
    var g11: vec2<f32> = vec2<f32>(gx.w, gy.w);
    let norm = 1.79284291400159 - 0.85373472095314 * vec4<f32>(dot(g00, g00), dot(g01, g01), dot(g10, g10), dot(g11, g11));
    g00 = g00 * norm.x;
    g01 = g01 * norm.y;
    g10 = g10 * norm.z;
    g11 = g11 * norm.w;
    let n00 = dot(g00, vec2<f32>(fx.x, fy.x));
    let n10 = dot(g10, vec2<f32>(fx.y, fy.y));
    let n01 = dot(g01, vec2<f32>(fx.z, fy.z));
    let n11 = dot(g11, vec2<f32>(fx.w, fy.w));
    let fade_xy = fade2(Pf.xy);
    let n_x = mix(vec2<f32>(n00, n01), vec2<f32>(n10, n11), vec2<f32>(fade_xy.x));
    let n_xy = mix(n_x.x, n_x.y, fade_xy.y);
    return 2.3 * n_xy;
}
