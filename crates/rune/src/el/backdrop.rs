//! Backdrops: system materials behind the window (`El::system_material`), and blur of and glass
//! over the window's own content (`El::backdrop_blur`, `El::backdrop_lens`).
use crate::El;

impl El {
    /// Back this node with a system backdrop material: the platform blurs what is behind the
    /// window under the node's rect and corner radius (the largest of its corners), which follow
    /// layout and animation like any other paint. Native on macOS in transparent windows, where
    /// it replaces the fill and the node's shadows skip its interior; elsewhere, in opaque
    /// windows and in headless renders, the fill (or a neutral tint without one) draws instead.
    /// Native materials sit under everything the window draws and ignore ancestor clips.
    /// `Glass` also turns on `continuous_corners`, the only curve `NSGlassEffectView` draws, so
    /// the fill, clip and shadow hole match the glass; `Popover` follows `continuous_corners`.
    pub fn system_material(mut self, material: crate::SystemMaterial) -> Self {
        if material == crate::SystemMaterial::Glass { self.vis_flags.set(crate::el::VisualFlags::CONTINUOUS_CORNERS, true); }
        self.cold_mut().material = Some(material);
        self
    }

    /// Blurs what is already drawn behind this element in the same window, inside its rounded
    /// (or continuous) corners and clip, then draws the element's fill over the blur as a tint,
    /// with its border and children on top: `el().backdrop_blur(8.0).fill(tint)` is a frosted
    /// bar. `sigma` is the Gaussian standard deviation in points, as in CSS
    /// `backdrop-filter: blur()`; 0 turns it off. Blurs past about 78 points at 2x stop growing.
    ///
    /// Unlike `El::system_material`, which shows what is behind the window, this blurs Rune's
    /// own content and looks the same on every platform. The element's opacity fades the blur
    /// with it, and its outer shadows leave the blurred interior clear. The first
    /// `MAX_BACKDROP_BLURS` blurred elements in a frame blur; any past that, and any drawn into a
    /// target that can't be copied (no `COPY_SRC`, or a caller's own `wgpu::RenderPass` via
    /// `Renderer::draw_range`), draw only their fill. Inside an ancestor drawn as one layer
    /// (`El::group_opacity`, `El::blur`, `El::cache`) the backdrop is what that layer has drawn
    /// so far, as in CSS.
    pub fn backdrop_blur(mut self, sigma: f32) -> Self {
        let s = if sigma.is_finite() { sigma.max(0.0) } else { 0.0 };
        if s != 0.0 || !self.vis.cold.is_empty() {
            self.vis.cold_mut().backdrop_blur = s;
        }
        self
    }
}

/// Glass for [`El::backdrop_lens`]: a clear slab resting on the element's backdrop, flat on top
/// with a rounded shoulder down to its rim. Light from straight above refracts at the top (Snell's
/// law, Fresnel reflection), so the flat middle shows the backdrop as it is and the shoulder pulls
/// it in from beyond the rim. The index of refraction varies with wavelength (Cauchy), so edges
/// between light and dark fringe in colour, and the steep shoulder reflects a soft light above or
/// the backdrop beside the glass. Sizes are in points, at most 1000.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lens {
    thickness: f32,
    bevel: f32,
    gap: f32,
    ior: f32,
    dispersion: f32,
    light: f32,
}

impl Default for Lens {
    fn default() -> Self { Lens::new() }
}

/// `v` if it is finite and at least `min`, else `otherwise`.
const fn finite(v: f32, min: f32, otherwise: f32) -> f32 { if v.is_finite() { v.max(min) } else { otherwise } }
/// Most a size may be, in points, so it stays finite in device pixels and in the shader.
const MAX_PT: f32 = 1000.0;

impl Lens {
    /// Crown glass: 8 points thick with an 8-point shoulder, 4 points above the backdrop, index
    /// 1.5, dispersion 0.03.
    pub const fn new() -> Lens { Lens { thickness: 8.0, bevel: 8.0, gap: 4.0, ior: 1.5, dispersion: 0.03, light: 1.0 } }
    /// Height the flat top rises over the rim.
    pub const fn thickness(mut self, pt: f32) -> Lens { self.thickness = finite(pt, 0.0, 8.0).min(MAX_PT); self }
    /// Width of the rounded shoulder from the rim to the flat top.
    pub const fn bevel(mut self, pt: f32) -> Lens { self.bevel = finite(pt, 0.5, 8.0).min(MAX_PT); self }
    /// Height of the rim above the backdrop: how far in the shoulder pulls the backdrop.
    pub const fn gap(mut self, pt: f32) -> Lens { self.gap = finite(pt, 0.0, 4.0).min(MAX_PT); self }
    /// Index of refraction at 550 nm; 1 bends nothing.
    pub const fn ior(mut self, n: f32) -> Lens { self.ior = finite(n, 1.0, 1.5); self }
    /// n(400 nm) − n(700 nm). Real glass is positive (blue bends most); a negative value swaps the
    /// fringe's order, red bending most.
    pub const fn dispersion(mut self, d: f32) -> Lens { self.dispersion = if d.is_finite() { d.clamp(-0.5, 0.5) } else { 0.03 }; self }
    /// Brightness of the light the rim reflects; 0 is none.
    pub const fn light(mut self, k: f32) -> Lens { self.light = finite(k, 0.0, 1.0); self }

    /// The record's `uv`: thickness, bevel and gap in device pixels, and the index.
    pub(crate) fn uv(&self, k_px: f32) -> [f32; 4] { [self.thickness * k_px, self.bevel * k_px, self.gap * k_px, self.ior] }
    /// The record's `border`: dispersion and light.
    pub(crate) fn border(&self) -> [f32; 4] { [self.dispersion, self.light, 0.0, 0.0] }
}

impl El {
    /// Draws what is already behind this element through glass ([`Lens`]) inside its rounded (or
    /// continuous) corners and clip, with its fill over the glass as a tint and its border and
    /// children on top. With `El::backdrop_blur` it is frosted glass: the flat top shows the blur
    /// and the shoulder fades from the sharp backdrop at the rim to the blur, so the edges it
    /// bends in stay legible. Its outer shadows leave the glass clear, as for a blur.
    ///
    /// The first `MAX_BACKDROP_LENSES` lensed elements in a frame draw glass, apart from the
    /// `MAX_BACKDROP_BLURS` blurs. It falls back where a blur does: past that many, or drawn into a
    /// target that can't be copied, it draws only its fill. The CPU painter draws its blur, if it
    /// has one, without the glass.
    pub fn backdrop_lens(mut self, lens: Lens) -> Self {
        self.vis.cold_mut().backdrop_lens = Some(lens);
        self
    }
}
