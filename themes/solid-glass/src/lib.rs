//! Solid Glass as data: thick panes of smoky glass whose finish is at the edge (a line of light
//! on top, a hairline round it, a shadow under it), after `solid-glass.css` (see `NOTICE`).
//!
//! Every pane is one [`Material`] read from a table ([`Look::material`]) and painted by
//! [`apply`]; [`kit`] gives rune-kit components the same look.
//!
//! ```ignore
//! let look = Look { dark: cx.system_dark(), opaque: false };
//! let card = apply(col().radius(19.6).padding(16.0), &look.material(Pane::Card));
//! ```
//!
//! An element that [`apply`] paints is the CSS padding box. For most panes that is the whole box.
//! The primary button, the field and the switch track keep a CSS `border` of one point, drawn as
//! an outer ring layer: size those one point in from the CSS box on each side, with corners one
//! point smaller. CSS clips their fill to the padding box, and a Rune fill runs under a Rune
//! border, so the ring keeps the hairline off the fill as CSS does.

use rune::{Color, El, Shadow, oklcha};
use rune_kit::{Elevation, Kit, Radius};

/// Light or dark, and whether panes give up their blur for opaque fills (CSS
/// `prefers-reduced-transparency`, or a part of the page marked `data-glass="opaque"`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Look {
    pub dark: bool,
    pub opaque: bool,
}

/// What a pane is. Thickness runs panel < card < popover: the higher it floats, the more opaque.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pane {
    /// `.glass-panel`: the thinnest, the columns a page is laid out in.
    Panel,
    /// `.bg-card`.
    Card,
    /// `.bg-popover`: menus, popovers and dialogs.
    Popover,
    /// `.bg-sidebar` with `[data-slot="sidebar-inner"]`'s edge: lit along the side facing the page.
    Sidebar,
    /// `.glass-control`: a glass button, a message bubble.
    Control,
    /// The one state (`.glass-chosen`): hovered, chosen or held open.
    State,
    /// `.glass-chosen.glass-armed`: the state a step brighter.
    Armed,
    /// The primary button: a solid pane the colour of the ink.
    Primary,
    PrimaryHover,
    PrimaryPressed,
    /// A text field at rest, and while it is typed into.
    Field,
    FieldFocus,
    /// A switch track that is off: cut into the surface, lit from below.
    Groove,
    /// A switch track that is on: the ink's pane without the shadow a button stands on.
    GrooveOn,
    /// A switch's thumb, on an off track and on an on one.
    Knob,
    KnobOn,
}

/// One pane: its fill, the backdrop blur under it (σ in points, 0 for none) and its box shadows
/// in Rune's paint order (the last on top; CSS lists the top one first).
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub fill: Color,
    pub blur: f32,
    pub layers: Vec<Shadow>,
}

/// Paints `m` on `e`. `e` is the CSS padding box (see the crate docs).
pub fn apply(e: El, m: &Material) -> El {
    m.layers.iter().fold(e.fill(m.fill).backdrop_blur(m.blur), |e, s| e.box_shadow(*s))
}

/// The tokens of `:root` (light) and `.dark` that the material reads, as `solid-glass.css` sets them.
#[derive(Clone, Copy, Debug)]
pub struct Tokens {
    /// `--background`
    pub background: Color,
    /// `--foreground`
    pub foreground: Color,
    /// `--muted-foreground`
    pub muted_foreground: Color,
    /// `--panel`
    pub panel: Color,
    /// `--card`
    pub card: Color,
    /// `--popover`
    pub popover: Color,
    /// `--sidebar`
    pub sidebar: Color,
    /// `--primary`
    pub primary: Color,
    /// `--primary-foreground`
    pub primary_foreground: Color,
    /// `--border`
    pub border: Color,
    /// `--input`, as a switch track that is off draws it (`dark:bg-input/80` in dark).
    pub input: Color,
    /// `--destructive`
    pub destructive: Color,
    /// `--glass-rim`
    pub glass_rim: Color,
    /// `--glass-inner`
    pub glass_inner: Color,
    /// `--glass-edge`
    pub glass_edge: Color,
    /// `--popover-edge`
    pub popover_edge: Color,
    /// `--popover-under`
    pub popover_under: Color,
    /// `--glass-shade`
    pub glass_shade: Color,
    /// `--glass-contact`
    pub glass_contact: Color,
    /// `--glass-lift`
    pub glass_lift: Color,
    /// `--glass-control`
    pub glass_control: Color,
    /// `--glass-state`
    pub glass_state: Color,
    /// `--glass-state-rim`
    pub glass_state_rim: Color,
    /// `--glass-state-edge`
    pub glass_state_edge: Color,
    /// `--glass-state-armed`
    pub glass_state_armed: Color,
    /// `--hover`
    pub hover: Color,
    /// `--primary-solid`, as a share.
    pub primary_solid: f32,
    /// `--primary-rim`
    pub primary_rim: Color,
    /// `--primary-under`
    pub primary_under: Color,
    /// `--primary-inner`
    pub primary_inner: Color,
    /// `--primary-edge`
    pub primary_edge: Color,
    /// `--field`
    pub field: Color,
    /// `--field-rim`
    pub field_rim: Color,
    /// `--field-edge`
    pub field_edge: Color,
    /// `--well-shade`
    pub well_shade: Color,
    /// `--well-rim`
    pub well_rim: Color,
    /// `--focus-line`
    pub focus_line: Color,
    /// `--knob-shade`
    pub knob_shade: Color,
    /// `--glass-blur`'s σ in points.
    pub blur: f32,
    /// `--glass-blur-control`'s σ in points.
    pub blur_control: f32,
}

/// `oklch(l c h / a)`.
fn ok(l: f32, c: f32, h: f32, a: f32) -> Color {
    oklcha(l, c, h, a)
}

fn light() -> Tokens {
    let ink = |a| ok(0.3, 0.02, 265.0, a);
    let white = |a| ok(1.0, 0.0, 0.0, a);
    Tokens {
        background: ok(0.925, 0.007, 265.0, 1.0),
        foreground: ok(0.16, 0.0, 0.0, 1.0),
        muted_foreground: ok(0.47, 0.0, 0.0, 1.0),
        panel: ok(0.975, 0.004, 265.0, 0.44),
        card: ok(0.98, 0.004, 265.0, 0.6),
        popover: ok(0.985, 0.003, 265.0, 0.96),
        sidebar: ok(0.975, 0.004, 265.0, 0.44),
        primary: ok(0.2, 0.0, 0.0, 1.0),
        primary_foreground: ok(0.985, 0.0, 0.0, 1.0),
        border: ink(0.13),
        input: ink(0.15),
        destructive: ok(0.577, 0.245, 27.325, 1.0),
        glass_rim: white(0.9),
        glass_inner: white(0.35),
        glass_edge: ok(0.38, 0.03, 265.0, 0.1),
        popover_edge: ok(0.38, 0.03, 265.0, 0.22),
        popover_under: ok(0.38, 0.03, 265.0, 0.16),
        glass_shade: ok(0.3, 0.06, 265.0, 0.28),
        glass_contact: ok(0.25, 0.03, 265.0, 0.06),
        glass_lift: ok(0.25, 0.03, 265.0, 0.14),
        glass_control: white(0.55),
        glass_state: ink(0.09),
        glass_state_rim: white(0.85),
        glass_state_edge: ink(0.13),
        glass_state_armed: ink(0.26),
        hover: ink(0.1),
        primary_solid: 0.88,
        primary_rim: white(0.3),
        primary_under: ok(0.0, 0.0, 0.0, 0.45),
        primary_inner: white(0.07),
        primary_edge: ok(0.2, 0.0, 0.0, 0.3),
        field: white(0.6),
        field_rim: white(1.0),
        field_edge: ink(0.14),
        well_shade: ok(0.3, 0.06, 265.0, 0.14),
        well_rim: white(0.85),
        focus_line: ok(0.2, 0.0, 0.0, 0.5),
        knob_shade: ok(0.3, 0.06, 265.0, 0.3),
        // ponytail: light's `--glass-blur` is `blur(26px) saturate(180%)`; Rune's backdrop blur has
        // no saturation. The page under it is near grey, so it shows only over colour; add a
        // saturation step to the blur's composite when a light pane sits over a picture.
        blur: 26.0,
        blur_control: 14.0,
    }
}

fn dark() -> Tokens {
    let white = |a| ok(1.0, 0.0, 0.0, a);
    let black = |a| ok(0.0, 0.0, 0.0, a);
    Tokens {
        background: ok(0.16, 0.0, 0.0, 1.0),
        foreground: ok(0.985, 0.0, 0.0, 1.0),
        muted_foreground: ok(0.72, 0.0, 0.0, 1.0),
        panel: white(0.045),
        card: ok(0.31, 0.0, 0.0, 0.46),
        popover: ok(0.22, 0.0, 0.0, 0.96),
        sidebar: ok(0.11, 0.0, 0.0, 0.5),
        primary: ok(0.94, 0.0, 0.0, 1.0),
        primary_foreground: ok(0.18, 0.0, 0.0, 1.0),
        border: white(0.11),
        input: white(0.13 * 0.8),
        destructive: ok(0.704, 0.191, 22.216, 1.0),
        glass_rim: white(0.16),
        glass_inner: white(0.04),
        glass_edge: white(0.06),
        popover_edge: white(0.06),
        popover_under: black(0.0),
        glass_shade: black(0.6),
        glass_contact: black(0.1),
        glass_lift: black(0.22),
        glass_control: white(0.07),
        glass_state: white(0.07),
        glass_state_rim: white(0.11),
        glass_state_edge: white(0.06),
        glass_state_armed: white(0.11),
        hover: white(0.08),
        primary_solid: 0.86,
        primary_rim: white(1.0),
        primary_under: black(0.22),
        primary_inner: black(0.07),
        primary_edge: white(0.26),
        field: white(0.06),
        field_rim: white(0.15),
        field_edge: white(0.08),
        well_shade: black(0.38),
        well_rim: white(0.09),
        focus_line: white(0.4),
        knob_shade: black(0.55),
        blur: 26.0,
        blur_control: 14.0,
    }
}

fn drop(y: f32, blur: f32, spread: f32, color: Color) -> Shadow {
    Shadow { y, blur, spread, color, ..Shadow::default() }
}

fn inset(y: f32, blur: f32, color: Color) -> Shadow {
    Shadow { y, blur, color, inset: true, ..Shadow::default() }
}

/// A CSS `0 0 0 1px` hairline, and a CSS `border: 1px`, which the padding box wears outside.
fn ring(color: Color) -> Shadow {
    drop(0.0, 0.0, 1.0, color)
}

/// A CSS `box-shadow` list, top layer first, in Rune's paint order.
fn css<const N: usize>(mut layers: [Shadow; N]) -> Vec<Shadow> {
    layers.reverse();
    layers.into()
}

/// `color-mix(in oklab, a p, b)`: premultiplied, as CSS mixes.
fn mix_oklab(a: Color, p: f32, b: Color) -> Color {
    let (x, y) = (a.to_oklab(), b.to_oklab());
    let alpha = x[3] * p + y[3] * (1.0 - p);
    if alpha <= 0.0 {
        return Color::TRANSPARENT;
    }
    let m = |i: usize| (x[i] * x[3] * p + y[i] * y[3] * (1.0 - p)) / alpha;
    Color::from_oklab([m(0), m(1), m(2), alpha])
}

impl Look {
    /// The tokens of this look. Opaque swaps in the reduced-transparency fills.
    pub fn tokens(self) -> Tokens {
        let mut t = if self.dark { dark() } else { light() };
        if self.opaque {
            // ponytail: CSS only drops the blur under `data-glass="opaque"`; the opaque fills are
            // its `prefers-reduced-transparency` ones, which is what "opaque" asks for.
            t.primary_solid = 1.0;
            [t.panel, t.card, t.popover, t.sidebar] = if self.dark {
                [ok(0.19, 0.0, 0.0, 1.0), ok(0.225, 0.0, 0.0, 1.0), ok(0.22, 0.0, 0.0, 1.0), ok(0.13, 0.0, 0.0, 1.0)]
            } else {
                [
                    ok(0.947, 0.006, 265.0, 1.0),
                    ok(0.958, 0.005, 265.0, 1.0),
                    ok(0.98, 0.003, 265.0, 1.0),
                    ok(0.985, 0.0, 0.0, 1.0),
                ]
            };
        }
        t
    }

    /// The material of `pane` in this look.
    pub fn material(self, pane: Pane) -> Material {
        let t = self.tokens();
        let blur = |sigma: f32| if self.opaque { 0.0 } else { sigma };
        let solid = |share: f32| t.primary.alpha(t.primary.a * share);
        // A primary button's border: the ink toned toward the edge's colour.
        let primary_edge = mix_oklab(t.primary, 0.55, t.primary_edge);
        let pane_layers = || {
            css([
                inset(1.0, 0.0, t.glass_rim),
                Shadow { spread: 1.0, ..inset(0.0, 0.0, t.glass_inner) },
                ring(t.glass_edge),
                drop(1.0, 2.0, 0.0, t.glass_contact),
                drop(12.0, 32.0, -14.0, t.glass_shade),
            ])
        };
        // Its outer shadows stand on the CSS border box, a point out from the padding box.
        let primary = |share: f32, pressed: bool| {
            let stand: &[Shadow] = if pressed {
                &[drop(1.0, 1.0, 1.0, t.glass_contact)]
            } else {
                &[drop(1.0, 2.0, 1.0, t.glass_contact), drop(8.0, 18.0, -8.0, t.glass_shade)]
            };
            let mut layers: Vec<Shadow> = stand.iter().rev().copied().collect();
            layers.extend(css([
                ring(primary_edge),
                inset(1.0, 0.0, t.primary_rim),
                inset(-1.0, 0.0, t.primary_under),
                Shadow { spread: 1.0, ..inset(0.0, 0.0, t.primary_inner) },
            ]));
            Material { fill: solid(share), blur: blur(t.blur_control), layers }
        };
        let field = |edge: Color| Material {
            fill: t.field,
            blur: 0.0,
            layers: css([ring(edge), inset(1.0, 0.0, t.field_rim)]),
        };
        // The thumb is `bg-background` in light; in dark the foreground, or the primary's ink on
        // an on track.
        let knob = |fill: Color| Material {
            fill,
            blur: 0.0,
            layers: css([
                inset(1.0, 0.0, ok(1.0, 0.0, 0.0, 0.9)),
                inset(-1.0, 0.0, ok(0.0, 0.0, 0.0, 0.14)),
                drop(1.0, 2.0, 0.0, t.knob_shade),
            ]),
        };
        let mut m = match pane {
            Pane::Panel => Material { fill: t.panel, blur: blur(t.blur), layers: pane_layers() },
            Pane::Card => Material { fill: t.card, blur: blur(t.blur), layers: pane_layers() },
            Pane::Popover => Material {
                fill: t.popover,
                blur: blur(t.blur),
                layers: css([
                    inset(1.0, 0.0, t.glass_rim),
                    inset(-1.0, 0.0, t.popover_under),
                    ring(t.popover_edge),
                    drop(24.0, 60.0, -18.0, t.glass_shade),
                    drop(6.0, 16.0, -6.0, t.glass_lift),
                ]),
            },
            Pane::Sidebar => Material {
                fill: t.sidebar,
                blur: blur(t.blur),
                layers: css([Shadow { x: -1.0, ..inset(0.0, 0.0, t.glass_edge) }, inset(1.0, 0.0, t.glass_inner)]),
            },
            Pane::Control => Material {
                fill: t.glass_control,
                blur: blur(t.blur_control),
                layers: css([inset(1.0, 0.0, t.glass_rim), ring(t.glass_edge), drop(1.0, 2.0, 0.0, t.glass_contact)]),
            },
            Pane::State => Material {
                fill: t.glass_state,
                blur: 0.0,
                layers: css([inset(1.0, 0.0, t.glass_state_rim), ring(t.glass_state_edge)]),
            },
            Pane::Armed => Material {
                // A second coat of the state's own fill.
                fill: t.glass_state.alpha(1.0 - (1.0 - t.glass_state.a).powi(2)),
                blur: 0.0,
                layers: css([inset(1.0, 0.0, t.glass_rim), ring(t.glass_state_armed)]),
            },
            Pane::Primary => primary(t.primary_solid, false),
            Pane::PrimaryHover => primary(t.primary_solid - 0.1, false),
            Pane::PrimaryPressed => primary(t.primary_solid, true),
            Pane::Field => field(t.field_edge),
            Pane::FieldFocus => field(t.focus_line),
            // The track keeps a transparent border its fill runs under: a ring of the fill.
            Pane::Groove => Material {
                fill: t.input,
                blur: 0.0,
                layers: css([inset(1.0, 1.5, t.well_shade), inset(-1.0, 0.0, t.well_rim), ring(t.input)]),
            },
            // Its border is its two lines, over the fill: the light along the top, the shade
            // along the bottom. Copies of the pane a point up and down show only in that row.
            Pane::GrooveOn => Material {
                fill: solid(t.primary_solid),
                blur: 0.0,
                layers: css([
                    drop(1.0, 0.0, 0.0, t.primary_under.alpha(t.primary_under.a * 0.45)),
                    drop(-1.0, 0.0, 0.0, t.primary_rim),
                    ring(solid(t.primary_solid)),
                ]),
            },
            Pane::Knob => knob(if self.dark { t.foreground } else { t.background }),
            Pane::KnobOn => knob(if self.dark { t.primary_foreground } else { t.background }),
        };
        m.layers.retain(|s| s.color.a > 0.0);
        m
    }
}

/// The rune-kit theme for one appearance: `Kit::neutral`'s type, glyphs and motion with Solid
/// Glass's colours, corners and edges. Submit buttons are the primary's ink pane.
pub fn kit(dark: bool) -> Kit {
    let look = Look { dark, opaque: false };
    let t = look.tokens();
    let m = |p: Pane| look.material(p);
    let mut k = Kit::neutral(dark);
    let c = &mut k.c;
    let ink = m(Pane::Primary).fill;
    c.page = t.background;
    c.bg_base = t.field;
    c.layer_01 = t.popover;
    c.layer_04 = t.input;
    c.surface_raised = t.popover;
    c.bg_button_neutral = t.glass_control;
    c.bg_contrast = ink;
    c.bg_icon_button_contrast = ink;
    c.bg_accent = ink;
    c.submit_highlight = Color::TRANSPARENT;
    // Under the pointer the ink thins by a tenth: the page laid over it at the share that leaves
    // `solid - 0.1` of the ink when it stands on the page.
    c.submit_hover = t.background.alpha(0.1 / t.primary_solid);
    c.contrast_pressed = c.submit_hover;
    for ink in [&mut c.submit_fg, &mut c.submit_icon, &mut c.text_contrast, &mut c.icon_contrast, &mut c.text_inverse] {
        *ink = t.primary_foreground;
    }
    for text in [&mut c.text_base, &mut c.text_strong, &mut c.icon_base, &mut c.std_text] {
        *text = t.foreground;
    }
    for muted in [&mut c.text_muted, &mut c.text_weak, &mut c.icon_muted] {
        *muted = t.muted_foreground;
    }
    c.border_muted = t.border;
    c.border_base = t.border;
    c.border_focus = t.focus_line;
    c.hover = t.hover;
    c.pressed = t.glass_state_armed;
    c.knob = m(Pane::Knob).fill;
    c.knob_on = m(Pane::KnobOn).fill;
    c.scrim = Color::BLACK.alpha(0.1);
    let mut e = Elevation::default().with_input(m(Pane::Field).layers);
    e.raised = m(Pane::Panel).layers;
    e.floating = m(Pane::Popover).layers;
    e.overlay = m(Pane::Popover).layers;
    e.md = m(Pane::Popover).layers;
    e.button_neutral = m(Pane::Control).layers;
    e.button_contrast = m(Pane::Primary).layers;
    e.switch_off = m(Pane::Groove).layers;
    e.switch_on = m(Pane::GrooveOn).layers;
    e.elements = m(Pane::Knob).layers;
    e.knob = m(Pane::Knob).layers;
    k.elevation = e;
    // `--radius` 0.875rem: sm, md, lg, xl are 0.6, 0.8, 1 and 1.4 of it.
    k.radius = Radius { xs: 4.0, sm: 8.4, md: 11.2, lg: 14.0, xl: 19.6 };
    k
}

#[cfg(test)]
mod tests;
