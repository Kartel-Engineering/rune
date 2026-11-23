//! A canvas that can't be copied from (WebGL 2's: wgpu offers it only as a render target) draws
//! backdrop effects through a texture of the renderer's own (`Renderer::draw_relayed`).
use rune::render::{Gpu, OFFSCREEN_FORMAT, Offscreen, Renderer};
use rune::{Color, Cx, El, Env, Lens, Runtime, el};

const W: u32 = 200;
const H: u32 = 120;
const SPLIT: f32 = 63.0;

#[derive(Clone, Copy, Debug)]
enum Look { Prism, Frosted, Slab, Blur, Plain }

/// White over black, a pill across the line drawn as `look`.
fn scene(look: &Look, _: &mut Cx<Look>) -> El {
    let pill = el().absolute(40.0, 20.0).size(120.0, 80.0).radius(40.0);
    let pill = match look {
        Look::Prism => pill.backdrop_lens(Lens::prism()),
        Look::Frosted => pill.backdrop_lens(Lens::prism()).backdrop_blur(6.0),
        Look::Slab => pill.backdrop_lens(Lens::new().thickness(24.0).bevel(9.0)),
        Look::Blur => pill.backdrop_blur(8.0),
        Look::Plain => pill.fill(Color::WHITE.alpha(0.5)),
    };
    el().size(W as f32, H as f32).fill(Color::WHITE)
        .child(el().absolute(0.0, SPLIT).size(W as f32, H as f32 - SPLIT).fill(Color::BLACK))
        .child(pill)
}

/// `look` drawn into a target with `usage` besides its render use, through `draw_relayed` when
/// `relayed`, read back by drawing it onto a copyable texture. Also whether it took the relay.
fn draw(look: Look, srgb: bool, usage: wgpu::TextureUsages, relayed: bool) -> (bool, Vec<u8>) {
    let gpu = Gpu::headless();
    draw_on(&gpu, &mut None, look, srgb, usage, relayed, (W, H))
}

/// `draw` at `(w, h)` with `renderer`, made on `gpu` if there is none yet.
fn draw_on(gpu: &Gpu, renderer: &mut Option<Renderer>, look: Look, srgb: bool, usage: wgpu::TextureUsages, relayed: bool, (w, h): (u32, u32)) -> (bool, Vec<u8>) {
    let (device, queue) = (&gpu.device, &gpu.queue);
    let mut rt = Runtime::new(look, scene, (w as f32, h as f32), Env::new(false, 1.0));
    rt.set_srgb_blend(srgb);
    rt.frame(0.0);
    // A WebGL 2 canvas has no sRGB view: one that blends in sRGB encoding draws into the plain format.
    let format = if srgb { OFFSCREEN_FORMAT.remove_srgb_suffix() } else { OFFSCREEN_FORMAT };
    let renderer = renderer.get_or_insert_with(|| Renderer::new(device, format));
    renderer.set_srgb_blend(srgb);
    renderer.prepare(device, queue, &mut rt.text.atlas, &rt.instances, (w, h));
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("canvas"), size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | usage, view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let read = Offscreen::with_format(device, w, h, format);
    let mut encoder = device.create_command_encoder(&Default::default());
    let took = relayed && renderer.draw_relayed(device, &mut encoder, &view, [0.0, 0.0, 0.0, 1.0]);
    if !relayed { renderer.draw(&mut encoder, &view, [0.0, 0.0, 0.0, 1.0]); }
    wgpu::util::TextureBlitter::new(device, format).copy(device, &mut encoder, &view, &read.view);
    read.copy(&mut encoder);
    queue.submit([encoder.finish()]);
    (took, read.read(device))
}

fn at(px: &[u8], x: u32, y: u32) -> [u8; 3] { let i = ((y * W + x) * 4) as usize; [px[i], px[i + 1], px[i + 2]] }

fn max_diff(a: &[u8], b: &[u8]) -> u8 { a.iter().zip(b).map(|(a, b)| a.abs_diff(*b)).max().unwrap_or(0) }

/// A canvas that can't be copied from (WebGL 2's) draws every backdrop effect through the
/// renderer's own texture as a canvas that can draws it, where a plain draw gives only the fill.
#[test]
fn a_canvas_that_cannot_be_copied_draws_backdrop_effects_through_the_relay() {
    let none = wgpu::TextureUsages::empty();
    for srgb in [false, true] {
        for look in [Look::Prism, Look::Frosted, Look::Slab, Look::Blur] {
            let (_, copied) = draw(look, srgb, wgpu::TextureUsages::COPY_SRC, false);
            let (took, relayed) = draw(look, srgb, none, true);
            let (_, fill) = draw(look, srgb, none, false);
            assert!(took, "{look:?}: drawn through the relay");
            let diff = max_diff(&copied, &relayed);
            assert!(diff <= 1, "{look:?} srgb {srgb}: the relay draws what a copyable canvas does, max {diff} levels apart");
            let changed = copied.chunks(4).zip(fill.chunks(4)).filter(|(a, b)| max_diff(a, b) > 8).count();
            assert!(changed > 500, "{look:?} srgb {srgb}: without the relay only the fill draws ({changed} pixels differ)");
        }
        // The prism's bands: the bottom rim pulls the white above the line over the black, red farthest.
        let (_, px) = draw(Look::Prism, srgb, none, true);
        let red = (SPLIT as u32 + 2..100).filter(|&y| { let c = at(&px, 100, y); c[0] > 150 && c[2] < 60 }).count();
        assert!(red >= 2, "srgb {srgb}: red rows under the bottom rim: {red}");
        // Its ring, just below the element over black: a dim blue-grey, not black.
        let ring = at(&px, 100, 100);
        assert!(ring.iter().all(|&v| (20..140).contains(&v)), "srgb {srgb}: the ring below the bottom edge: {ring:?}");
    }
}

/// A frame with no backdrop effect, and a canvas that can be copied, draw straight into the
/// canvas: no texture of the renderer's, no extra pass.
#[test]
fn only_a_frame_with_a_backdrop_effect_on_a_canvas_that_cannot_be_copied_takes_the_relay() {
    let (plain, relayed) = draw(Look::Plain, true, wgpu::TextureUsages::empty(), true);
    let (_, direct) = draw(Look::Plain, true, wgpu::TextureUsages::empty(), false);
    assert!(!plain, "no backdrop effect, no relay");
    assert_eq!(relayed, direct, "and the same pixels");
    let (copyable, _) = draw(Look::Prism, true, wgpu::TextureUsages::COPY_SRC, true);
    assert!(!copyable, "a canvas that can be copied needs no relay");
}

/// One renderer's relay follows the canvas to a new size: the second frame draws as a fresh
/// renderer draws it.
#[test]
fn the_relay_follows_the_canvas_to_a_new_size() {
    let (gpu, none) = (Gpu::headless(), wgpu::TextureUsages::empty());
    let mut renderer = None;
    draw_on(&gpu, &mut renderer, Look::Prism, true, none, true, (W, H));
    let (took, resized) = draw_on(&gpu, &mut renderer, Look::Prism, true, none, true, (160, 100));
    let (_, fresh) = draw_on(&gpu, &mut None, Look::Prism, true, none, true, (160, 100));
    assert!(took, "drawn through the relay");
    let diff = max_diff(&resized, &fresh);
    assert!(diff <= 1, "the resized frame draws as a fresh renderer's, max {diff} levels apart");
}
