//! The arithmetic behind the browser shell: the canvas's size in device pixels, when a resize
//! needs the surface configured again, and the band a page's canvas covers. Kept out of
//! `platform/web.rs`, which only builds on `wasm32`, so the tests run headless.
#![cfg_attr(not(test), allow(dead_code))]

/// The canvas's size in device pixels. `css` is the canvas's CSS box and `reported` what winit
/// reported for it. A canvas reports pixels, but an emulated device scale reports CSS pixels, so
/// where a side disagrees with the canvas's own box the box wins. A window that fits its content
/// (`fit_height`) keeps the reported height: the app measures its content and sets the height
/// itself, so the box is only as tall as the last frame's content.
pub(crate) fn fix_canvas_size(css: (f32, f32), reported: (u32, u32), spec: (f32, f32), scale: f32, fit_height: bool) -> (u32, u32) {
    let scale = scale.max(1e-3);
    let want = |css: f32| (css * scale).round().max(1.0) as u32;
    // Within a pixel of the box the browser's own number is the box's: it knows the device
    // grid, and a fractional CSS box rounds by a fraction of a pixel either way.
    let fixed = |got: u32, css: f32| {
        if css > 0.0 && (got as f32 - want(css) as f32).abs() > scale.ceil() + 1.0 { want(css) } else { got }
    };
    let (w, h) = css;
    let mut size = reported;
    if w > 0.0 { size.0 = fixed(size.0, w); }
    if !fit_height && h > 0.0 { size.1 = fixed(size.1, h); }
    if size.0 == 0 { size.0 = if w > 0.0 { want(w) } else { want(spec.0) }; }
    if size.1 == 0 { size.1 = if !fit_height && h > 0.0 { want(h) } else { want(spec.1) }; }
    size
}

/// The surface's next size and whether it needs configuring again. `current` is the size it is
/// configured at. A page (`document`) never shrinks its surface: a phone's toolbar expanding
/// shrinks the viewport for a moment mid-scroll, and a web surface configured again destroys and
/// reallocates its swapchain textures in the GPU process right then.
pub(crate) fn should_reconfigure_surface(current: (u32, u32), next: (u32, u32), document: bool) -> ((u32, u32), bool) {
    if next.0 == 0 || next.1 == 0 || next == current { return (current, false); }
    if document && next.0 == current.0 && next.1 < current.1 { return (current, false); }
    (next, true)
}

/// The format the surface is configured with and the format the renderer draws in, for a
/// runtime that blends in sRGB encoding (`srgb_blend`) or not. `configured` is the surface's
/// current format, `offered` what it supports, and `views` whether it takes view formats (WebGPU
/// does, WebGL 2 does not). The wanted variant of `configured` is used when the surface offers it,
/// or as a view of it; otherwise the surface keeps `configured`, and the caller must tell the
/// runtime so (`!draws.is_srgb()`), or its colours are encoded twice.
pub(crate) fn blend_target(offered: &[wgpu::TextureFormat], configured: wgpu::TextureFormat, srgb_blend: bool, views: bool)
    -> (wgpu::TextureFormat, wgpu::TextureFormat) {
    let wanted = if srgb_blend { configured.remove_srgb_suffix() } else { configured.add_srgb_suffix() };
    if wanted == configured { (configured, configured) }
    else if offered.contains(&wanted) { (wanted, wanted) }
    else if views { (configured, wanted) }
    else { (configured, configured) }
}

#[cfg(test)]
#[path = "web_math_tests.rs"]
mod tests;
