//! `El::backdrop_lens`: glass over what is drawn behind an element. Its flat middle shows the
//! backdrop as it is, its shoulder bends the backdrop inward from the rim, and where the GPU
//! can't take it the element draws its fill.
use rune::render::cpu_boot::{CpuPainter, Scene, Target};
use rune::render::{Gpu, KIND_BACKDROP_BLUR, KIND_LAYER, OFFSCREEN_FORMAT, Offscreen, Renderer};
use rune::{Color, Cx, El, Env, Lens, Runtime, el, text};

const W: f32 = 200.0;
const H: f32 = 120.0;
/// The glass: 120 x 80 at (40, 20), with fully rounded ends.
const PILL: [f32; 4] = [40.0, 20.0, 120.0, 80.0];
/// White above this line, black below: 24 points above the pill's bottom.
const SPLIT: f32 = 76.0;

#[derive(Clone, Copy, PartialEq)]
enum Look {
    Lens,
    Blur,
    Plain,
    /// A lens over a blur: frosted glass.
    Frosted,
}

/// The lens the split tests use.
const LENS: Lens = Lens::new().thickness(24.0).bevel(9.0).gap(45.0).dispersion(-0.3);

/// White above `SPLIT`, black below, and the pill over the line.
fn split(look: &Look, _: &mut Cx<Look>) -> El {
    let pill = el().absolute(PILL[0], PILL[1]).size(PILL[2], PILL[3]).radius(40.0).fill(Color::TRANSPARENT);
    let pill = match look {
        Look::Lens => pill.backdrop_lens(LENS),
        Look::Blur => pill.backdrop_blur(8.0),
        Look::Frosted => pill.backdrop_lens(LENS).backdrop_blur(8.0),
        Look::Plain => pill.fill(Color::WHITE.alpha(0.5)).backdrop_lens(Lens::new()),
    };
    el().size(W, H).fill(Color::WHITE)
        .child(el().absolute(0.0, SPLIT).size(W, H - SPLIT).fill(Color::BLACK))
        .child(pill)
}

/// `view` over `state` drawn by the GPU renderer at `scale`, into a target that can be copied.
fn draw<S: 'static>(state: S, view: fn(&S, &mut Cx<S>) -> El, scale: f32) -> (Runtime<S>, Vec<u8>) {
    let gpu = Gpu::headless();
    let mut rt = Runtime::new(state, view, (W, H), Env::new(false, scale));
    rt.frame(0.0);
    let size = ((W * scale) as u32, (H * scale) as u32);
    let mut renderer = Renderer::new(&gpu.device, OFFSCREEN_FORMAT);
    let target = Offscreen::new(&gpu.device, size.0, size.1);
    renderer.prepare(&gpu.device, &gpu.queue, &mut rt.text.atlas, &rt.instances, size);
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    renderer.draw(&mut encoder, &target.view, [0.0, 0.0, 0.0, 1.0]);
    target.copy(&mut encoder);
    gpu.queue.submit([encoder.finish()]);
    let px = target.read(&gpu.device);
    (rt, px)
}

fn gpu_pixels(look: Look) -> (Runtime<Look>, Vec<u8>) { draw(look, split, 1.0) }

fn at(px: &[u8], x: u32, y: u32) -> [u8; 4] { at_w(px, W as u32, x, y) }

fn at_w(px: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * width + x) * 4) as usize;
    [px[i], px[i + 1], px[i + 2], px[i + 3]]
}

#[test]
fn the_middle_shows_the_backdrop_and_the_rim_bends_it() {
    let (rt, lens) = gpu_pixels(Look::Lens);
    let (_, blur) = gpu_pixels(Look::Blur);
    assert_eq!(rt.instances.iter().filter(|i| i.params[1] == KIND_BACKDROP_BLUR).count(), 1, "one record");
    // The middle is flat glass: the backdrop undistorted, white above the line and black below,
    // a sharp edge where a blur would grey it.
    let (above, below) = (at(&lens, 100, 66)[0], at(&lens, 100, 86)[0]);
    assert!(above > 230 && below < 25, "middle shows the backdrop: {above} above, {below} below");
    let (blur_above, blur_below) = (at(&blur, 100, 74)[0], at(&blur, 100, 78)[0]);
    let (lens_above, lens_below) = (at(&lens, 100, 74)[0], at(&lens, 100, 78)[0]);
    assert!(lens_above as i32 - lens_below as i32 > blur_above as i32 - blur_below as i32 + 100,
        "the lens keeps the line sharp: lens {lens_above}/{lens_below}, blur {blur_above}/{blur_below}");
    // The shoulder pulls the backdrop in from the rim: just inside the bottom rim, over black, the
    // glass shows the white above the line, where a blur stays dark.
    let rim = (95..99).map(|y| at(&lens, 100, y)[0]).max().unwrap();
    let under = (95..99).map(|y| at(&blur, 100, y)[0]).max().unwrap();
    assert!(rim > 150 && under < 60, "the rim brings the white in: lens {rim}, blur {under}");
    // Outside the glass nothing changes.
    for (x, y) in [(20, 20), (180, 110), (100, 8), (100, 112)] {
        assert_eq!(at(&lens, x, y), at(&blur, x, y), "untouched at {x},{y}");
    }
}

#[test]
fn on_the_cpu_a_lens_draws_its_fill() {
    let mut rt = Runtime::new(Look::Plain, split, (W, H), Env::new(false, 1.0));
    rt.frame(0.0);
    let size = (W as u32, H as u32);
    let scene = Scene::new(&rt.instances, &rt.text.atlas, &rt.images, size, rt.clear_color());
    let mut buf = vec![0u8; (size.0 * size.1 * 4) as usize];
    CpuPainter::new(1).paint(&scene, &mut Target::new(&mut buf, size.0, size.1)).expect("the CPU paints it");
    // B, G, R, A. Half white over black below the line: mid grey. Over white: white.
    let px = |x: u32, y: u32| { let i = ((y * size.0 + x) * 4) as usize; [buf[i + 2], buf[i + 1], buf[i]] };
    let grey = px(100, 86);
    assert!(grey.iter().all(|&v| (120..=200).contains(&v)), "the fill over black: {grey:?}");
    assert_eq!(px(100, 50), [255, 255, 255], "the fill over white");
    assert_eq!(px(20, 110), [0, 0, 0], "black outside the pill");
}

/// `n` small lensed squares and `n` blurred ones in a row.
fn many(n: &usize, _: &mut Cx<usize>) -> El {
    el().size(W, H).fill(Color::WHITE).children((0..*n * 2).map(|k| {
        let e = el().absolute(k as f32 * 6.0, 10.0).size(5.0, 5.0);
        if k % 2 == 0 { e.backdrop_lens(Lens::new()) } else { e.backdrop_blur(4.0) }
    }))
}

#[test]
fn lenses_have_a_frame_limit_of_their_own() {
    let n = rune::MAX_BACKDROP_LENSES + 3;
    let mut rt = Runtime::new(n, many, (W, H), Env::new(false, 1.0));
    rt.frame(0.0);
    let records: Vec<_> = rt.instances.iter().filter(|i| i.params[1] == KIND_BACKDROP_BLUR).collect();
    let lenses = records.iter().filter(|i| i.uv[3] >= 1.0).count();
    assert_eq!(lenses, rune::MAX_BACKDROP_LENSES, "lenses past the limit draw only their fill");
    assert_eq!(records.len() - lenses, rune::MAX_BACKDROP_BLURS, "blurs keep their own limit");
}

#[test]
fn frosted_glass_blurs_the_middle_and_still_bends_the_rim() {
    let (rt, frosted) = gpu_pixels(Look::Frosted);
    let (_, lens) = gpu_pixels(Look::Lens);
    let record = rt.instances.iter().find(|i| i.params[1] == KIND_BACKDROP_BLUR).expect("one record");
    assert!(record.params[0] > 0.0 && record.uv[3] >= 1.0, "a lens over a blur");
    // The middle is frosted: the line between white and black is soft, where clear glass keeps it
    // sharp.
    let step = |px: &[u8]| at(px, 100, 74)[0] as i32 - at(px, 100, 78)[0] as i32;
    assert!(step(&frosted) < step(&lens) - 100, "frosted {} vs clear {}", step(&frosted), step(&lens));
    // The shoulder still pulls the white in over the black at the bottom rim.
    let rim = (95..99).map(|y| at(&frosted, 100, y)[0]).max().unwrap();
    assert!(rim > 150, "the rim brings the white in: {rim}");
}

#[test]
fn at_scale_two_the_glass_is_in_device_pixels() {
    let (rt, px) = draw(Look::Lens, split, 2.0);
    let (_, one) = gpu_pixels(Look::Lens);
    let record = rt.instances.iter().find(|i| i.params[1] == KIND_BACKDROP_BLUR).unwrap();
    assert_eq!(record.uv, [48.0, 18.0, 90.0, 1.5], "thickness, bevel and gap double");
    // The same picture at twice the size: the rim's white and the sharp middle land where they do
    // at scale 1.
    let rim = (190..198).map(|y| at_w(&px, 400, 200, y)[0]).max().unwrap();
    let rim1 = (95..99).map(|y| at(&one, 100, y)[0]).max().unwrap();
    assert!((rim as i32 - rim1 as i32).abs() < 40, "rim at 2x {rim}, at 1x {rim1}");
    assert!(at_w(&px, 400, 200, 132)[0] > 230 && at_w(&px, 400, 200, 172)[0] < 25, "the middle is sharp");
}

/// Half-transparent white over the split, inside a half-faded group: the group is a layer, so the
/// lens reads a premultiplied, half-transparent backdrop.
fn grouped(look: &Look, _: &mut Cx<Look>) -> El {
    let pill = el().absolute(PILL[0], PILL[1]).size(PILL[2], PILL[3]).radius(40.0);
    let pill = match look {
        Look::Lens => pill.backdrop_lens(Lens::new()),
        Look::Blur => pill.backdrop_blur(8.0),
        _ => pill,
    };
    el().size(W, H).fill(Color::BLACK)
        .child(el().absolute(0.0, 0.0).size(W, H).group_opacity(0.5, 0.5)
            .child(el().absolute(0.0, 0.0).size(W, H).fill(Color::WHITE.alpha(0.5)))
            .child(pill))
}

#[test]
fn inside_a_layer_the_glass_shows_the_layer_as_a_blur_does() {
    let (rt, lens) = draw(Look::Lens, grouped, 1.0);
    let (_, blur) = draw(Look::Blur, grouped, 1.0);
    let (_, none) = draw(Look::Plain, grouped, 1.0);
    assert!(rt.instances.iter().any(|i| i.params[1] == KIND_LAYER), "the group is a layer");
    // The flat top passes the layer's own pixels (less 4 % reflected), as a blur of an even field
    // does, so the middle is lighter than the layer alone, about as the blur's.
    let (l, b, n) = (at(&lens, 100, 60)[0] as i32, at(&blur, 100, 60)[0] as i32, at(&none, 100, 60)[0] as i32);
    assert!(l > n + 15 && (l - b).abs() <= 12, "lens {l}, blur {b}, layer alone {n}");
}

/// `n` squares in a row, each over its own black bar on white, lensed when `.1`.
fn squares(s: &(usize, bool), _: &mut Cx<(usize, bool)>) -> El {
    el().size(W, H).fill(Color::WHITE).children((0..s.0).flat_map(|k| {
        let x = 4.0 + k as f32 * 16.0;
        let glass = el().absolute(x, 30.0).size(12.0, 60.0).radius(6.0);
        [el().absolute(x + 4.0, 40.0).size(4.0, 40.0).fill(Color::BLACK),
         if s.1 { glass.backdrop_lens(Lens::new().thickness(4.0).bevel(4.0).gap(6.0)) } else { glass }]
    }))
}

#[test]
fn the_renderer_draws_glass_for_the_first_lenses_only() {
    let n = rune::MAX_BACKDROP_LENSES + 2;
    let (_, plain) = draw((n, false), squares, 1.0);
    let gpu = Gpu::headless();
    let mut rt = Runtime::new((n, true), squares, (W, H), Env::new(false, 1.0));
    rt.frame(0.0);
    // The build stops at its limit; records past it, as an app's own instances may carry, reach
    // the renderer's own limit.
    let lensed: Vec<_> = rt.instances.iter().filter(|i| i.params[1] == KIND_BACKDROP_BLUR).copied().collect();
    assert_eq!(lensed.len(), rune::MAX_BACKDROP_LENSES);
    for k in rune::MAX_BACKDROP_LENSES..n {
        let mut r = lensed[0];
        r.rect[0] = 4.0 + k as f32 * 16.0;
        rt.instances.push(r);
    }
    let size = (W as u32, H as u32);
    let mut renderer = Renderer::new(&gpu.device, OFFSCREEN_FORMAT);
    let target = Offscreen::new(&gpu.device, size.0, size.1);
    renderer.prepare(&gpu.device, &gpu.queue, &mut rt.text.atlas, &rt.instances, size);
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    renderer.draw(&mut encoder, &target.view, [0.0, 0.0, 0.0, 1.0]);
    target.copy(&mut encoder);
    gpu.queue.submit([encoder.finish()]);
    let px = target.read(&gpu.device);
    // Glass changes the pixels over its square; past the renderer's limit nothing does.
    let changed = |k: usize| {
        let x0 = (4 + k * 16) as u32;
        (x0..x0 + 12).flat_map(|x| (30..90).map(move |y| (x, y))).filter(|&(x, y)| at(&px, x, y) != at(&plain, x, y)).count()
    };
    for k in 0..rune::MAX_BACKDROP_LENSES { assert!(changed(k) > 20, "lens {k} draws glass: {} px", changed(k)); }
    for k in rune::MAX_BACKDROP_LENSES..n { assert_eq!(changed(k), 0, "lens {k} past the limit draws nothing"); }
}

#[test]
fn a_pass_that_cannot_be_copied_draws_only_the_fill() {
    let gpu = Gpu::headless();
    let mut rt = Runtime::new(Look::Plain, split, (W, H), Env::new(false, 1.0));
    rt.frame(0.0);
    let size = (W as u32, H as u32);
    let mut renderer = Renderer::new(&gpu.device, OFFSCREEN_FORMAT);
    let target = Offscreen::new(&gpu.device, size.0, size.1);
    renderer.prepare(&gpu.device, &gpu.queue, &mut rt.text.atlas, &rt.instances, size);
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    {
        // A pass the caller owns (`draw_range`) can't be split for a copy.
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view, depth_slice: None, resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
        });
        renderer.draw_range(&mut pass, OFFSCREEN_FORMAT, 0..rt.instances.len() as u32);
    }
    target.copy(&mut encoder);
    renderer.submit(&gpu.queue, encoder);
    let px = target.read(&gpu.device);
    // Half white over black below the line, white above: the fill alone, no glass, so the bottom
    // rim stays the fill's grey.
    let grey = at(&px, 100, 86)[0];
    assert!((120..=200).contains(&grey), "the fill over black: {grey}");
    assert_eq!(at(&px, 100, 50)[..3], [255, 255, 255], "the fill over white");
    let rim = at(&px, 100, 97)[0];
    assert!((rim as i32 - grey as i32).abs() <= 2, "no glass at the rim: {rim} vs {grey}");
}

/// A lens with sizes far past any screen.
fn huge(_: &(), _: &mut Cx<()>) -> El {
    el().size(W, H).child(el().size(50.0, 50.0).backdrop_lens(Lens::new().thickness(1e38).bevel(f32::MAX).gap(1e30)))
}

#[test]
fn sizes_too_large_stay_finite() {
    let mut rt = Runtime::new((), huge, (W, H), Env::new(false, 4.0));
    rt.frame(0.0);
    let record = rt.instances.iter().find(|i| i.params[1] == KIND_BACKDROP_BLUR).unwrap();
    assert!(record.uv.iter().all(|v| v.is_finite()), "finite: {:?}", record.uv);
    assert_eq!(record.uv[..3], [4000.0, 4000.0, 4000.0], "1000 points each at scale 4");
}

/// Reads `Cx::backdrop_copies`.
fn copies(_: &(), cx: &mut Cx<()>) -> El {
    el().child(text(if cx.backdrop_copies() { "copies" } else { "fill" }).key("label"))
}

#[test]
fn a_view_reads_whether_backdrop_effects_draw() {
    let label = |rt: &Runtime<()>| {
        let e = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("label")).unwrap();
        e.label.or(e.text).unwrap_or_default()
    };
    let mut rt = Runtime::new((), copies, (W, H), Env::new(false, 1.0));
    rt.frame(0.0);
    assert_eq!(label(&rt), "fill", "headless copies nothing");
    rt.set_on_gpu(true);
    rt.frame(1.0 / 60.0);
    assert_eq!(label(&rt), "fill", "a GPU surface that can't be copied (WebGL 2)");
    rt.set_backdrop_copies(true);
    rt.frame(1.0 / 60.0);
    assert_eq!(label(&rt), "copies", "a surface that can be copied");
    rt.set_backdrop_copies(false);
    rt.frame(1.0 / 60.0);
    assert_eq!(label(&rt), "fill", "its device was lost");
}
