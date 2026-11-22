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
pub fn glass(el: El, radius: f32) -> El {
    // The close shadow of the look the prism was fitted to, without its spread: it stays under
    // the element, so a small pill on a dark page gets no dark cloud around it.
    let k = radius / 41.0;
    let shadow = Shadow { x: 3.28 * k, y: 9.82 * k, blur: 33.0 * k, color: Color::BLACK.alpha(0.118), ..Default::default() };
    el.radius(radius).box_shadow(shadow).backdrop_lens(Lens::prism())
}
