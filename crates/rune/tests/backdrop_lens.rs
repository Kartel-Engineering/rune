//! `El::backdrop_lens`: glass over what is drawn behind an element. Its flat middle shows the
//! backdrop as it is, its shoulder bends the backdrop inward from the rim, and where the GPU
//! can't take it the element draws its fill.
use rune::render::cpu_boot::{CpuPainter, Scene, Target};
use rune::render::{Gpu, KIND_BACKDROP_BLUR, OFFSCREEN_FORMAT, Offscreen, Renderer};
use rune::{Color, Cx, El, Env, Lens, Runtime, el};

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
}

/// White above `SPLIT`, black below, and the pill over the line.
fn split(look: &Look, _: &mut Cx<Look>) -> El {
    let pill = el().absolute(PILL[0], PILL[1]).size(PILL[2], PILL[3]).radius(40.0).fill(Color::TRANSPARENT);
    let pill = match look {
        Look::Lens => pill.backdrop_lens(Lens::new().thickness(24.0).bevel(9.0).gap(45.0).dispersion(-0.3)),
        Look::Blur => pill.backdrop_blur(8.0),
        Look::Plain => pill.fill(Color::WHITE.alpha(0.5)).backdrop_lens(Lens::new()),
    };
    el().size(W, H).fill(Color::WHITE)
        .child(el().absolute(0.0, SPLIT).size(W, H - SPLIT).fill(Color::BLACK))
        .child(pill)
}

fn gpu_pixels(look: Look) -> (Runtime<Look>, Vec<u8>) {
    let gpu = Gpu::headless();
    let mut rt = Runtime::new(look, split, (W, H), Env::new(false, 1.0));
    rt.frame(0.0);
    let size = (W as u32, H as u32);
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

fn at(px: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W as u32 + x) * 4) as usize;
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
