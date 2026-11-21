// Draws a backdrop blur record's backdrop through glass (`El::backdrop_lens`), inside its rounded
// rect, cut by its clip, at its opacity (`fill.a`). The glass is the element's own shape floating
// `gap` above the backdrop: a flat bottom, and a top that is flat at `gap + thickness` and falls
// over the `bevel` with a quarter-circle profile (vertical at the rim) to a thin edge. A viewer
// straight above sends one ray per pixel; it refracts at the top (Snell), crosses the glass to
// the bottom, refracts again into the air gap and lands on the backdrop, so the flat middle shows
// the backdrop as it is and the shoulder pulls it in from beyond the rim. The glass absorbs a
// little along the path (Beer-Lambert), which dims light and leaves black black. Two indices (Cauchy,
// n(400 nm) − n(700 nm) = dispersion) are traced and eight wavelengths interpolate between them,
// weighted into RGB, so light and dark edges fringe as a spectrum. The top takes exact dielectric
// Fresnel; what it reflects is a soft room light above, or the backdrop beside the glass off the
// steep shoulder. The optics follow rune-light's glass.wgsl. Everything is premultiplied, as
// the copied backdrop is. The backdrop is the sharp copy; when
// the record also blurs (frosted glass), the flat top shows the blur and the shoulder fades from
// sharp at the rim to the blur at the top. The shoulder squeezes many backdrop pixels into one,
// so each ray reads the backdrop prefiltered at its footprint, from the blur's down chain, as
// rune-light's glass reads its pyramid: text that moves under the rim stays smooth instead of
// aliasing. Prepended with `clip.wgsl`; group 0 is the uber shader's.

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
// The blur's down chain over the record's region, every level: level m is a prefilter about
// 2^(m+1) device pixels wide. Placed as `blurred` is (`u.place`).
@group(1) @binding(4) var prefiltered: texture_2d<f32>;
// The coarsest level of `prefiltered` a ray reads (`LENS_LEVELS` in backdrop_blur.rs, less one).
const TOP: f32 = 4.0;

struct In {
    @builtin(vertex_index) vi: u32,
    @location(0) rect: vec4<f32>,
    @location(1) radii: vec4<f32>,
    @location(2) fill: vec4<f32>,
    // x: dispersion, y: light, z: 1 for a prism (`backdrop_prism.wgsl`).
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
    // dispersion, light, frosted (0 or 1), device px per point.
    @location(7) @interpolate(flat) optics: vec4<f32>,
    @location(8) @interpolate(flat) prism: f32,
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
    o.optics = vec4<f32>(i.border.x, i.border.y, select(0.0, 1.0, i.params.x > 0.0), max(i.params.w, 1e-3));
    o.prism = i.border.z;
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

// Absorption per point of path in the glass (Beer-Lambert), the same for every channel.
const ABSORB: f32 = 0.003;

// The top's height over the backdrop: the shoulder starts at 4 % of the thickness, so the rim has
// an edge to refract through.
fn height_d(k: Slab, d: f32) -> f32 {
    let x = clamp(-d / max(k.s.y, 0.5), 0.0, 1.0);
    return k.s.z + k.s.x * (0.04 + 0.96 * sqrt(max(1.0 - (1.0 - x) * (1.0 - x), 0.0)));
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
// (`prepare_backdrop_blur`'s reach). `radius` (device px) prefilters it, as rune-light's `ui_at`:
// sharp up to 1, the down chain's level log2(radius) − 1 from 3. `frost` (0…1) mixes in the
// record's blur.
fn backdrop(k: Slab, q0: vec2<f32>, frost: f32, radius: f32) -> vec4<f32> {
    let r = (k.s.x + k.s.z) * 2.0;
    let q = clamp(q0, k.rect.xy - vec2<f32>(r), k.rect.xy + k.rect.zw + vec2<f32>(r));
    var c = textureSampleLevel(sharp, smp, q / vec2<f32>(textureDimensions(sharp)), 0.0);
    if (radius > 1.0) {
        let lod = clamp(log2(radius) - 1.0, 0.0, TOP);
        c = mix(c, textureSampleLevel(prefiltered, smp, (q - u.place.xy) / u.place.zw, lod), smoothstep(1.0, 3.0, radius));
    }
    if (frost > 0.0) {
        c = mix(c, textureSampleLevel(blurred, smp, (q - u.place.xy) / u.place.zw, 0.0), frost);
    }
    return linear(c);
}

// The optics work in linear light. A frame that blends in sRGB encoding (`g.flags.x`) holds
// encoded values in its copy: `linear` decodes what the glass reads and `encoded` encodes what it
// writes, both premultiplied.
fn linear(c: vec4<f32>) -> vec4<f32> {
    if (g.flags.x < 0.5 || c.a <= 0.0) {
        return c;
    }
    let v = clamp(c.rgb / c.a, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(select(pow((v + 0.055) / 1.055, vec3<f32>(2.4)), v / 12.92, v <= vec3<f32>(0.04045)) * c.a, c.a);
}

fn encoded(c: vec4<f32>) -> vec4<f32> {
    if (g.flags.x < 0.5 || c.a <= 0.0) {
        return c;
    }
    let v = clamp(c.rgb / c.a, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(select(1.055 * pow(v, vec3<f32>(1.0 / 2.4)) - 0.055, v * 12.92, v <= vec3<f32>(0.0031308)) * c.a, c.a);
}

// Where a ray that enters the top at p (height zt, normal n, index ior) lands on the backdrop
// (xy), and its path in the glass (z). Past the critical angle at the bottom it lands below the
// exit point, as rune-light's glass does without its total internal reflection.
fn land(k: Slab, p: vec2<f32>, n: vec3<f32>, zt: f32, ior: f32) -> vec3<f32> {
    let r1 = refract(vec3<f32>(0.0, 0.0, -1.0), n, 1.0 / ior);
    let s = (zt - k.s.z) / max(-r1.z, 1e-3);
    let r2 = refract(r1, vec3<f32>(0.0, 0.0, 1.0), ior);
    return vec3<f32>(p + r1.xy * s + r2.xy * (k.s.z / max(-r2.z, 0.05)), s);
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

// Where a ray off the shoulder at p reflects to: the backdrop beside the glass, `reach` out.
fn beside_at(p: vec2<f32>, n: vec3<f32>, zt: f32) -> vec2<f32> {
    let rr = reflect(vec3<f32>(0.0, 0.0, -1.0), n);
    let reach = min(zt * length(rr.xy) / max(-rr.z, 1e-3), zt * 4.0);
    return p + normalize(rr.xy + vec2<f32>(1e-5)) * reach;
}

// Where the ray at p lands (xy, at index `ior`) and where its reflection reads (zw).
fn reads(k: Slab, p: vec2<f32>, ior: f32) -> vec4<f32> {
    let n = normal_at(k, p);
    let zt = height_d(k, min(sdf(k, p), 0.0));
    return vec4<f32>(land(k, p, n, zt, ior).xy, beside_at(p, n, zt));
}

fn shade(k: Slab, p: vec2<f32>, d: f32, optics: vec4<f32>) -> vec4<f32> {
    let ior = k.s.w;
    // Only the flat top is frosted; the shoulder bends the sharp backdrop and frosts toward the top,
    // so what it pulls in from beyond the rim stays legible.
    let x = clamp(-d / max(k.s.y, 0.5), 0.0, 1.0);
    let frost = optics.z * x * x;
    let absorb = ABSORB / optics.w;
    // The flat top: straight through, less what the top reflects.
    if (-d >= k.s.y + 1.0) {
        let f0 = fresnel(1.0, 1.0, ior);
        let b = backdrop(k, p, optics.z, 0.0);
        let t = (1.0 - f0) * exp(-absorb * k.s.x);
        return vec4<f32>(b.rgb * t + env(vec3<f32>(0.0, 0.0, 1.0), optics.y) * f0 * b.a, b.a);
    }
    let zt = height_d(k, d);
    let n = normal_at(k, p);
    let bc = optics.x / (1.0 / (0.4 * 0.4) - 1.0 / (0.7 * 0.7));
    let ac = ior - bc / (0.55 * 0.55);
    let red = land(k, p, n, zt, max(ac + bc / (0.7 * 0.7), 1.0));
    let violet = land(k, p, n, zt, max(ac + bc / (0.4 * 0.4), 1.0));
    // The footprint: how far what this ray reads moves per device pixel, the larger of x and y
    // (finite differences half a pixel apart, the rays' spacing), turned into a prefilter radius
    // as rune-light's glass turns its derivatives.
    let e = 0.5;
    let at = reads(k, p, ior);
    let dx = (reads(k, p + vec2<f32>(e, 0.0), ior) - at) / e;
    let dy = (reads(k, p + vec2<f32>(0.0, e), ior) - at) / e;
    let through = clamp(max(length(dx.xy), length(dy.xy)) * 0.5 - 0.5, 0.0, 24.0);
    let off = clamp(max(length(dx.zw), length(dy.zw)) * 0.5 - 0.5, 0.0, 24.0);
    var trans = vec3<f32>(0.0);
    var wsum = vec3<f32>(0.0);
    var alpha = 0.0;
    for (var i = 0; i < 8; i = i + 1) {
        let w = spectrum(i);
        let q = mix(violet, red, f32(i) / 7.0);
        let b = backdrop(k, q.xy, frost, through);
        trans = trans + w * b.rgb * exp(-absorb * q.z);
        wsum = wsum + w;
        alpha = alpha + b.a / 8.0;
    }
    trans = trans / wsum;
    // Reflection: upward rays see the room, downward ones (off the steep shoulder) the backdrop
    // beside the glass.
    let f = fresnel(n.z, 1.0, ior);
    let rr = reflect(vec3<f32>(0.0, 0.0, -1.0), n);
    let beside = backdrop(k, at.zw, frost, off).rgb;
    let refl = mix(env(rr, optics.y) * alpha, beside, smoothstep(0.05, -0.15, rr.z));
    return vec4<f32>(trans * (1.0 - f) + refl * f, alpha);
}

@fragment
fn fs(i: Out) -> @location(0) vec4<f32> {
    let k = Slab(i.rect, i.radii, i.slab);
    let d = sdf(k, i.p);
    if (i.prism > 0.5) {
        let c = prism(k, i.p, i.optics.w);
        let o = prism_out(c, clamp(0.5 - d, 0.0, 1.0)) * clip_alpha(i.p, i.clip, i.fade) * i.alpha;
        if (o.a <= 0.0) {
            discard;
        }
        return o;
    }
    let cov = clamp(0.5 - d, 0.0, 1.0) * clip_alpha(i.p, i.clip, i.fade) * i.alpha;
    if (cov <= 0.0) {
        discard;
    }
    if (-d >= k.s.y + 1.0) {
        return encoded(shade(k, i.p, d, i.optics)) * cov;
    }
    // The shoulder bends fastest: four rays on a rotated grid, as rune-light's glass, so the rim's
    // fringe does not stair-step.
    let o = array<vec2<f32>, 4>(vec2<f32>(-0.125, -0.375), vec2<f32>(0.375, -0.125), vec2<f32>(0.125, 0.375), vec2<f32>(-0.375, 0.125));
    var acc = vec4<f32>(0.0);
    for (var n = 0; n < 4; n = n + 1) {
        let q = i.p + o[n];
        acc = acc + shade(k, q, min(sdf(k, q), 0.0), i.optics);
    }
    return encoded(acc * 0.25) * cov;
}
