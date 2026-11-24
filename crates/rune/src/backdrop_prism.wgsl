// Generated: the prism's numbers, fitted to one reference rendering of the look. See prism() below.
const PRISM_NH: i32 = 16;
const PRISM_NH2: u32 = 32u;
const PRISM_NRD: i32 = 14;
const PRISM_KT = array<f32, 25>(0.00000, 0.50000, 1.00000, 1.50000, 2.00000, 2.50000, 3.00000, 3.50000, 4.00000, 5.00000, 6.00000, 7.00000, 8.00000, 10.00000, 12.00000, 15.00000, 19.00000, 23.00000, 27.00000, 31.00000, 34.00000, 36.00000, 38.00000, 39.50000, 41.00000);
const PRISM_D = array<vec4<f32>, 25>(
    vec4<f32>(22.44450, -12.26570, 2.28137, 1.32366),
    vec4<f32>(9.42197, 37.77737, 29.24877, 37.55758),
    vec4<f32>(44.13781, 45.74563, 40.01756, 33.97793),
    vec4<f32>(45.13585, 50.50776, 37.24442, 33.00022),
    vec4<f32>(43.49076, 37.97548, 35.96377, 32.50339),
    vec4<f32>(42.03235, 26.58899, 34.27953, 29.78764),
    vec4<f32>(38.87984, 1.42321, 32.92599, 25.93372),
    vec4<f32>(39.35427, 31.62191, 31.73049, 29.86577),
    vec4<f32>(36.07913, 28.38939, 30.33216, 25.50614),
    vec4<f32>(32.15567, 31.26861, 28.17787, 24.62192),
    vec4<f32>(29.74280, 26.38992, 25.26574, 22.73060),
    vec4<f32>(26.40629, 27.41064, 22.50546, 21.25813),
    vec4<f32>(24.07747, 22.69514, 20.63500, 18.58957),
    vec4<f32>(21.63961, 21.88138, 18.80524, 17.83444),
    vec4<f32>(18.23051, 14.63468, 13.89174, 10.63987),
    vec4<f32>(11.63209, 14.01126, 9.16427, 5.37863),
    vec4<f32>(4.26438, 8.27765, 2.81054, 0.43892),
    vec4<f32>(6.57273, 6.20268, 5.48565, 4.75869),
    vec4<f32>(3.45283, 2.91314, 2.75947, 2.44410),
    vec4<f32>(1.82892, 2.07931, 1.57401, 1.30317),
    vec4<f32>(1.21370, 1.20699, 0.96712, 0.75127),
    vec4<f32>(0.70804, 0.45404, 0.53025, 0.42774),
    vec4<f32>(0.39956, 0.63459, 0.39169, 0.21482),
    vec4<f32>(0.53829, 0.36534, 0.33227, 0.57800),
    vec4<f32>(0.35786, 0.41202, 0.35686, 0.15356),
);
const PRISM_W = array<vec3<f32>, 4>(vec3<f32>(0.57347, 0.03151, 0.01211), vec3<f32>(0.17026, 0.03018, 0.00440), vec3<f32>(0.19307, 0.80048, 0.02745), vec3<f32>(0.06320, 0.13784, 0.95604));
const PRISM_ST = array<f32, 12>(0.00000, 1.00000, 2.00000, 3.00000, 5.00000, 8.00000, 12.00000, 17.00000, 23.00000, 30.00000, 36.00000, 41.00000);
const PRISM_G = array<vec3<f32>, 12>(
    vec3<f32>(1.07813, 0.83722, 0.67902),
    vec3<f32>(0.70910, 0.92772, 0.83522),
    vec3<f32>(0.91985, 0.93231, 0.85612),
    vec3<f32>(0.91740, 0.92028, 0.88693),
    vec3<f32>(0.89394, 0.88294, 0.84936),
    vec3<f32>(0.89272, 0.88079, 0.85863),
    vec3<f32>(0.87590, 0.86017, 0.84402),
    vec3<f32>(0.85259, 0.84407, 0.83531),
    vec3<f32>(0.84972, 0.84629, 0.84310),
    vec3<f32>(0.85996, 0.85931, 0.85564),
    vec3<f32>(0.88094, 0.88054, 0.88100),
    vec3<f32>(0.88696, 0.88714, 0.88693),
);
const PRISM_LT = array<f32, 10>(0.00000, 0.50000, 1.00000, 1.50000, 2.00000, 2.50000, 3.00000, 4.00000, 6.00000, 9.00000);
const PRISM_L = array<f32, 10>(0.00000, 0.00000, 0.00000, 0.00000, 0.00000, 0.00000, 0.00000, 0.00000, 0.00000, 0.00000);
const PRISM_RD = array<f32, 14>(-3.00000, -2.50000, -2.00000, -1.50000, -1.00000, -0.75000, -0.50000, -0.25000, 0.00000, 0.25000, 0.50000, 0.75000, 1.00000, 1.50000);
const PRISM_RING = array<vec4<f32>, 14>(
    vec4<f32>(-14.21990, -14.94058, -6.41429, -8.35126),
    vec4<f32>(-2.97422, -25.47901, -0.56698, -5.70323),
    vec4<f32>(-1.34887, -4.07849, -2.83530, -3.14619),
    vec4<f32>(-0.18854, -1.95729, -1.00001, -2.90730),
    vec4<f32>(0.39462, 0.08973, -0.27179, -1.72151),
    vec4<f32>(0.39848, -0.10976, 0.40856, -1.67261),
    vec4<f32>(0.09590, 0.13145, 0.56187, -1.50882),
    vec4<f32>(-1.09830, -2.56756, -1.00020, -4.33129),
    vec4<f32>(-2.41944, -13.67696, -11.65221, -11.24098),
    vec4<f32>(-5.00175, -5.64792, -5.00920, -4.49398),
    vec4<f32>(-0.11526, -0.22961, 1.34736, -3.68220),
    vec4<f32>(-16.47416, -17.39335, -18.45947, -5.77979),
    vec4<f32>(-28.84362, -29.86819, -30.65253, -6.49695),
    vec4<f32>(-11.75914, -13.92275, -11.70869, -4.55582),
);
const PRISM_HA = array<vec4<f32>, 32>(
    vec4<f32>(0.01776, -0.02748, 0.00000, 0.06283),
    vec4<f32>(0.01658, 0.03175, 0.00000, 1.70043),
    vec4<f32>(0.01178, 0.04778, 0.00000, 0.87174),
    vec4<f32>(0.01324, 0.03839, 0.00000, 0.37380),
    vec4<f32>(0.01808, 0.00172, 0.00000, 0.56442),
    vec4<f32>(0.01945, -0.02709, 0.00000, -0.96391),
    vec4<f32>(0.00451, -0.01993, 0.00000, -0.19038),
    vec4<f32>(-0.01473, 0.00853, 0.00000, -0.65688),
    vec4<f32>(-0.00054, 0.02801, 0.00000, -0.48386),
    vec4<f32>(-0.02016, 0.01573, 0.00000, 0.37860),
    vec4<f32>(0.03641, -0.00169, 0.00000, -0.14347),
    vec4<f32>(0.02068, -0.00196, 0.00000, 0.05265),
    vec4<f32>(-0.01040, 0.00201, 0.00000, -0.10694),
    vec4<f32>(0.01250, 0.01249, 0.00000, 0.00752),
    vec4<f32>(0.01094, -0.00065, 0.00000, 0.16291),
    vec4<f32>(-0.01549, -0.02816, 0.00000, 0.18281),
    vec4<f32>(-0.00441, -0.00765, 0.00000, 0.06865),
    vec4<f32>(-0.01270, 0.00947, 0.00000, 0.02797),
    vec4<f32>(0.02511, 0.01406, 0.00000, -0.03576),
    vec4<f32>(0.00155, -0.00014, 0.00000, -0.06850),
    vec4<f32>(-0.00214, 0.00095, 0.00000, -0.01201),
    vec4<f32>(0.00090, 0.00270, 0.00000, -0.10164),
    vec4<f32>(-0.00083, -0.00088, 0.00000, 0.02339),
    vec4<f32>(-0.00732, -0.00167, 0.00000, 0.08634),
    vec4<f32>(0.00361, -0.00746, 0.00000, -0.06816),
    vec4<f32>(-0.00082, -0.00076, 0.00000, -0.06148),
    vec4<f32>(-0.00027, 0.00028, 0.00000, -0.00408),
    vec4<f32>(0.00113, 0.00635, 0.00000, 0.02830),
    vec4<f32>(0.00343, 0.00465, 0.00000, -0.03094),
    vec4<f32>(0.00099, 0.00098, 0.00000, 0.02931),
    vec4<f32>(-0.00027, -0.00009, 0.00000, 0.01943),
    vec4<f32>(-0.00112, 0.01119, 0.00000, 0.11764),
);
const PRISM_HK = array<vec4<f32>, 32>(
    vec4<f32>(-0.01262, 0.06349, 0.00104, -0.04646),
    vec4<f32>(0.02258, -0.01001, 0.00078, 0.02610),
    vec4<f32>(0.01820, 0.01698, 0.03007, 0.00271),
    vec4<f32>(-0.01125, -0.06878, -0.02000, -0.01199),
    vec4<f32>(-0.02482, 0.03174, -0.01166, 0.03405),
    vec4<f32>(0.00390, -0.01911, -0.00225, -0.00343),
    vec4<f32>(0.01101, 0.01443, 0.00720, 0.02169),
    vec4<f32>(0.01907, -0.01880, 0.01816, 0.02602),
    vec4<f32>(0.00968, 0.07176, 0.00975, 0.00074),
    vec4<f32>(0.02170, 0.05087, 0.03245, 0.00231),
    vec4<f32>(-0.01284, -0.03594, -0.02763, 0.00167),
    vec4<f32>(-0.01758, -0.04178, -0.01900, -0.05321),
    vec4<f32>(0.00483, 0.01803, 0.01567, 0.03400),
    vec4<f32>(-0.00130, -0.00633, 0.00248, -0.03603),
    vec4<f32>(-0.00445, -0.02177, -0.00927, 0.02980),
    vec4<f32>(0.02039, -0.02960, -0.00494, 0.02694),
    vec4<f32>(0.00820, -0.01234, -0.00216, 0.01329),
    vec4<f32>(0.01534, 0.01927, 0.02298, -0.04074),
    vec4<f32>(-0.01554, -0.01239, -0.01276, -0.06311),
    vec4<f32>(-0.00082, -0.02121, 0.00167, -0.02120),
    vec4<f32>(0.00092, 0.01071, 0.00234, 0.00263),
    vec4<f32>(-0.00165, 0.00631, -0.00174, 0.00440),
    vec4<f32>(-0.00030, 0.01377, 0.00058, 0.00285),
    vec4<f32>(-0.00146, 0.00029, 0.00290, 0.01287),
    vec4<f32>(0.00215, -0.00356, -0.00399, -0.01170),
    vec4<f32>(0.00015, 0.00803, 0.00180, -0.00056),
    vec4<f32>(-0.00050, -0.01018, -0.00011, 0.00385),
    vec4<f32>(-0.00746, 0.00893, 0.00230, -0.00180),
    vec4<f32>(0.00350, -0.01456, -0.00413, 0.00104),
    vec4<f32>(-0.00204, 0.01935, 0.00542, -0.00551),
    vec4<f32>(0.00016, -0.01115, 0.00353, 0.00448),
    vec4<f32>(0.00275, 0.00345, 0.00457, 0.00483),
);
const PRISM_HS = array<vec4<f32>, 32>(
    vec4<f32>(0.09373, 0.09814, 0.19417, -0.00036),
    vec4<f32>(-1.01053, -0.92260, -1.20563, -0.02643),
    vec4<f32>(-0.07317, -0.03569, 0.12331, 0.05876),
    vec4<f32>(-0.98745, -1.06600, -1.03324, 0.07965),
    vec4<f32>(-0.09152, -0.04607, -0.08307, 0.09664),
    vec4<f32>(-0.22376, -0.20014, -0.24072, 0.02803),
    vec4<f32>(0.05443, -0.02120, 0.04482, -0.02141),
    vec4<f32>(-0.07695, -0.07283, -0.07963, -0.02934),
    vec4<f32>(0.09254, 0.13309, 0.05649, 0.07611),
    vec4<f32>(-0.24950, -0.33194, -0.27694, 0.02482),
    vec4<f32>(-0.08323, -0.07699, -0.13588, -0.03542),
    vec4<f32>(-0.20002, -0.14050, -0.21262, -0.02438),
    vec4<f32>(-0.18758, -0.17814, -0.12368, 0.02690),
    vec4<f32>(-0.17881, -0.21716, -0.25268, 0.01958),
    vec4<f32>(-0.09767, -0.05928, -0.11686, 0.00657),
    vec4<f32>(0.07206, 0.07588, 0.11045, -0.01869),
    vec4<f32>(0.00462, -0.04777, -0.07994, 0.01183),
    vec4<f32>(-0.01058, -0.04013, -0.11053, 0.00795),
    vec4<f32>(-0.01120, -0.01960, 0.04873, 0.00497),
    vec4<f32>(0.03465, 0.05348, 0.05276, -0.00340),
    vec4<f32>(-0.02689, -0.01077, -0.01608, 0.01299),
    vec4<f32>(-0.07640, -0.08835, -0.07638, -0.00052),
    vec4<f32>(0.00714, 0.05678, 0.03258, -0.00482),
    vec4<f32>(0.03314, 0.01647, -0.01554, -0.00278),
    vec4<f32>(0.01937, -0.02564, -0.01376, 0.00709),
    vec4<f32>(-0.03386, -0.03555, -0.07675, -0.00019),
    vec4<f32>(-0.00263, -0.03802, 0.01452, -0.00461),
    vec4<f32>(-0.00797, -0.03137, -0.02401, 0.00429),
    vec4<f32>(-0.04509, -0.04003, -0.03808, 0.02526),
    vec4<f32>(-0.03652, -0.03524, -0.01955, 0.00362),
    vec4<f32>(0.00512, -0.00547, -0.01343, -0.00663),
    vec4<f32>(0.02917, 0.04077, 0.02382, -0.01177),
);
const PRISM_HC = array<vec4<f32>, 32>(
    vec4<f32>(-0.03186, 0.22669, 0.04248, 0.00000),
    vec4<f32>(-0.33982, -0.24868, -0.05303, 0.00000),
    vec4<f32>(-0.21126, -0.31805, -0.35480, 0.00000),
    vec4<f32>(0.39018, 0.73309, 0.63827, 0.00000),
    vec4<f32>(-0.39694, -0.17944, -0.20992, 0.00000),
    vec4<f32>(-0.38248, -0.57262, -0.23739, 0.00000),
    vec4<f32>(0.19944, 0.32908, -0.09457, 0.00000),
    vec4<f32>(0.28251, -0.22394, -0.48631, 0.00000),
    vec4<f32>(-0.54934, -1.05003, -0.42969, 0.00000),
    vec4<f32>(-0.13818, 0.03475, -0.35206, 0.00000),
    vec4<f32>(-0.08367, -0.03438, 0.06220, 0.00000),
    vec4<f32>(-0.18039, -0.16497, 0.02703, 0.00000),
    vec4<f32>(-0.02616, 0.09035, -0.15817, 0.00000),
    vec4<f32>(-0.06622, 0.09349, 0.02459, 0.00000),
    vec4<f32>(0.01877, -0.10760, 0.05948, 0.00000),
    vec4<f32>(0.11767, -0.23599, -0.18651, 0.00000),
    vec4<f32>(-0.01612, -0.19127, -0.11798, 0.00000),
    vec4<f32>(-0.16967, -0.04225, 0.03976, 0.00000),
    vec4<f32>(-0.03847, 0.14519, 0.18742, 0.00000),
    vec4<f32>(-0.08231, -0.09105, -0.10926, 0.00000),
    vec4<f32>(-0.04522, -0.12395, -0.02871, 0.00000),
    vec4<f32>(0.00085, -0.00034, 0.01466, 0.00000),
    vec4<f32>(0.01808, 0.00610, -0.01765, 0.00000),
    vec4<f32>(-0.03769, -0.09318, -0.00470, 0.00000),
    vec4<f32>(-0.03693, -0.11175, -0.10225, 0.00000),
    vec4<f32>(0.03565, -0.02941, 0.01220, 0.00000),
    vec4<f32>(0.00006, 0.04046, -0.01501, 0.00000),
    vec4<f32>(-0.08012, -0.05370, -0.02466, 0.00000),
    vec4<f32>(0.02560, 0.00300, -0.09266, 0.00000),
    vec4<f32>(0.02578, 0.02418, 0.00338, 0.00000),
    vec4<f32>(0.02678, -0.01251, 0.00868, 0.00000),
    vec4<f32>(-0.01112, -0.01969, -0.10522, 0.00000),
);
const PRISM_LIGHT1 = array<f32, 5>(-3.98912, 0.50787, 9.44808, 0.90111, 0.83233);
const PRISM_LIGHT2 = array<f32, 5>(-0.56476, 1.00564, 5.83087, 1.14667, 0.53057);
const PRISM_VY: f32 = 0.15316;
const PRISM_SIDE_W: f32 = 1.94115;
const PRISM_GROW: f32 = 0.08039;

// `Lens::prism`: liquid glass. Four colour samples (`PRISM_W`, linear RGB, each channel's weights sum
// to one) each read the backdrop moved in along the rim's inward normal by its own distance
// (`PRISM_D`, piecewise linear in the distance in from the rim, `PRISM_KT`, points), so red reaches
// farthest and blue least: a light/dark edge under the rim splits into bands of yellow, red and cyan.
// What lies outside the element is not read (transparent). The bent backdrop is shaded (a per-colour
// gain by depth, `PRISM_G`, a top to bottom term and a side term near the rim), lit near the rim, and a
// thin ring lies over the edge. The glass is not mirror-symmetric: most terms vary with the direction
// of the rim's normal, as a few harmonics (`PRISM_H*`). It is defined in sRGB encoding, as a browser
// composes; a frame that blends in linear light gets the same colours decoded.

fn prism_lerp(x: f32, x0: f32, x1: f32) -> f32 { return clamp((x - x0) / (x1 - x0), 0.0, 1.0); }

fn prism_bend(t: f32) -> vec4<f32> {
    var i = 0;
    for (; i < 23; i = i + 1) { if (t < PRISM_KT[i + 1]) { break; } }
    return mix(PRISM_D[i], PRISM_D[i + 1], prism_lerp(t, PRISM_KT[i], PRISM_KT[i + 1]));
}

fn prism_gain(t: f32) -> vec3<f32> {
    var i = 0;
    for (; i < 10; i = i + 1) { if (t < PRISM_ST[i + 1]) { break; } }
    return mix(PRISM_G[i], PRISM_G[i + 1], prism_lerp(t, PRISM_ST[i], PRISM_ST[i + 1]));
}

fn prism_glow(t: f32) -> f32 {
    var i = 0;
    for (; i < 8; i = i + 1) { if (t < PRISM_LT[i + 1]) { break; } }
    return mix(PRISM_L[i], PRISM_L[i + 1], prism_lerp(t, PRISM_LT[i], PRISM_LT[i + 1]));
}

fn prism_ring(d: f32) -> vec4<f32> {
    var i = 0;
    for (; i < PRISM_NRD - 2; i = i + 1) { if (d < PRISM_RD[i + 1]) { break; } }
    return mix(PRISM_RING[i], PRISM_RING[i + 1], prism_lerp(d, PRISM_RD[i], PRISM_RD[i + 1]));
}

// 1 + sum over harmonics h of a cos(h th) + b sin(h th), for four curves at once.
fn prism_harm(c: array<vec4<f32>, PRISM_NH2>, th: f32) -> vec4<f32> {
    var s = vec4<f32>(1.0);
    for (var h = 1; h <= PRISM_NH; h = h + 1) {
        s = s + c[2 * h - 2] * cos(f32(h) * th) + c[2 * h - 1] * sin(f32(h) * th);
    }
    return s;
}

fn prism_normal(k: Slab, p: vec2<f32>) -> vec2<f32> {
    let e = 0.5;
    let g = vec2<f32>(sdf(k, p + vec2<f32>(e, 0.0)) - sdf(k, p - vec2<f32>(e, 0.0)), sdf(k, p + vec2<f32>(0.0, e)) - sdf(k, p - vec2<f32>(0.0, e)));
    return g / max(length(g), 1e-6);
}

// The glass's own edge: the rect's signed distance moved by a direction term, in device pixels.
fn prism_sdf(k: Slab, p: vec2<f32>, n: vec2<f32>, kp: f32) -> f32 {
    let th = atan2(n.y, n.x);
    return sdf(k, p) + (PRISM_GROW + prism_harm(PRISM_HS, th).w - 1.0) * kp;
}

// The backdrop at q in linear light, premultiplied, its colour 0 outside the element's rect (its
// alpha as read: inside a layer the backdrop can be transparent). `frost` (0…1) mixes in the
// record's blur.
fn prism_read(k: Slab, q: vec2<f32>, frost: f32) -> vec4<f32> {
    let lo = clamp(q - k.rect.xy + vec2<f32>(0.5), vec2<f32>(0.0), vec2<f32>(1.0));
    let hi = clamp(k.rect.xy + k.rect.zw - q + vec2<f32>(0.5), vec2<f32>(0.0), vec2<f32>(1.0));
    var c = textureSampleLevel(sharp, smp, q / vec2<f32>(textureDimensions(sharp)), 0.0);
    if (frost > 0.0) {
        c = mix(c, textureSampleLevel(blurred, smp, (q - u.place.xy) / u.place.zw, 0.0), frost);
    }
    let l = linear(c);
    return vec4<f32>(l.rgb * (lo.x * lo.y * hi.x * hi.y), l.a);
}

// `v` moved from `base` by `k` times: exactly `v` at 1, so the fitted look keeps its bytes.
fn prism_k3(v: vec3<f32>, base: vec3<f32>, k: f32) -> vec3<f32> { return select(base + (v - base) * k, v, k == 1.0); }
fn prism_k4(v: vec4<f32>, base: vec4<f32>, k: f32) -> vec4<f32> { return select(base + (v - base) * k, v, k == 1.0); }

fn prism_light(l: array<f32, 5>, n: vec2<f32>, t: f32) -> f32 {
    let x = (t - l[4]) / max(abs(l[3]), 0.05);
    return l[1] * pow(max(n.x * cos(l[0]) + n.y * sin(l[0]), 1e-6), max(l[2], 0.1)) * exp(-x * x);
}

// The prism at device pixel p: the glass (straight, sRGB-encoded), its own edge's signed distance in
// device pixels, the ring (straight colour, alpha), and how opaque the backdrop is (`a`). `kp` is device pixels per point: the ring
// and the edge are in points. The bend, gain and glow are in units of the corner radius (`prism_unit`).
struct Prism { glass: vec3<f32>, d: f32, ring: vec4<f32>, a: f32 };

// Device pixels per unit of depth: the fit's pill has a 41 point radius, so its numbers are in 41ths
// of the largest corner radius (at most half the short side, at least one pixel). A pill of any size
// shows the same bands, smaller; a sharp rect barely bends.
fn prism_unit(k: Slab) -> f32 {
    let r = max(max(abs(k.radii.x), abs(k.radii.y)), max(abs(k.radii.z), abs(k.radii.w)));
    return max(min(r, 0.5 * min(k.rect.z, k.rect.w)), 1.0) / 41.0;
}

// `tune`: bend, split, shade and ring (`Lens::prism`), each 1 as fitted; `lights` scales the rim's
// two lights; `frosted` (0 or 1) shows the record's blur from a little in from the rim.
fn prism(k: Slab, p: vec2<f32>, kp: f32, tune: vec4<f32>, lights: f32, frosted: f32) -> Prism {
    let u = prism_unit(k);
    let o = array<vec2<f32>, 4>(vec2<f32>(-0.125, -0.375), vec2<f32>(0.375, -0.125), vec2<f32>(0.125, 0.375), vec2<f32>(-0.375, 0.125));
    var acc = vec3<f32>(0.0);
    var alpha = 0.0;
    for (var s = 0; s < 4; s = s + 1) {
        let q = p + o[s];
        let n = prism_normal(k, q);
        let th = atan2(n.y, n.x);
        let t = max(-prism_sdf(k, q, n, kp), 0.0) / u;
        let a = prism_harm(PRISM_HA, th);
        let b = prism_bend(t * max(a.y, 0.2)) * a.x * prism_harm(PRISM_HK, th) * u * tune.x;
        let bend = prism_k4(b, vec4<f32>(0.25 * (b.x + b.y + b.z + b.w)), tune.y);
        // Past the bands the middle shows the blur. Below bend 1 the blur starts nearer the rim as
        // the bend falls, so with no bend it reaches the rim (no bands to keep sharp); at bend 1 and
        // above it starts where it always did.
        let fb = min(tune.x, 1.0);
        let frost = frosted * select(smoothstep(10.0 * fb, 24.0 * fb, t), 1.0, fb <= 0.0);
        for (var c = 0; c < 4; c = c + 1) {
            let b = prism_read(k, q - n * bend[c], frost);
            acc = acc + b.rgb * PRISM_W[c];
            alpha = alpha + b.a;
        }
    }
    // Straight colour: a transparent part of a layer is no colour, not black. Over an opaque
    // backdrop the sixteen alphas sum to exactly 16, so the glass is as fitted.
    alpha = alpha / 16.0;
    acc = acc * 0.25 / max(alpha, 1e-6);
    let n = prism_normal(k, p);
    let th = atan2(n.y, n.x);
    let d = prism_sdf(k, p, n, kp);
    let t = max(-d, 0.0) / u;
    let a = prism_harm(PRISM_HA, th);
    let hs = prism_harm(PRISM_HS, th);
    let v = (p.y - k.rect.y) / k.rect.w - 0.5;
    let gain = prism_k3(prism_gain(t) * (1.0 + PRISM_VY * v), vec3<f32>(1.0), tune.z);
    let side = prism_k3(vec3<f32>(1.0) + (hs.xyz - vec3<f32>(1.0)) * exp(-t / PRISM_SIDE_W), vec3<f32>(1.0), tune.z);
    let glow = max(prism_glow(t) * a.z, 0.0) + prism_light(PRISM_LIGHT1, n, t) * lights + prism_light(PRISM_LIGHT2, n, t) * lights;
    let lit = clamp(acc * gain * side + vec3<f32>(glow), vec3<f32>(0.0), vec3<f32>(1.0));
    let enc = select(1.055 * pow(lit, vec3<f32>(1.0 / 2.4)) - 0.055, lit * 12.92, lit <= vec3<f32>(0.0031308));
    let r = prism_ring(d / kp);
    let rc = vec3<f32>(1.0) / (vec3<f32>(1.0) + exp(-(r.xyz + prism_harm(PRISM_HC, th).xyz - vec3<f32>(1.0))));
    let ra = min(select(0.0, 1.0 / (1.0 + exp(-(r.w + a.w - 1.0))), d / kp <= 1.5) * tune.w, 1.0);
    return Prism(enc, d, vec4<f32>(rc, ra), alpha);
}

// The ring over the glass over what is behind: premultiplied, in the frame's blend space (the prism
// is defined in sRGB encoding). The glass covers only as much as the backdrop it shows.
fn prism_out(c: Prism) -> vec4<f32> {
    var ring = c.ring.rgb;
    var glass = c.glass;
    if (g.flags.x < 0.5) {
        ring = select(pow((ring + 0.055) / 1.055, vec3<f32>(2.4)), ring / 12.92, ring <= vec3<f32>(0.04045));
        glass = select(pow((glass + 0.055) / 1.055, vec3<f32>(2.4)), glass / 12.92, glass <= vec3<f32>(0.04045));
    }
    let cov = clamp(0.5 - c.d, 0.0, 1.0) * c.a;
    return vec4<f32>(ring * c.ring.a + glass * cov * (1.0 - c.ring.a), c.ring.a + cov * (1.0 - c.ring.a));
}
