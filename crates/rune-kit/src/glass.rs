//! Liquid glass for pills and floating bars: a lightly bent, lightly frosted `Lens::prism()` over a
//! soft, wide shadow.

use rune::theme::{blends_srgb, srgb_decode, srgb_encode};
use rune::{Color, Cx, El, Lens, Shadow};

/// Whether [`glass`] can draw this frame: the window can copy its surface, a GPU draws it, and
/// reduced motion is off. Reduced motion stands in for reduced transparency, which Rune does not
/// report.
pub fn glass_draws<S>(cx: &Cx<S>) -> bool {
    cx.backdrop_copies() && cx.on_gpu() && !cx.is_reduced_motion()
}

/// `el` as liquid glass with corner radius `radius`, drawn for a 41 point radius and scaled to this
/// one: a `Lens::prism()` with a fifth of its fitted bend and shade, no colour bands, most of its
/// lights and a trace of its ring, a light frost, a faint white veil and white hairlines, over one
/// wide, soft shadow that spreads past the element and falls below it. What is behind shows
/// through, a little blurred and lifted.
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
/// ignored), white at `g.veil` alpha over that, and `g.hairline` white lines inside the top and
/// bottom edges. `glass_with(el, r, &Glass::default(), c)` is `glass(el, r)`. Where the glass does
/// not draw (past `MAX_BACKDROP_LENSES`, or a frame that can't be copied) the fill and the
/// hairlines still draw, over sharp content.
pub fn glass_with(el: El, radius: f32, g: &Glass, tint: Color) -> El {
    let k = radius / 41.0;
    // A value that is not a number draws as none; a huge one stops at 1000 points.
    let fin = |v: f32| if v.is_finite() { v.clamp(-1000.0, 1000.0) } else { 0.0 };
    let shadow = Shadow {
        x: fin(g.shadow_x) * k,
        y: fin(g.shadow_y) * k,
        blur: fin(g.shadow_blur).max(0.0) * k,
        spread: fin(g.shadow_spread) * k,
        color: Color::BLACK.alpha(fin(g.shadow_alpha).clamp(0.0, 1.0)),
        ..Default::default()
    };
    let lens = Lens::prism().bend(g.bend).split(g.split).shade(g.shade).light(g.lights).ring(g.ring);
    let el = el.radius(radius).box_shadow(shadow).backdrop_lens(lens);
    let el = if g.frost > 0.0 { el.backdrop_blur(g.frost) } else { el };
    let el = if g.hairline > 0.0 {
        // One point in from the top and bottom edges; the caps fade them out toward the sides.
        let line = |y: f32| Shadow { y, color: Color::WHITE.alpha(g.hairline.min(1.0)), inset: true, ..Default::default() };
        el.box_shadow(line(1.0)).box_shadow(line(-1.0))
    } else {
        el
    };
    let tint = if g.tint > 0.0 { Some(tint.alpha(g.tint.min(1.0))) } else { None };
    match (tint, g.veil > 0.0) {
        (None, false) => el,
        (Some(t), false) => el.fill(t),
        (t, true) => el.fill(veil_over(t.unwrap_or(Color::TRANSPARENT), g.veil.min(1.0))),
    }
}

/// White at alpha `v` over `under` (straight alpha), as one colour that paints as the two layers
/// do: mixed in sRGB encoding when the frame blends there (`App::srgb_blend`), else in linear light.
fn veil_over(under: Color, v: f32) -> Color {
    let a = v + under.a * (1.0 - v);
    let srgb = blends_srgb();
    let mix = |c: f32| {
        let m = (v + if srgb { srgb_encode(c) } else { c } * under.a * (1.0 - v)) / a;
        if srgb { srgb_decode(m) } else { m }
    };
    Color { r: mix(under.r), g: mix(under.g), b: mix(under.b), a }
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
    /// Alpha of a white veil over the glass and the tint: it lifts a dark backdrop to a grey; 0 is
    /// none.
    pub veil: f32,
    /// Alpha of the white lines one point inside the top and bottom edges; 0 is none.
    pub hairline: f32,
}

impl Default for Glass {
    fn default() -> Self { Glass::DEFAULT }
}

impl Glass {
    /// [`glass`]'s look, for a `const`. Build a `Glass` from this (`Glass { frost: 6.0, ..Glass::DEFAULT }`),
    /// not from a full literal: a later field then does not break the build. `Lens::prism()` at its
    /// fitted look, under the close shadow it was fitted with, is `Glass { bend: 1.0, split: 1.0,
    /// shade: 1.0, lights: 1.0, ring: 1.0, frost: 0.0, shadow_x: 3.28, shadow_y: 9.82, shadow_blur:
    /// 33.0, shadow_spread: 0.0, shadow_alpha: 0.118, veil: 0.0, hairline: 0.0, ..Glass::DEFAULT }`.
    pub const DEFAULT: Glass =
        Glass { bend: 0.19, split: 0.0, shade: 0.196, lights: 0.821, ring: 0.071, frost: 1.714, tint: 0.0, shadow_x: 4.286, shadow_y: 21.429, shadow_blur: 61.429, shadow_spread: 21.429, shadow_alpha: 0.185, veil: 0.052, hairline: 0.167 };

    /// One flat JSON object: the field names as keys, in order, each number with at most three
    /// decimals.
    pub fn to_json(&self) -> String {
        let fields = [
            ("bend", self.bend), ("split", self.split), ("shade", self.shade), ("lights", self.lights), ("ring", self.ring),
            ("frost", self.frost), ("tint", self.tint), ("shadow_x", self.shadow_x), ("shadow_y", self.shadow_y),
            ("shadow_blur", self.shadow_blur), ("shadow_spread", self.shadow_spread), ("shadow_alpha", self.shadow_alpha),
            ("veil", self.veil), ("hairline", self.hairline),
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
