//! Solid Glass in Rune: the three thicknesses over lines of text, a list in the one state,
//! primary and glass buttons, fields at rest and in focus, switches, a menu, and a kit row
//! themed by `rune_theme_solid_glass::kit`.
//!
//! ```sh
//! cargo run -p solid-glass -- --snap glass-dark.png --scale 2 --theme dark
//! cargo run -p solid-glass -- --snap glass-light.png --scale 2 --theme light
//! cargo run -p solid-glass -- --snap glass-opaque.png --scale 2 --theme dark --scene opaque
//! ```
//!
//! Every pane sits at fixed points, so a page drawn from `solid-glass.css` with the same
//! layout can be compared pixel for pixel.
//!
//! A frame blurs at most `rune::MAX_BACKDROP_BLURS` (4) panes. The gallery spends them on the
//! panel, the card, the popover and the list, and asks no blur of the rest: the menu is the
//! popover's material again (96% fill, the page shows through it by 5 of 255), and the buttons
//! sit on the bare page, where a blur changes nothing.

use rune::{AppSpec, Color, Cx, El, TextEdit, TextStyle, col, el, row, text};
use rune_kit::{Button, ButtonVariant, State, Switch, TextInput};
use rune_theme_solid_glass::{Look, Material, Pane, apply, kit};

#[derive(Default)]
struct Gallery {
    opaque: bool,
    edit: TextEdit,
}

const SIZE: (f32, f32) = (720.0, 470.0);
const LINE: &str =
    "solid glass · solid glass · solid glass · solid glass · solid glass · solid glass · ";

fn label(s: &str, c: Color) -> El {
    text(s).font(TextStyle::new(13.0, 500.0).lh(16.0)).color(c)
}

/// A pane at `x, y` in the page, `w` by `h`, with its corners.
fn at(e: El, [x, y, w, h]: [f32; 4], r: f32) -> El {
    e.absolute(x, y).size(w, h).radius(r)
}

/// `n` lines of text under the panes, kept clear of their edges so an edge can be measured.
fn page_text(look: Look, y: f32, n: usize) -> El {
    let ink = look.tokens().foreground;
    let line = |i: usize| {
        let style = TextStyle::new(13.0, 400.0).lh(17.5);
        text(LINE.repeat(2))
            .single_line()
            .font(style)
            .color(ink.alpha(0.55))
            .key(format!("line{i}"))
    };
    col()
        .absolute(0.0, y)
        .w(SIZE.0)
        .clip_rounded(0.0)
        .children((0..n).map(line))
}

fn view(s: &Gallery, cx: &mut Cx<Gallery>) -> El {
    let look = Look {
        dark: cx.system_dark(),
        opaque: s.opaque,
    };
    let t = look.tokens();
    let m = |p: Pane| look.material(p);
    // Past the frame's four blurs (see the module docs).
    let unblurred = |p: Pane| Material { blur: 0.0, ..m(p) };
    let k = kit(look.dark);
    let pane = |p: Pane, rect: [f32; 4], r: f32, name: &str| {
        at(apply(el(), &m(p)), rect, r)
            .center()
            .child(label(name, t.foreground))
            .key(name.to_lowercase())
    };
    let row_item = |name: &str, y: f32, chosen: bool| {
        let e = at(el(), [8.0, y, 176.0, 28.0], 8.4).px(10.0).center_y();
        let e = if chosen { apply(e, &m(Pane::State)) } else { e };
        e.child(label(
            name,
            if chosen {
                t.foreground
            } else {
                t.muted_foreground
            },
        ))
        .key(format!("row-{name}"))
    };
    // The list is a panel; its rows sit in it.
    let list = at(
        apply(el(), &m(Pane::Panel)),
        [24.0, 160.0, 192.0, 108.0],
        11.2,
    )
    .key("list")
    .child(row_item("Sources", 8.0, false))
    .child(row_item("Experiments", 40.0, true))
    .child(row_item("Draft", 72.0, false));
    let menu = at(
        apply(el(), &unblurred(Pane::Popover)),
        [240.0, 160.0, 192.0, 108.0],
        14.0,
    )
    .key("menu")
    .child(row_item("Rename", 8.0, false))
    .child(row_item("Copy link", 40.0, true))
    .child(row_item("Delete", 72.0, false));
    // CSS boxes with a 1 pt border: the element is the padding box (see the crate docs).
    let primary = |p: Pane, x: f32, name: &str| {
        at(
            apply(el(), &unblurred(p)),
            [x + 1.0, 293.0, 86.0, 30.0],
            10.2,
        )
        .center()
        .child(label(name, t.primary_foreground))
        .key(name.to_lowercase())
    };
    let glass = at(
        apply(el(), &unblurred(Pane::Control)),
        [324.0, 292.0, 88.0, 32.0],
        11.2,
    )
    .center()
    .child(label("Later", t.foreground))
    .key("glass");
    let field = |p: Pane, x: f32, name: &str| {
        at(apply(el(), &m(p)), [x + 1.0, 349.0, 174.0, 30.0], 10.2)
            .px(9.0)
            .center_y()
            .child(label(name, t.muted_foreground))
            .key(name.to_lowercase().replace(' ', "-"))
    };
    let switch = |on: bool, x: f32| {
        let (track, knob) = if on {
            (Pane::GrooveOn, Pane::KnobOn)
        } else {
            (Pane::Groove, Pane::Knob)
        };
        at(apply(el(), &m(track)), [x + 1.0, 405.0, 30.0, 16.4], 8.2)
            .key(if on { "switch-on" } else { "switch-off" })
            .child(at(
                apply(el(), &m(knob)),
                [if on { 14.0 } else { 0.0 }, 0.2, 16.0, 16.0],
                8.0,
            ))
    };
    // The kit's own components, themed by `kit()`: a submit button, a field, a switch.
    let kit_row = row()
        .absolute(456.0, 292.0)
        .gap(12.0)
        .center_y()
        .key("kit")
        .child(Button::new("Send").variant(ButtonVariant::Submit).build(&k))
        .child(Button::new("Cancel").build(&k))
        .child(Switch::new(true).build(&k));
    let field_input = TextInput::new(&s.edit)
        .placeholder("Kit field")
        .state(State::Rest)
        .build(&k);
    let kit_field = col()
        .absolute(456.0, 348.0)
        .w(240.0)
        .items_stretch()
        .child(field_input.w_full());
    el().w_full()
        .h_full()
        .fill(t.background)
        .child(page_text(look, 46.0, 3))
        .child(pane(Pane::Panel, [24.0, 32.0, 200.0, 80.0], 11.2, "Panel"))
        .child(pane(Pane::Card, [260.0, 32.0, 200.0, 80.0], 19.6, "Card"))
        .child(pane(
            Pane::Popover,
            [496.0, 32.0, 200.0, 80.0],
            14.0,
            "Popover",
        ))
        .child(page_text(look, 172.0, 4))
        .child(list)
        .child(menu)
        .child(primary(Pane::Primary, 24.0, "Send"))
        .child(primary(Pane::PrimaryHover, 124.0, "Hover"))
        .child(primary(Pane::PrimaryPressed, 224.0, "Pressed"))
        .child(glass)
        .child(field(Pane::Field, 24.0, "Field"))
        .child(field(Pane::FieldFocus, 216.0, "Field in focus"))
        .child(switch(false, 24.0))
        .child(switch(true, 72.0))
        .child(kit_row)
        .child(kit_field)
}

fn main() {
    rune::run(AppSpec::<Gallery> {
        scenes: &[("opaque", |s| s.opaque = true)],
        ..AppSpec::new("Solid Glass", SIZE, Gallery::default, view)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::motion_gate::{Run, check::paint::S_BACKDROP};
    use rune::{Env, Runtime, render::KIND_BACKDROP_BLUR};

    /// Every blur the gallery asks for draws: it asks for no more than a frame blurs.
    #[test]
    fn every_blur_the_gallery_asks_for_draws() {
        for (dark, opaque) in [(false, false), (true, false), (true, true)] {
            let gallery = Gallery {
                opaque,
                ..Gallery::default()
            };
            let mut run = Run::new(Runtime::new(gallery, view, SIZE, Env::new(dark, 2.0)), 60.0);
            run.step();
            let asked = run
                .rec
                .tracks
                .iter()
                .filter(|t| t.paint_at(0).is_some_and(|p| p.sizes[S_BACKDROP] > 0.0))
                .count();
            let drawn = run
                .rt
                .instances
                .iter()
                .filter(|i| i.params[1] == KIND_BACKDROP_BLUR)
                .count();
            assert_eq!(
                asked,
                if opaque { 0 } else { rune::MAX_BACKDROP_BLURS },
                "dark {dark}, opaque {opaque}"
            );
            assert_eq!(
                drawn, asked,
                "dark {dark}, opaque {opaque}: a blur past the cap draws only its fill"
            );
            assert!(run.rt.find("row-Experiments").is_some() && run.rt.find("kit").is_some());
        }
    }
}
