use rune::render::KIND_BACKDROP_BLUR;
use rune::{Color, Cx, El, Env, Runtime, col, el};
use rune_kit::{Button, ButtonVariant};

use super::*;

const LOOKS: [Look; 4] = [
    Look { dark: false, opaque: false },
    Look { dark: true, opaque: false },
    Look { dark: false, opaque: true },
    Look { dark: true, opaque: true },
];

fn card(look: &Look, _: &mut Cx<Look>) -> El {
    el().size(200.0, 120.0).child(apply(el().size(120.0, 60.0).radius(18.6), &look.material(Pane::Card)))
}

#[test]
fn a_pane_blurs_its_backdrop_at_26_points_and_a_control_at_14() {
    for dark in [false, true] {
        let look = Look { dark, opaque: false };
        for pane in [Pane::Panel, Pane::Card, Pane::Popover, Pane::Sidebar] {
            assert_eq!(look.material(pane).blur, 26.0, "{pane:?}");
        }
        for pane in [Pane::Control, Pane::Primary, Pane::PrimaryHover, Pane::PrimaryPressed] {
            assert_eq!(look.material(pane).blur, 14.0, "{pane:?}");
        }
        let mut rt = Runtime::new(look, card, (200.0, 120.0), Env::new(dark, 2.0));
        rt.frame(0.0);
        let blurs: Vec<f32> =
            rt.instances.iter().filter(|i| i.params[1] == KIND_BACKDROP_BLUR).map(|i| i.params[0]).collect();
        assert_eq!(blurs, [52.0], "one blur, σ 26 pt in device pixels at 2x");
    }
}

#[test]
fn opaque_panes_drop_the_blur_turn_solid_and_keep_their_edges() {
    for dark in [false, true] {
        let (glass, opaque) = (Look { dark, opaque: false }, Look { dark, opaque: true });
        for pane in [Pane::Panel, Pane::Card, Pane::Popover, Pane::Sidebar, Pane::Primary] {
            let m = opaque.material(pane);
            assert_eq!((m.blur, m.fill.a), (0.0, 1.0), "{pane:?}");
            assert_eq!(m.layers.len(), glass.material(pane).layers.len(), "{pane:?} keeps its edge");
        }
        assert_eq!(opaque.material(Pane::Control).blur, 0.0);
        let mut rt = Runtime::new(opaque, card, (200.0, 120.0), Env::new(dark, 2.0));
        rt.frame(0.0);
        assert!(rt.instances.iter().all(|i| i.params[1] != KIND_BACKDROP_BLUR));
    }
}

#[test]
fn thicker_panes_are_more_opaque() {
    for dark in [false, true] {
        let a = |p| Look { dark, opaque: false }.material(p).fill.a;
        assert!(a(Pane::Panel) < a(Pane::Card) && a(Pane::Card) < a(Pane::Popover), "dark {dark}");
    }
}

#[test]
fn the_rim_paints_over_the_inner_line_and_the_shade_goes_under_everything() {
    for look in LOOKS {
        let t = look.tokens();
        let l = look.material(Pane::Card).layers;
        let rim = l.last().unwrap();
        assert!(rim.inset && rim.y == 1.0 && rim.color == t.glass_rim, "CSS's first layer paints last");
        let shade = l.first().unwrap();
        assert_eq!((shade.y, shade.blur, shade.spread, shade.inset), (12.0, 32.0, -14.0, false));
    }
}

#[test]
fn the_one_state_is_a_tint_a_rim_and_a_hairline_and_nothing_else() {
    for look in LOOKS {
        let t = look.tokens();
        let m = look.material(Pane::State);
        assert_eq!((m.fill, m.blur), (t.glass_state, 0.0));
        assert_eq!(m.layers, [ring(t.glass_state_edge), inset(1.0, 0.0, t.glass_state_rim)]);
        assert!(look.material(Pane::Armed).fill.a > m.fill.a, "armed is a second coat");
    }
}

#[test]
fn a_kit_submit_button_is_the_ink_pane() {
    fn view(dark: &bool, _: &mut Cx<bool>) -> El {
        col().child(Button::new("Send").variant(ButtonVariant::Submit).build(&kit(*dark)).key("send"))
    }
    for dark in [false, true] {
        let look = Look { dark, opaque: false };
        let k = kit(dark);
        assert_eq!(k.elevation.button_contrast, look.material(Pane::Primary).layers);
        assert_eq!(k.elevation.input(), look.material(Pane::Field).layers);
        let mut rt = Runtime::new(dark, view, (200.0, 80.0), Env::new(dark, 2.0));
        rt.frame(0.0);
        let send = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("send")).unwrap();
        assert_eq!(send.fill, look.material(Pane::Primary).fill);
        let ink = look.tokens().primary;
        assert_eq!(send.fill, ink.alpha(if dark { 0.86 } else { 0.88 }), "--primary at --primary-solid");
    }
}

#[test]
fn a_primary_border_is_the_ink_toned_toward_the_edge() {
    let t = Look::default().tokens();
    let edge = mix_oklab(t.primary, 0.55, t.primary_edge);
    // color-mix(in oklab, oklch(0.2 0 0) 55%, oklch(0.2 0 0 / 30%)): the same grey at 68.5%.
    assert!((edge.a - 0.685).abs() < 1e-4 && (edge.to_oklab()[0] - 0.2).abs() < 1e-4, "{edge:?}");
    assert_eq!(mix_oklab(Color::TRANSPARENT, 0.5, Color::TRANSPARENT), Color::TRANSPARENT);
}
