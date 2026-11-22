//! Liquid glass for pills and floating bars: `Lens::prism()` with one close, soft shadow.

use rune::{Color, Cx, El, Lens, Shadow};

/// Whether [`glass`] can draw this frame: the window can copy its surface, a GPU draws it, and
/// reduced motion is off. Reduced motion stands in for reduced transparency, which Rune does not
/// report.
pub fn glass_draws<S>(cx: &Cx<S>) -> bool {
    cx.backdrop_copies() && cx.on_gpu() && !cx.is_reduced_motion()
}

/// `el` as liquid glass with corner radius `radius`: `Lens::prism()` and one close, soft shadow,
/// drawn for a 41 point radius and scaled to this one. No fill: what is behind shows through.
///
/// Call it only while [`glass_draws`] is true, and give the element a solid fill otherwise. Where
/// the frame can't be copied the glass draws nothing, so a pill with no fill would vanish:
///
/// ```
/// # use rune::{Color, Cx, El};
/// # fn pill(cx: &Cx<()>, el: El) -> El {
/// if rune_kit::glass_draws(cx) {
///     rune_kit::glass(el, 20.0)
/// } else {
///     el.radius(20.0).shadow(2.0, 12.0, Color::BLACK.alpha(0.1)).fill(Color::WHITE)
/// }
/// # }
/// ```
///
/// Labels on the glass sit on whatever is behind it, so their contrast is the app's job. It suits
/// pills and bars, not window-sized panels: each pixel costs about three times a plain lens.
/// Inside a see-through layer (`El::layer_opacity`) it reads the backdrop as opaque and draws dark.
pub fn glass(el: El, radius: f32) -> El { glass_with(el, radius, &Glass::default(), Color::TRANSPARENT) }

/// [`glass`] tuned by `g`, with `tint` at `g.tint` alpha over the glass (its own alpha is
/// ignored). `glass_with(el, r, &Glass::default(), c)` is `glass(el, r)`.
pub fn glass_with(el: El, radius: f32, g: &Glass, tint: Color) -> El {
    // The shadow is the close one of the look the prism was fitted to, without its spread by
    // default: it stays under the element, so a small pill on a dark page gets no dark cloud.
    let k = radius / 41.0;
    let shadow = Shadow {
        x: g.shadow_x * k,
        y: g.shadow_y * k,
        blur: g.shadow_blur.max(0.0) * k,
        spread: g.shadow_spread * k,
        color: Color::BLACK.alpha(g.shadow_alpha.clamp(0.0, 1.0)),
        ..Default::default()
    };
    let lens = Lens::prism().bend(g.bend).split(g.split).shade(g.shade).light(g.lights).ring(g.ring);
    let el = el.radius(radius).box_shadow(shadow).backdrop_lens(lens);
    let el = if g.frost > 0.0 { el.backdrop_blur(g.frost) } else { el };
    if g.tint > 0.0 { el.fill(tint.alpha(g.tint.min(1.0))) } else { el }
}

/// The settings of [`glass_with`]. `Default` is [`glass`]'s look. The shadow is for a 41 point
/// radius and scales with the element's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glass {
    /// How far the rim pulls the backdrop in, times the fitted bend (`Lens::bend`).
    pub bend: f32,
    /// How far apart the colours land under the rim; 0 is no colour bands (`Lens::split`).
    pub split: f32,
    /// The fitted darkening and warming by depth; 0 leaves the backdrop unshaded (`Lens::shade`).
    pub shade: f32,
    /// The two lights that glint off the rim (`Lens::light`).
    pub lights: f32,
    /// The thin ring's opacity (`Lens::ring`).
    pub ring: f32,
    /// Blur of the backdrop in the middle, sigma in points; 0 is clear (`El::backdrop_blur`).
    pub frost: f32,
    /// Alpha of the tint colour over the glass; 0 is none.
    pub tint: f32,
    pub shadow_x: f32,
    pub shadow_y: f32,
    /// Shadow blur radius (CSS), in points.
    pub shadow_blur: f32,
    pub shadow_spread: f32,
    pub shadow_alpha: f32,
}

impl Default for Glass {
    fn default() -> Self {
        Glass { bend: 1.0, split: 1.0, shade: 1.0, lights: 1.0, ring: 1.0, frost: 0.0, tint: 0.0, shadow_x: 3.28, shadow_y: 9.82, shadow_blur: 33.0, shadow_spread: 0.0, shadow_alpha: 0.118 }
    }
}

impl Glass {
    /// One flat JSON object: the field names as keys, in order, each number with at most three
    /// decimals.
    pub fn to_json(&self) -> String {
        let fields = [
            ("bend", self.bend), ("split", self.split), ("shade", self.shade), ("lights", self.lights), ("ring", self.ring),
            ("frost", self.frost), ("tint", self.tint), ("shadow_x", self.shadow_x), ("shadow_y", self.shadow_y),
            ("shadow_blur", self.shadow_blur), ("shadow_spread", self.shadow_spread), ("shadow_alpha", self.shadow_alpha),
        ];
        let body: Vec<String> = fields.iter().map(|(k, v)| format!("\"{k}\":{}", number(*v))).collect();
        format!("{{{}}}", body.join(","))
    }
}

/// `v` with at most three decimals and no trailing zeros; JSON has no NaN or infinity, so 0.
fn number(v: f32) -> String {
    if !v.is_finite() { return "0".into(); }
    let s = format!("{:.3}", v);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.into() }
}
