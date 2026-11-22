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
    draw_in(state, view, scale, false)
}

/// `draw`, blending in sRGB encoding when `srgb` (as a browser canvas does) or in linear light.
fn draw_in<S: 'static>(state: S, view: fn(&S, &mut Cx<S>) -> El, scale: f32, srgb: bool) -> (Runtime<S>, Vec<u8>) {
    let gpu = Gpu::headless();
    let mut rt = Runtime::new(state, view, (W, H), Env::new(false, scale));
    rt.set_srgb_blend(srgb);
    rt.frame(0.0);
    let size = ((W * scale) as u32, (H * scale) as u32);
    let format = if srgb { OFFSCREEN_FORMAT.remove_srgb_suffix() } else { OFFSCREEN_FORMAT };
    let mut renderer = Renderer::new(&gpu.device, format);
    renderer.set_srgb_blend(srgb);
    let target = Offscreen::with_format(&gpu.device, size.0, size.1, format);
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

/// A frame that blends in sRGB encoding (a browser canvas) holds encoded colours. The glass
/// bends and dims light, not the encoding, so it draws what it draws in a linear frame.
#[test]
fn glass_looks_the_same_when_the_frame_blends_in_srgb() {
    let (_, linear) = draw_in(Look::Lens, split, 1.0, false);
    let (_, srgb) = draw_in(Look::Lens, split, 1.0, true);
    for (x, y) in [(100, 30), (100, 50), (100, 66), (100, 86), (100, 97), (45, 60), (155, 60)] {
        let (a, b) = (at(&linear, x, y), at(&srgb, x, y));
        assert!(a.iter().zip(b).all(|(p, q)| p.abs_diff(q) <= 3), "at {x},{y}: linear {a:?}, sRGB {b:?}");
    }
}

/// Fine stripes (1 point dark in every 3) that scroll down `.0` points under a still pill with a
/// steep, high shoulder, as text scrolls under a docked bar. `.1`: the stripes alone.
fn stripes(s: &(u32, bool), _: &mut Cx<(u32, bool)>) -> El {
    let pill = el().absolute(40.0, 45.0).size(120.0, 30.0).radius(15.0);
    let pill = if s.1 { pill } else { pill.backdrop_lens(Lens::new().thickness(9.0).bevel(3.5).gap(16.0).dispersion(-0.3)) };
    el().size(W, H).fill(Color::WHITE)
        .children((0..H as u32 / 3 + 1).map(|k| el().absolute(0.0, (k * 3 + s.0) as f32 - 3.0).size(W, 1.0).fill(Color::BLACK)))
        .child(pill)
}

/// Mean change per channel between two frames over the rows `ys`, columns 60..140 (the pill's
/// straight run).
fn change(a: &[u8], b: &[u8], ys: std::ops::Range<u32>) -> f32 {
    let (mut sum, mut n) = (0u32, 0u32);
    for y in ys {
        for x in 60..140 {
            let (p, q) = (at(a, x, y), at(b, x, y));
            sum += (0..3).map(|c| p[c].abs_diff(q[c]) as u32).sum::<u32>();
            n += 3;
        }
    }
    sum as f32 / n as f32
}

/// The shoulder squeezes tens of backdrop pixels into each of its few: it reads them prefiltered
/// at that footprint, so stripes that scroll under a still lens change its rim far less than they
/// change the page beside it, instead of flickering.
#[test]
fn text_scrolling_under_the_rim_does_not_flicker() {
    let frames: Vec<_> = (0..3).map(|k| draw((k, false), stripes, 1.0).1).collect();
    let pages: Vec<_> = (0..3).map(|k| draw((k, true), stripes, 1.0).1).collect();
    // The rim band: the outer 3 points of the top and bottom shoulders, where they squeeze. Their
    // inner half point is nearly flat and moves with the page, as it should.
    let rim = |a: &[u8], b: &[u8]| (change(a, b, 45..48) + change(a, b, 72..75)) / 2.0;
    let rim = (rim(&frames[0], &frames[1]) + rim(&frames[1], &frames[2])) / 2.0;
    let page = (change(&pages[0], &pages[1], 45..75) + change(&pages[1], &pages[2], 45..75)) / 2.0;
    eprintln!("rim {rim:.1}, page {page:.1}, ratio {:.3}", rim / page);
    assert!(page > 100.0, "the stripes move: {page}");
    // Unfiltered, the rim changes about 5 % of what the page does; prefiltered, about 1 %.
    assert!(rim < page * 0.025, "the rim changes {rim:.1} a step, the page beside it {page:.1}");
}

/// A lens, a frosted lens and a plain fill, for the motion gate's paint probe.
fn probed(_: &(), _: &mut Cx<()>) -> El {
    el().size(W, H)
        .child(el().key("lens").size(40.0, 20.0).backdrop_lens(Lens::new()))
        .child(el().key("frosted").size(40.0, 20.0).backdrop_lens(Lens::new()).backdrop_blur(6.0))
        .child(el().key("plain").size(40.0, 20.0).fill(Color::WHITE))
}

#[test]
fn the_motion_gate_counts_a_lens_as_backdrop_paint() {
    use rune::motion_gate::check::paint::S_BACKDROP;
    let mut run = rune::motion_gate::Run::new(Runtime::new((), probed, (W, H), Env::new(false, 1.0)), 60.0);
    run.step_n(1);
    let last = run.rec.frames - 1;
    let backdrop = |key: &str| {
        let path = run.rt.elements().into_iter().find(|e| e.key.as_deref() == Some(key)).unwrap().path;
        run.rec.tracks.iter().find(|t| t.path == path).and_then(|t| t.paint_at(last)).unwrap().sizes[S_BACKDROP]
    };
    assert!(backdrop("lens") > 0.0, "a lens without a blur still paints its backdrop");
    assert_eq!(backdrop("frosted"), 6.0, "a frosted lens reports its blur");
    assert_eq!(backdrop("plain"), 0.0, "no lens, no blur");
}

/// The prism's split: a little below the pill's middle, so its lower half has black that the rim
/// does not reach.
const PRISM_SPLIT: f32 = 63.0;

/// White over black, a `Lens::prism()` pill across the line.
fn prism_split(_: &(), _: &mut Cx<()>) -> El {
    el().size(W, H).fill(Color::WHITE)
        .child(el().absolute(0.0, PRISM_SPLIT).size(W, H - PRISM_SPLIT).fill(Color::BLACK))
        .child(el().absolute(PILL[0], PILL[1]).size(PILL[2], PILL[3]).radius(40.0).backdrop_lens(Lens::prism()))
}

#[test]
fn a_prism_splits_a_light_edge_under_its_rim_into_red() {
    let (_, px) = draw((), prism_split, 1.0);
    // The bottom rim pulls the white from above the line down over the black, red farthest: a band
    // of red there, and black in the middle.
    let red = (PRISM_SPLIT as u32 + 2..(PILL[1] + PILL[3]) as u32).filter(|&y| { let c = at(&px, 100, y); c[0] > 150 && c[2] < 60 }).count();
    assert!(red >= 2, "red rows under the bottom rim: {red}");
    let mid = at(&px, 100, PRISM_SPLIT as u32 + 8);
    assert!(mid[0] < 10 && mid[1] < 10 && mid[2] < 10, "the middle stays black: {mid:?}");
}

#[test]
fn a_prism_rings_its_edge_just_outside_the_element() {
    let (_, px) = draw((), prism_split, 1.0);
    // The ring lies over the edge, past the element's rect: the row just below the pill, over black,
    // is a dim blue-grey, not black.
    let c = at(&px, (PILL[0] + PILL[2] / 2.0) as u32, (PILL[1] + PILL[3]) as u32);
    assert!(c.iter().take(3).all(|&v| (20..140).contains(&v)), "below the bottom edge: {c:?}");
}

/// A small `Lens::prism()` pill (radius 15) over white.
fn small_prism(_: &(), _: &mut Cx<()>) -> El {
    el().size(W, H).fill(Color::WHITE).child(el().absolute(40.0, 45.0).size(120.0, 30.0).radius(15.0).backdrop_lens(Lens::prism()))
}

#[test]
fn a_small_prism_over_white_is_light_inside() {
    let (_, px) = draw((), small_prism, 1.0);
    // Its bend scales with the radius, so it reads white from inside the element, not the
    // transparent black outside it.
    for (x, y) in [(100, 60), (60, 60), (140, 60), (100, 50), (100, 70), (48, 60)] {
        let c = at(&px, x, y);
        assert!(c.iter().take(3).all(|&v| v > 180), "inside at ({x}, {y}): {c:?}");
    }
}

/// A `Lens::prism()` pill with no height over white.
fn flat_prism(_: &(), _: &mut Cx<()>) -> El {
    el().size(W, H).fill(Color::WHITE).child(el().absolute(40.0, 60.0).size(120.0, 0.0).radius(15.0).backdrop_lens(Lens::prism()))
}

#[test]
fn a_prism_with_no_area_draws_nothing() {
    for scale in [1.0, 2.0] {
        let (_, px) = draw((), flat_prism, scale);
        assert!(px.chunks(4).all(|c| c[..3] == [255, 255, 255]), "at scale {scale}");
    }
}

/// `prism_split` with the prism tuned by `tune` and blurred by `sigma`.
fn tuned(s: &(fn(Lens) -> Lens, f32), _: &mut Cx<(fn(Lens) -> Lens, f32)>) -> El {
    el().size(W, H).fill(Color::WHITE)
        .child(el().absolute(0.0, PRISM_SPLIT).size(W, H - PRISM_SPLIT).fill(Color::BLACK))
        .child(el().absolute(PILL[0], PILL[1]).size(PILL[2], PILL[3]).radius(40.0).backdrop_lens((s.0)(Lens::prism())).backdrop_blur(s.1))
}

fn same(l: Lens) -> Lens { l }

#[test]
fn a_prism_with_no_split_has_no_colour_bands() {
    let (_, px) = draw((same as fn(Lens) -> Lens, 0.0), tuned, 1.0);
    let (_, flat) = draw(((|l: Lens| l.split(0.0)) as fn(Lens) -> Lens, 0.0), tuned, 1.0);
    let (_, unshaded) = draw(((|l: Lens| l.shade(0.0)) as fn(Lens) -> Lens, 0.0), tuned, 1.0);
    // Under the bottom rim, above the ring: the fitted prism splits the edge into colours, split
    // 0 leaves only the shading's slight warmth.
    let hue = |px: &[u8]| (PRISM_SPLIT as u32..(PILL[1] + PILL[3]) as u32 - 2)
        .map(|y| { let c = at(px, 100, y); c[..3].iter().max().unwrap() - c[..3].iter().min().unwrap() }).max().unwrap();
    assert!(hue(&px) > 100, "the fitted prism has bands: {}", hue(&px));
    assert!(hue(&unshaded) > 100, "unshaded, the bands stay: {}", hue(&unshaded));
    assert!(hue(&flat) < 20, "split 0 has none: {}", hue(&flat));
}

#[test]
fn a_prism_with_ring_zero_has_no_ring() {
    let (_, flat) = draw(((|l: Lens| l.ring(0.0)) as fn(Lens) -> Lens, 0.0), tuned, 1.0);
    // The row just below the pill lies over black: `a_prism_rings_its_edge_just_outside_the_element`
    // finds the ring there, ring 0 leaves it black.
    let c = at(&flat, (PILL[0] + PILL[2] / 2.0) as u32, (PILL[1] + PILL[3]) as u32);
    assert!(c.iter().take(3).all(|&v| v < 10), "below the bottom edge: {c:?}");
}

#[test]
fn a_frosted_prism_blurs_its_middle_and_keeps_its_ring() {
    let (_, clear) = draw((same as fn(Lens) -> Lens, 0.0), tuned, 1.0);
    let (_, frosted) = draw((same as fn(Lens) -> Lens, 8.0), tuned, 1.0);
    // The line between white and black, in the middle: sharp through clear glass, soft frosted.
    let step = |px: &[u8]| at(px, 100, PRISM_SPLIT as u32 - 2)[0] as i32 - at(px, 100, PRISM_SPLIT as u32 + 2)[0] as i32;
    assert!(step(&frosted) < step(&clear) - 60, "frosted {} vs clear {}", step(&frosted), step(&clear));
    let c = at(&frosted, (PILL[0] + PILL[2] / 2.0) as u32, (PILL[1] + PILL[3]) as u32);
    assert!(c.iter().take(3).all(|&v| (20..140).contains(&v)), "the ring below the bottom edge: {c:?}");
}
