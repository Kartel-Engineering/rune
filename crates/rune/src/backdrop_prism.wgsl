// Generated: the prism's numbers, fitted to one reference rendering of the look. See prism() below.
const PRISM_T = array<f32, 20>(0.00000, 0.50000, 1.00000, 1.50000, 2.00000, 2.50000, 3.00000, 3.50000, 4.00000, 5.00000, 6.00000, 7.00000, 8.00000, 10.00000, 12.00000, 15.00000, 19.00000, 25.00000, 32.00000, 41.00000);
const PRISM_D = array<vec4<f32>, 20>(
    vec4<f32>(24.25000, -12.25000, 0.50000, 2.50000),
    vec4<f32>(0.25000, 40.51000, 32.66000, 38.80000),
    vec4<f32>(42.75000, 46.03000, 38.23000, 35.90000),
    vec4<f32>(43.50000, 45.34000, 35.69000, 20.95000),
    vec4<f32>(40.75000, 35.15000, 36.65000, 33.00000),
    vec4<f32>(37.50000, 36.32500, 34.57500, 31.50000),
    vec4<f32>(39.25000, 34.50000, 34.50000, 26.50000),
    vec4<f32>(37.50000, 33.67500, 32.80000, 28.00000),
    vec4<f32>(34.50000, 31.10000, 31.85000, 24.75000),
    vec4<f32>(32.66667, 29.36000, 29.59333, 23.88333),
    vec4<f32>(30.08333, 28.37000, 27.08667, 22.01667),
    vec4<f32>(27.75000, 26.00500, 23.83000, 20.90000),
    vec4<f32>(25.91667, 25.14000, 22.07333, 16.53333),
    vec4<f32>(23.00000, 21.41000, 20.56000, 18.80000),
    vec4<f32>(19.25000, 17.32000, 13.62000, 5.85000),
    vec4<f32>(11.72222, 11.25333, 10.08111, 9.37778),
    vec4<f32>(5.50000, 5.28000, 4.73000, 4.40000),
    vec4<f32>(3.20000, 3.07200, 2.75200, 2.56000),
    vec4<f32>(2.08000, 1.99680, 1.78880, 1.66400),
    vec4<f32>(0.64000, 0.61440, 0.55040, 0.51200),
);
const PRISM_W = array<vec3<f32>, 4>(vec3<f32>(0.57569, 0.03247, 0.01153), vec3<f32>(0.19037, 0.08442, 0.01729), vec3<f32>(0.10551, 0.76623, 0.01441), vec3<f32>(0.12844, 0.11688, 0.95677));
const PRISM_OFF = array<vec2<f32>, 4>(vec2<f32>(0.00000, 0.00000), vec2<f32>(0.00000, 0.00000), vec2<f32>(0.00000, 0.00000), vec2<f32>(0.00000, 0.00000));
const PRISM_RING_D = array<f32, 9>(-3.00000, -2.00000, -1.50000, -1.00000, -0.50000, 0.00000, 0.50000, 1.00000, 1.50000);
const PRISM_RING_A = array<f32, 9>(0.00000, 0.05000, 0.18750, 0.05000, 0.11250, -0.27500, -0.03750, 0.01250, -0.00000);
const PRISM_RING = vec3<f32>(0.31500, 0.03500, 0.23500);
const PRISM_RING_N = vec2<f32>(-0.95000, 1.02500);
const PRISM_BEND_N = vec2<f32>(0.01500, 0.00000);
const PRISM_SHADE = vec4<f32>(0.83250, 0.87750, 0.98750, 0.96125);
const PRISM_RIM = vec2<f32>(0.10000, 3.50000);
const PRISM_SIDE = vec4<f32>(0.11250, 0.15750, 0.27000, 0.00000);
const PRISM_SIDE_W: f32 = 1.75000;
const PRISM_LIGHT1 = array<f32, 5>(-3.15000, 0.47500, 9.50000, 0.47500, 1.37500);
const PRISM_LIGHT2 = array<f32, 5>(-0.80000, 0.70000, 4.00000, 1.82500, 1.42500);

// `Lens::prism`: liquid glass. Four colour samples (`PRISM_W`, linear RGB, each channel's weights sum
// to one) each read the backdrop moved in along the rim's inward normal by its own distance
// (`PRISM_D`, piecewise linear in the distance in from the rim, `PRISM_T`, points), so red reaches
// farthest and blue least: a light/dark edge under the rim splits into bands of yellow, red and cyan.
// What lies outside the element is not read (transparent). The bent backdrop is shaded (a top to bottom
// gain, a warm tint, more near the rim), lit by two rim lights, and a thin ring lies over the edge. It
// is defined in sRGB encoding, as a browser composes; a frame that blends in linear light gets the
// same colours decoded.

fn prism_bend(t: f32) -> vec4<f32> {
    var i = 0;
    for (; i < 18; i = i + 1) {
        if (t < PRISM_T[i + 1]) { break; }
    }
    let x = clamp((t - PRISM_T[i]) / (PRISM_T[i + 1] - PRISM_T[i]), 0.0, 1.0);
    return mix(PRISM_D[i], PRISM_D[i + 1], x);
}

fn prism_ring(d: f32) -> f32 {
    if (d <= PRISM_RING_D[0]) { return PRISM_RING_A[0]; }
    for (var i = 0; i < 8; i = i + 1) {
        if (d < PRISM_RING_D[i + 1]) {
            return mix(PRISM_RING_A[i], PRISM_RING_A[i + 1], (d - PRISM_RING_D[i]) / (PRISM_RING_D[i + 1] - PRISM_RING_D[i]));
        }
    }
    return PRISM_RING_A[8];
}

fn prism_normal(k: Slab, p: vec2<f32>) -> vec2<f32> {
    let e = 0.5;
    let g = vec2<f32>(sdf(k, p + vec2<f32>(e, 0.0)) - sdf(k, p - vec2<f32>(e, 0.0)), sdf(k, p + vec2<f32>(0.0, e)) - sdf(k, p - vec2<f32>(0.0, e)));
    return g / max(length(g), 1e-6);
}

// The backdrop at q in linear light, straight (not premultiplied), 0 outside the element's rect.
fn prism_read(k: Slab, q: vec2<f32>) -> vec3<f32> {
    let lo = clamp(q - k.rect.xy + vec2<f32>(0.5), vec2<f32>(0.0), vec2<f32>(1.0));
    let hi = clamp(k.rect.xy + k.rect.zw - q + vec2<f32>(0.5), vec2<f32>(0.0), vec2<f32>(1.0));
    let c = linear(textureSampleLevel(sharp, smp, q / vec2<f32>(textureDimensions(sharp)), 0.0));
    return c.rgb * (lo.x * lo.y * hi.x * hi.y);
}

fn prism_light(l: array<f32, 5>, n: vec2<f32>, t: f32) -> f32 {
    let x = (t - l[4]) / l[3];
    return l[1] * pow(max(n.x * cos(l[0]) + n.y * sin(l[0]), 1e-6), l[2]) * exp(-x * x);
}

// The prism's colour at device pixel p (straight, sRGB-encoded) and its ring's alpha; `kp` is device
// pixels per point.
fn prism(k: Slab, p: vec2<f32>, kp: f32) -> vec4<f32> {
    let o = array<vec2<f32>, 4>(vec2<f32>(-0.125, -0.375), vec2<f32>(0.375, -0.125), vec2<f32>(0.125, 0.375), vec2<f32>(-0.375, 0.125));
    var acc = vec3<f32>(0.0);
    for (var s = 0; s < 4; s = s + 1) {
        let q = p + o[s];
        let n = prism_normal(k, q);
        let t = max(-sdf(k, q), 0.0) / kp;
        let bend = prism_bend(t) * (1.0 + dot(PRISM_BEND_N, n)) * kp;
        for (var c = 0; c < 4; c = c + 1) {
            acc = acc + prism_read(k, q - n * bend[c]) * PRISM_W[c];
        }
    }
    acc = acc * 0.25;
    let d = sdf(k, p);
    let t = max(-d, 0.0) / kp;
    let n = prism_normal(k, p);
    let v = clamp((p.y - k.rect.y) / k.rect.w, 0.0, 1.0);
    let shade = mix(PRISM_SHADE.x, PRISM_SHADE.y, v) * vec3<f32>(1.0, PRISM_SHADE.z, PRISM_SHADE.w) * (1.0 + PRISM_RIM.x * exp(-t / PRISM_RIM.y));
    let side = vec3<f32>(1.0) + exp(-t / PRISM_SIDE_W) * (PRISM_SIDE.xyz * n.x + vec3<f32>(PRISM_SIDE.w * n.y));
    let lit = acc * shade * side + vec3<f32>(prism_light(PRISM_LIGHT1, n, t) + prism_light(PRISM_LIGHT2, n, t));
    let v2 = clamp(lit, vec3<f32>(0.0), vec3<f32>(1.0));
    let enc = select(1.055 * pow(v2, vec3<f32>(1.0 / 2.4)) - 0.055, v2 * 12.92, v2 <= vec3<f32>(0.0031308));
    let ring = clamp(prism_ring(d / kp) * (1.0 + dot(PRISM_RING_N, n)), 0.0, 1.0);
    return vec4<f32>(enc, ring);
}

// The ring over the glass at coverage `cov`, over what is behind: premultiplied, in the frame's
// blend space (the prism is defined in sRGB encoding).
fn prism_out(c: vec4<f32>, cov: f32) -> vec4<f32> {
    var ring = clamp(PRISM_RING, vec3<f32>(0.0), vec3<f32>(1.0));
    var glass = c.rgb;
    if (g.flags.x < 0.5) {
        ring = select(pow((ring + 0.055) / 1.055, vec3<f32>(2.4)), ring / 12.92, ring <= vec3<f32>(0.04045));
        glass = select(pow((glass + 0.055) / 1.055, vec3<f32>(2.4)), glass / 12.92, glass <= vec3<f32>(0.04045));
    }
    return vec4<f32>(ring * c.a + glass * cov * (1.0 - c.a), c.a + cov * (1.0 - c.a));
}
