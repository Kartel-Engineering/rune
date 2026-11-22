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

/// The pill built by `glass_with` and the default settings, else as `pill`.
fn pill_with(_: &(), cx: &mut Cx<()>) -> El {
    let p = el().key("pill").absolute(20.0, 20.0).size(120.0, 60.0);
    let p = if rune_kit::glass_draws(cx) { rune_kit::glass_with(p, 30.0, &rune_kit::Glass::default(), Color::WHITE) } else { p };
    el().size(200.0, 120.0).fill(Color::WHITE).child(p.child(text("Label")))
}

#[test]
fn default_settings_build_the_same_glass() {
    let frame = |view: fn(&(), &mut Cx<()>) -> El| {
        let mut rt = Runtime::new((), view, (200.0, 120.0), Env::new(false, 1.0));
        rt.set_on_gpu(true);
        rt.set_backdrop_copies(true);
        rt.frame(0.0);
        format!("{:?}", rt.instances)
    };
    assert_eq!(frame(pill_with), frame(pill));
}

#[test]
fn the_default_settings_as_json() {
    assert_eq!(
        rune_kit::Glass::default().to_json(),
        r#"{"bend":1,"split":1,"shade":1,"lights":1,"ring":1,"frost":0,"tint":0,"shadow_x":3.28,"shadow_y":9.82,"shadow_blur":33,"shadow_spread":0,"shadow_alpha":0.118}"#
    );
    let g = rune_kit::Glass { tint: 0.08, frost: 12.0, shadow_x: -1.23456, ..Default::default() };
    assert!(g.to_json().contains(r#""frost":12,"tint":0.08,"shadow_x":-1.235,"#), "{}", g.to_json());
}
