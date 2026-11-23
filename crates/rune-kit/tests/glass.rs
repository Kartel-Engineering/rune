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
        r#"{"bend":1,"split":1,"shade":1,"lights":1,"ring":1,"frost":0,"tint":0,"shadow_x":3.28,"shadow_y":9.82,"shadow_blur":33,"shadow_spread":0,"shadow_alpha":0.118,"veil":0,"hairline":0}"#
    );
    let g = rune_kit::Glass { tint: 0.08, frost: 12.0, shadow_x: -1.23456, ..Default::default() };
    assert!(g.to_json().contains(r#""frost":12,"tint":0.08,"shadow_x":-1.235,"#), "{}", g.to_json());
}

#[test]
fn settings_that_are_not_numbers_draw_no_shadow_instead_of_a_broken_one() {
    fn broken(_: &(), _: &mut Cx<()>) -> El {
        let g = rune_kit::Glass { shadow_x: f32::NAN, shadow_y: f32::INFINITY, shadow_spread: f32::NEG_INFINITY, shadow_alpha: f32::NAN, ..rune_kit::Glass::DEFAULT };
        el().size(200.0, 120.0).child(rune_kit::glass_with(el().key("pill").absolute(20.0, 20.0).size(120.0, 60.0), 30.0, &g, Color::WHITE))
    }
    let mut rt = Runtime::new((), broken, (200.0, 120.0), Env::new(false, 1.0));
    rt.set_on_gpu(true);
    rt.set_backdrop_copies(true);
    rt.frame(0.0);
    let drawn = format!("{:?}", rt.instances);
    assert!(!drawn.contains("NaN") && !drawn.contains("inf"), "{drawn}");
}

#[test]
fn veil_and_hairline_are_off_by_default_and_draw_when_set() {
    fn frosted(_: &(), _: &mut Cx<()>) -> El {
        let g = rune_kit::Glass { veil: 0.14, hairline: 0.2, ..rune_kit::Glass::DEFAULT };
        el().size(200.0, 120.0).child(rune_kit::glass_with(el().key("pill").absolute(20.0, 20.0).size(120.0, 60.0), 30.0, &g, Color::TRANSPARENT))
    }
    let d = rune_kit::Glass::DEFAULT;
    assert_eq!((d.veil, d.hairline), (0.0, 0.0));
    assert!(d.to_json().ends_with(r#","veil":0,"hairline":0}"#), "{}", d.to_json());
    let mut rt = Runtime::new((), frosted, (200.0, 120.0), Env::new(false, 1.0));
    rt.set_on_gpu(true);
    rt.set_backdrop_copies(true);
    rt.frame(0.0);
    let line = |y| Shadow { y, color: Color::WHITE.alpha(0.2), inset: true, ..Default::default() };
    let shadows = rt.presented_pose("pill").unwrap().shadows().to_vec();
    assert_eq!(&shadows[1..], &[line(1.0), line(-1.0)], "a white line inside the top and one inside the bottom");
}

#[test]
fn the_veil_is_white_over_the_tint() {
    fn tinted(_: &(), _: &mut Cx<()>) -> El {
        let g = rune_kit::Glass { tint: 0.5, veil: 0.5, ..rune_kit::Glass::DEFAULT };
        el().size(200.0, 120.0).child(rune_kit::glass_with(el().key("pill").absolute(20.0, 20.0).size(120.0, 60.0), 30.0, &g, Color::BLACK))
    }
    let mut rt = Runtime::new((), tinted, (200.0, 120.0), Env::new(false, 1.0));
    rt.set_on_gpu(true);
    rt.set_backdrop_copies(true);
    rt.frame(0.0);
    // Black at 0.5, then white at 0.5 over it: alpha 0.75, and premultiplied white 0.5.
    let fill = rt.instances.iter().map(|i| i.fill).find(|f| (f[3] - 0.75).abs() < 1e-4).expect("one fill at alpha 0.75");
    assert!((fill[0] - 0.5).abs() < 1e-4 && fill[0] == fill[1] && fill[1] == fill[2], "{fill:?}");
}
