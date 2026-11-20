// Draws a backdrop blur record's backdrop through glass (`El::backdrop_lens`), inside its rounded
// rect, cut by its clip, at its opacity (`fill.a`). The glass is the element's own shape resting
// on the backdrop: flat on top at `gap + thickness`, falling over the `bevel` with a quarter-circle
// profile (vertical at the rim) to `gap`. A viewer straight above sends one ray per pixel; it
// refracts at the top (Snell) and runs straight to the backdrop, so the flat middle shows the
// backdrop as it is and the shoulder pulls it in from beyond the rim. Two indices (Cauchy,
// n(400 nm) − n(700 nm) = dispersion) are traced and eight wavelengths interpolate between them,
// weighted into RGB, so light and dark edges fringe as a spectrum. The top takes exact dielectric
// Fresnel; what it reflects is a soft room light above, or the backdrop beside the glass off the
// steep shoulder. The optics follow rune-light's glass.wgsl. The backdrop is the sharp copy; when
// the record also blurs (frosted glass), the flat top shows the blur and the shoulder fades from
// sharp at the rim to the blur at the top. Prepended with `clip.wgsl`; group 0 is the uber
// shader's.

struct Globals {
    viewport: vec2<f32>,
    pad: vec2<f32>,
    flags: vec4<f32>,
};

@group(0) @binding(0) var<uniform> g: Globals;
@group(0) @binding(3) var<uniform> projection: vec4<f32>;
@group(1) @binding(0) var blurred: texture_2d<f32>;
@group(1) @binding(1) var smp: sampler;

struct Level {
    w: vec4<f32>,
    region: vec4<f32>,
    map: vec4<f32>,
    // The pyramid's window origin xy and extent zw, in device pixels.
    place: vec4<f32>,
};
@group(1) @binding(2) var<uniform> u: Level;
// The window's pixels below the record, in window device pixels.
@group(1) @binding(3) var sharp: texture_2d<f32>;

struct In {
    @builtin(vertex_index) vi: u32,
    @location(0) rect: vec4<f32>,
    @location(1) radii: vec4<f32>,
    @location(2) fill: vec4<f32>,
    // x: dispersion, y: light.
    @location(3) border: vec4<f32>,
    // x: blur sigma in device px (0: sharp).
    @location(4) params: vec4<f32>,
    // thickness, bevel, gap (device px), index of refraction.
    @location(5) uv: vec4<f32>,
    @location(6) clip: vec4<f32>,
    @location(7) fade: vec4<u32>,
};

struct Out {
    @builtin(position) pos: vec4<f32>,
    @location(0) p: vec2<f32>,
    @location(1) @interpolate(flat) rect: vec4<f32>,
    @location(2) @interpolate(flat) radii: vec4<f32>,
    @location(3) @interpolate(flat) alpha: f32,
    @location(4) @interpolate(flat) clip: vec4<f32>,
    @location(5) @interpolate(flat) fade: vec4<u32>,
    @location(6) @interpolate(flat) slab: vec4<f32>,
    // dispersion, light, frosted (0 or 1).
    @location(7) @interpolate(flat) optics: vec4<f32>,
};

@vertex
fn vs(i: In) -> Out {
    let corner = vec2<f32>(f32(i.vi & 1u), f32(i.vi >> 1u));
    let p = i.rect.xy - vec2<f32>(1.0) + corner * (i.rect.zw + vec2<f32>(2.0));
    let size_px = select(g.viewport, projection.zw, projection.z > 0.0);
    let local = p - projection.xy;
    var o: Out;
    o.pos = vec4<f32>(local.x / size_px.x * 2.0 - 1.0, 1.0 - local.y / size_px.y * 2.0, 0.0, 1.0);
    o.p = p;
    o.rect = i.rect;
    o.radii = i.radii;
    o.alpha = i.fill.a;
    o.clip = i.clip;
    o.fade = i.fade;
    o.slab = i.uv;
    o.optics = vec4<f32>(i.border.x, i.border.y, select(0.0, 1.0, i.params.x > 0.0), 0.0);
    return o;
}

struct Slab {
    rect: vec4<f32>,
    radii: vec4<f32>,
    // thickness, bevel, gap, ior
    s: vec4<f32>,
};

fn sdf(k: Slab, p: vec2<f32>) -> f32 {
    let half = k.rect.zw * 0.5;
    return sd_rounded(p - (k.rect.xy + half), half, k.radii);
}

fn height_d(k: Slab, d: f32) -> f32 {
    let x = clamp(-d / max(k.s.y, 0.5), 0.0, 1.0);
    return k.s.z + k.s.x * sqrt(max(1.0 - (1.0 - x) * (1.0 - x), 0.0));
}

fn height(k: Slab, p: vec2<f32>) -> f32 {
    return height_d(k, sdf(k, p));
}

fn normal_at(k: Slab, p: vec2<f32>) -> vec3<f32> {
    let e = 0.5;
    let hx = height(k, p + vec2<f32>(e, 0.0)) - height(k, p - vec2<f32>(e, 0.0));
    let hy = height(k, p + vec2<f32>(0.0, e)) - height(k, p - vec2<f32>(0.0, e));
    return normalize(vec3<f32>(-hx / (2.0 * e), -hy / (2.0 * e), 1.0));
}

fn fresnel(cos_i: f32, n1: f32, n2: f32) -> f32 {
    let ci = clamp(cos_i, 0.0, 1.0);
    let s2 = (n1 / n2) * (n1 / n2) * (1.0 - ci * ci);
    if (s2 >= 1.0) {
        return 1.0;
    }
    let ct = sqrt(1.0 - s2);
    let rs = (n1 * ci - n2 * ct) / (n1 * ci + n2 * ct);
    let rp = (n1 * ct - n2 * ci) / (n1 * ct + n2 * ci);
    return 0.5 * (rs * rs + rp * rp);
}

// A dim room with a soft light up and to the left and a thin one to the right, scaled by `light`.
fn env(d: vec3<f32>, light: f32) -> vec3<f32> {
    let up = clamp(d.z, 0.0, 1.0);
    var c = mix(vec3<f32>(0.006, 0.007, 0.011), vec3<f32>(0.030, 0.034, 0.046), up);
    let box = normalize(vec3<f32>(-0.42, -0.58, 0.70));
    c = c + vec3<f32>(0.85, 0.88, 0.95) * smoothstep(0.80, 0.97, dot(d, box)) * 4.0;
    let strip = normalize(vec3<f32>(0.80, 0.10, 0.59));
    c = c + vec3<f32>(0.55, 0.62, 0.80) * smoothstep(0.93, 0.99, dot(d, strip)) * 2.0;
    return c * light;
}

// The backdrop at q, kept inside what was copied: the rect plus twice the glass's height
// (`prepare_backdrop_blur`'s reach).
// `frost` (0…1) mixes in the record's blur.
fn backdrop(k: Slab, q0: vec2<f32>, frost: f32) -> vec4<f32> {
    let r = (k.s.x + k.s.z) * 2.0;
    let q = clamp(q0, k.rect.xy - vec2<f32>(r), k.rect.xy + k.rect.zw + vec2<f32>(r));
    let s = textureSampleLevel(sharp, smp, q / vec2<f32>(textureDimensions(sharp)), 0.0);
    if (frost <= 0.0) {
        return s;
    }
    return mix(s, textureSampleLevel(blurred, smp, (q - u.place.xy) / u.place.zw, 0.0), frost);
}

// Where a ray that enters the top at p with normal n and index ior lands on the backdrop.
fn land(p: vec2<f32>, n: vec3<f32>, zt: f32, ior: f32) -> vec2<f32> {
    let r = refract(vec3<f32>(0.0, 0.0, -1.0), n, 1.0 / ior);
    return p + r.xy * (zt / max(-r.z, 1e-3));
}

// Approximate CIE response of eight wavelengths 400…700 nm in linear sRGB.
fn spectrum(i: i32) -> vec3<f32> {
    var w = array<vec3<f32>, 8>(
        vec3<f32>(0.20, 0.00, 0.62),
        vec3<f32>(0.12, 0.02, 1.00),
        vec3<f32>(0.00, 0.22, 0.86),
        vec3<f32>(0.00, 0.62, 0.30),
        vec3<f32>(0.18, 0.92, 0.00),
        vec3<f32>(0.70, 0.62, 0.00),
        vec3<f32>(1.00, 0.20, 0.00),
        vec3<f32>(0.62, 0.00, 0.00),
    );
    return w[i];
}

fn shade(k: Slab, p: vec2<f32>, d: f32, optics: vec4<f32>) -> vec4<f32> {
    let ior = k.s.w;
    // Only the flat top is frosted; the shoulder bends the sharp backdrop and frosts toward the top,
    // so what it pulls in from beyond the rim stays legible.
    let x = clamp(-d / max(k.s.y, 0.5), 0.0, 1.0);
    let frost = optics.z * x * x;
    // The flat top: straight through, less what the top reflects.
    if (-d >= k.s.y + 1.0) {
        let f0 = fresnel(1.0, 1.0, ior);
        let b = backdrop(k, p, optics.z);
        return vec4<f32>(b.rgb * (1.0 - f0) + env(vec3<f32>(0.0, 0.0, 1.0), optics.y) * f0, b.a);
    }
    let zt = height_d(k, d);
    let n = normal_at(k, p);
    let bc = optics.x / (1.0 / (0.4 * 0.4) - 1.0 / (0.7 * 0.7));
    let ac = ior - bc / (0.55 * 0.55);
    let red = land(p, n, zt, max(ac + bc / (0.7 * 0.7), 1.0));
    let violet = land(p, n, zt, max(ac + bc / (0.4 * 0.4), 1.0));
    var trans = vec3<f32>(0.0);
    var wsum = vec3<f32>(0.0);
    var alpha = 0.0;
    for (var i = 0; i < 8; i = i + 1) {
        let w = spectrum(i);
        let b = backdrop(k, mix(violet, red, f32(i) / 7.0), frost);
        trans = trans + w * b.rgb;
        wsum = wsum + w;
        alpha = alpha + b.a / 8.0;
    }
    trans = trans / wsum;
    // Reflection: upward rays see the room, downward ones (off the steep shoulder) the backdrop
    // beside the glass.
    let f = fresnel(n.z, 1.0, ior);
    let rr = reflect(vec3<f32>(0.0, 0.0, -1.0), n);
    let reach = min(zt * length(rr.xy) / max(-rr.z, 1e-3), zt * 4.0);
    let beside = backdrop(k, p + normalize(rr.xy + vec2<f32>(1e-5)) * reach, frost).rgb;
    let refl = mix(env(rr, optics.y), beside, smoothstep(0.05, -0.15, rr.z));
    return vec4<f32>(trans * (1.0 - f) + refl * f, alpha);
}

@fragment
fn fs(i: Out) -> @location(0) vec4<f32> {
    let k = Slab(i.rect, i.radii, i.slab);
    let d = sdf(k, i.p);
    let cov = clamp(0.5 - d, 0.0, 1.0) * clip_alpha(i.p, i.clip, i.fade) * i.alpha;
    if (cov <= 0.0) {
        discard;
    }
    // ponytail: one ray per pixel. rune-light supersamples the shoulder 4x against aliasing; add
    // that if a thin bevel shimmers while it moves.
    let c = shade(k, i.p, min(d, 0.0), i.optics);
    return vec4<f32>(c.rgb * c.a, c.a) * cov;
}
