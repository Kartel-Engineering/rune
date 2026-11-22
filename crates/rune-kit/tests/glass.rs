//! `glass` and `glass_draws`: liquid glass for a pill, and when it can draw.
use rune::render::KIND_BACKDROP_BLUR;
use rune::{Color, Cx, El, Env, Runtime, Shadow, el, text};

/// A 30-point pill: glass while it can draw, else a white fill.
fn pill(_: &(), cx: &mut Cx<()>) -> El {
    let p = el().key("pill").absolute(20.0, 20.0).size(120.0, 60.0);
    let p = if rune_kit::glass_draws(cx) { rune_kit::glass(p, 30.0) } else { p.radius(30.0).fill(Color::WHITE) };
    el().size(200.0, 120.0).fill(Color::WHITE).child(p.child(text("Label")))
}

fn lenses(rt: &Runtime<()>) -> Vec<[f32; 4]> {
    rt.instances.iter().filter(|i| i.params[1] == KIND_BACKDROP_BLUR).map(|i| i.border).collect()
}

#[test]
fn a_pill_is_prism_glass_with_one_close_shadow_scaled_to_its_radius() {
    let mut rt = Runtime::new((), pill, (200.0, 120.0), Env::new(false, 1.0));
    rt.set_on_gpu(true);
    rt.set_backdrop_copies(true);
    rt.frame(0.0);
    let k = 30.0 / 41.0;
    let want = [Shadow { x: 3.28 * k, y: 9.82 * k, blur: 33.0 * k, color: Color::BLACK.alpha(0.118), ..Default::default() }];
    assert_eq!(rt.presented_pose("pill").unwrap().shadows(), &want[..]);
    let lens = lenses(&rt);
    assert_eq!(lens.len(), 1, "one lens");
    assert_eq!(lens[0][2], 1.0, "it is a prism");
}

#[test]
fn glass_waits_for_a_gpu_that_copies_and_motion_that_is_not_reduced() {
    let mut rt = Runtime::new((), pill, (200.0, 120.0), Env::new(false, 1.0));
    rt.frame(0.0);
    assert!(lenses(&rt).is_empty(), "headless: no copies");
    rt.set_on_gpu(true);
    rt.frame(1.0 / 60.0);
    assert!(lenses(&rt).is_empty(), "a GPU surface that can't be copied");
    rt.set_backdrop_copies(true);
    rt.frame(2.0 / 60.0);
    assert_eq!(lenses(&rt).len(), 1, "a surface that can be copied");
    rt.set_system_reduced_motion(true);
    rt.frame(3.0 / 60.0);
    assert!(lenses(&rt).is_empty(), "reduced motion");
}
