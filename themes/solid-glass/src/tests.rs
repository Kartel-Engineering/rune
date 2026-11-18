use rune::render::KIND_BACKDROP_BLUR;
use rune::{Color, Cx, El, Env, Runtime, col, el};
use rune_kit::{Button, ButtonVariant, State};

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
fn a_pane_blurs_its_backdrop_by_its_own_sigma_in_device_pixels() {
    for dark in [false, true] {
        let look = Look { dark, opaque: false };
        let mut rt = Runtime::new(look, card, (200.0, 120.0), Env::new(dark, 2.0));
        rt.frame(0.0);
        let blurs: Vec<f32> =
            rt.instances.iter().filter(|i| i.params[1] == KIND_BACKDROP_BLUR).map(|i| i.params[0]).collect();
        assert_eq!(blurs, [look.material(Pane::Card).blur * 2.0], "one blur, in device pixels at 2x");
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

/// `src` over `dst` as a browser blends: in sRGB, straight alpha, onto an opaque `dst`.
fn on_page(src: Color, dst: Color) -> [f32; 3] {
    use rune::color::srgb_encode;
    [(src.r, dst.r), (src.g, dst.g), (src.b, dst.b)]
        .map(|(s, d)| srgb_encode(s) * src.a + srgb_encode(d) * (1.0 - src.a))
}

fn close(a: [f32; 3], b: [f32; 3]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() * 255.0 <= 1.0)
}

#[test]
fn armed_is_a_second_coat_of_the_state() {
    for look in LOOKS {
        let (state, page) = (look.material(Pane::State).fill, look.tokens().background);
        let [r, g, b] = on_page(state, page).map(rune::color::srgb_decode);
        let twice = on_page(state, Color { r, g, b, a: 1.0 });
        assert!(close(on_page(look.material(Pane::Armed).fill, page), twice), "{look:?}");
    }
}

#[test]
fn a_kit_submit_button_is_the_ink_pane_and_its_hover_the_hover_pane() {
    fn view(s: &(bool, State), _: &mut Cx<(bool, State)>) -> El {
        col().child(Button::new("Send").variant(ButtonVariant::Submit).state(s.1).build(&kit(s.0)).key("send"))
    }
    for dark in [false, true] {
        let look = Look { dark, opaque: false };
        let page = look.tokens().background;
        for (state, pane) in [(State::Rest, Pane::Primary), (State::Hover, Pane::PrimaryHover)] {
            let mut rt = Runtime::new((dark, state), view, (200.0, 80.0), Env::new(dark, 2.0));
            rt.set_srgb_blend(true);
            rt.frame(0.0);
            let send = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("send")).unwrap();
            assert!(close(on_page(send.fill, page), on_page(look.material(pane).fill, page)), "dark {dark}, {state:?}");
        }
    }
}

/// The tokens `kit` sets reach the components that read them: a kit field is the field pane, an
/// off switch track the groove, a plain button the glass control, and text the stylesheet's ink.
#[test]
fn kit_components_wear_the_panes_they_stand_for() {
    use rune::TextEdit;
    use rune_kit::{Switch, TextInput};
    fn view(dark: &bool, _: &mut Cx<bool>) -> El {
        let k = kit(*dark);
        col()
            .child(TextInput::new(&TextEdit::default()).placeholder("Name").build(&k).key("field").w(160.0))
            .child(Switch::new(false).build(&k))
            .child(Button::new("Cancel").build(&k).key("cancel"))
    }
    for dark in [false, true] {
        let look = Look { dark, opaque: false };
        let mut rt = Runtime::new(dark, view, (200.0, 120.0), Env::new(dark, 2.0));
        rt.frame(0.0);
        let els = rt.elements();
        let by_key = |key: &str| els.iter().find(|e| e.key.as_deref() == Some(key)).unwrap();
        for (key, pane) in [("field", Pane::Field), ("control", Pane::Groove), ("cancel", Pane::Control)] {
            assert_eq!(by_key(key).fill, look.material(pane).fill, "dark {dark}: {key}");
        }
        let label = els.iter().find(|e| e.text.as_deref() == Some("Cancel")).unwrap();
        assert_eq!(label.text_color, Some(look.tokens().foreground), "dark {dark}");
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

/// A kit switch's thumb shows the knob's own face, with no sheen over it: CSS paints the thumb a
/// flat `--primary-foreground` (dark, on) or `--background`.
#[test]
fn a_kit_switch_thumb_is_the_knob_face() {
    use rune::color::srgb_encode;
    use rune::render::{Gpu, OFFSCREEN_FORMAT, Offscreen, Renderer};
    use rune_kit::Switch;
    fn view(on: &(bool, bool), _: &mut Cx<(bool, bool)>) -> El {
        let k = kit(on.0);
        col().w_full().h_full().fill(k.c.page).padding(10.0).child(Switch::new(on.1).build(&k))
    }
    let (w, h) = (80u32, 40u32);
    let gpu = Gpu::headless();
    let mut off = Vec::new();
    for dark in [false, true] {
        for on in [false, true] {
            let mut rt = Runtime::new((dark, on), view, (w as f32, h as f32), Env::new(dark, 1.0));
            rt.set_srgb_blend(true);
            rt.frame(0.0);
            rt.settle_all();
            rt.frame(0.0);
            let thumb = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("thumb")).unwrap().rect;
            let format = OFFSCREEN_FORMAT.remove_srgb_suffix();
            let mut renderer = Renderer::new(&gpu.device, format);
            renderer.set_srgb_blend(true);
            let target = Offscreen::with_format(&gpu.device, w, h, format);
            renderer.prepare(&gpu.device, &gpu.queue, &mut rt.text.atlas, &rt.instances, (w, h));
            let mut encoder = gpu.device.create_command_encoder(&Default::default());
            renderer.draw(&mut encoder, &target.view, rt.clear_color());
            target.copy(&mut encoder);
            renderer.submit(&gpu.queue, encoder);
            let px = target.read(&gpu.device);
            let (x, y) = ((thumb[0] + thumb[2] / 2.0) as u32, (thumb[1] + thumb[3] / 2.0) as u32);
            let got = px[((y * w + x) * 4) as usize] as f32;
            let k = kit(dark);
            let face = (srgb_encode(if on { k.c.knob_on } else { k.c.knob }.r) * 255.0).round();
            if (got - face).abs() > 3.0 {
                off.push(format!("dark {dark}, on {on}: thumb centre {got}, knob face {face}"));
            }
        }
    }
    assert!(off.is_empty(), "{off:#?}");
}
