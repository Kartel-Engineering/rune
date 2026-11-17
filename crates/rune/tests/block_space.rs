//! `El::block_space_above`: extra space above a rich field's blocks, by kind. The caret, clicks, Up
//! and Down, a selection, the field's height, caret following and block decorations all agree
//! with where the blocks are drawn.
use rune::runtime::ElementInfo;
use rune::{
    Block, Color, Direction, El, Env, Ime, Key, KeyEvent, RichStyle, Runtime, ScrollSpec, TextEdit,
    TextStyle, col,
};

struct S {
    field: TextEdit,
    space: Option<fn(Block) -> f32>,
    viewport: Option<f32>,
    dir: Option<Direction>,
}

fn ink(hex: &str) -> Color {
    rune::color::hex(hex).unwrap()
}
fn caret_ink() -> Color {
    ink("#ff0077")
}
fn selection_ink() -> Color {
    ink("#00ff55")
}
fn code_ink() -> Color {
    ink("#3355ff")
}
fn bar_ink() -> Color {
    ink("#ffaa00")
}

fn headings(b: Block) -> f32 {
    if let Block::Heading(_) = b { 30.0 } else { 0.0 }
}
fn everywhere(_: Block) -> f32 {
    30.0
}
fn ten(b: Block) -> f32 {
    if let Block::Heading(_) = b { 10.0 } else { 0.0 }
}

fn view(s: &S) -> El {
    let rich = RichStyle {
        code_block_bg: code_ink(),
        quote_bar: bar_ink(),
        ..RichStyle::default()
    };
    let mut field = rune::text_input(&s.field, TextStyle::new(14.0, 400.0).lh(20.0))
        .key("field")
        .w(250.0)
        .caret(caret_ink(), selection_ink())
        .rich_style(rich)
        .on_edit(|s: &mut S| &mut s.field);
    if let Some(space) = s.space {
        field = field.block_space_above(space);
    }
    let body = match s.viewport {
        Some(h) => col()
            .key("viewport")
            .w_full()
            .h(h)
            .scroll(ScrollSpec {
                stick_bottom: false,
            })
            .child(field),
        None => field,
    };
    let page = col().w_full().h_full().padding(10.0).child(body);
    match s.dir {
        Some(dir) => page.dir(dir),
        None => page,
    }
}

fn runtime(field: TextEdit, space: Option<fn(Block) -> f32>, viewport: Option<f32>) -> Runtime<S> {
    runtime_with(field, space, viewport, None, 1.0)
}

fn runtime_with(
    field: TextEdit,
    space: Option<fn(Block) -> f32>,
    viewport: Option<f32>,
    dir: Option<Direction>,
    scale: f32,
) -> Runtime<S> {
    let mut field = field;
    field.focused = true;
    let mut rt = Runtime::new(
        S {
            field,
            space,
            viewport,
            dir,
        },
        |s: &S, _: &mut rune::Cx<S>| view(s),
        (300.0, 400.0),
        Env::new(true, scale),
    );
    rt.frame(0.0);
    rt
}

fn find(rt: &Runtime<S>, key: &str) -> ElementInfo {
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key))
        .unwrap()
}

/// The caret line at byte `at`, `[x, y, w, h]` in window points.
fn caret_at(rt: &mut Runtime<S>, at: usize) -> [f32; 4] {
    rt.update(|s, _| s.field.set_selection(at, at));
    rt.frame(1.0 / 60.0);
    find(rt, "field").caret.expect("caret")
}

fn key(rt: &mut Runtime<S>, key: Key) {
    rt.key(KeyEvent {
        key,
        mods: Default::default(),
    });
    rt.frame(1.0 / 60.0);
}

fn click(rt: &mut Runtime<S>, x: f32, y: f32) -> usize {
    for _ in 0..40 {
        rt.frame(1.0 / 60.0);
    }
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
    rt.state.field.focus
}

/// Drawn rects filled with `c`, `[x, y, w, h]` in device px.
fn rects(rt: &Runtime<S>, c: Color) -> Vec<[f32; 4]> {
    let want = c.premul(1.0);
    rt.instances
        .iter()
        .filter(|i| i.fill.iter().zip(want).all(|(a, b)| (a - b).abs() < 1e-3))
        .map(|i| i.rect)
        .collect()
}

/// Everything drawn, as `(rect, fill)`.
type Drawn = Vec<([f32; 4], [f32; 4])>;

fn drawn(rt: &Runtime<S>) -> Drawn {
    rt.instances.iter().map(|i| (i.rect, i.fill)).collect()
}

fn boundaries(rt: &Runtime<S>) -> Vec<usize> {
    let text = &rt.state.field.text;
    (0..=text.len())
        .filter(|&i| text.is_char_boundary(i))
        .collect()
}

/// The caret line at each char boundary.
fn carets(rt: &mut Runtime<S>) -> Vec<[f32; 4]> {
    boundaries(rt)
        .into_iter()
        .map(|at| caret_at(rt, at))
        .collect()
}

/// Where Up and Down go from each char boundary.
fn ups_and_downs(rt: &mut Runtime<S>) -> Vec<[usize; 2]> {
    let mut out = Vec::new();
    for at in boundaries(rt) {
        let mut to = [0; 2];
        for (j, k) in [Key::Up, Key::Down].into_iter().enumerate() {
            rt.update(|s, _| s.field.set_selection(at, at));
            rt.frame(1.0 / 60.0);
            key(rt, k);
            to[j] = rt.state.field.focus;
        }
        out.push(to);
    }
    out
}

/// How far the space moved byte `at`'s caret down.
fn shift(plain: &mut Runtime<S>, spaced: &mut Runtime<S>, at: usize) -> f32 {
    caret_at(spaced, at)[1] - caret_at(plain, at)[1]
}

/// "Intro", an empty line, an H1, a paragraph and two bullets; paragraph starts.
fn document() -> (TextEdit, [usize; 6]) {
    let blocks = vec![
        Block::Paragraph,
        Block::Paragraph,
        Block::Heading(1),
        Block::Paragraph,
        Block::Bullet(0),
        Block::Bullet(0),
    ];
    (
        TextEdit::from_parts("Intro\n\nTitle\nBody text\none\ntwo", vec![], blocks),
        [0, 6, 7, 13, 23, 27],
    )
}

#[test]
fn each_block_moves_down_by_the_space_above_it_and_the_field_grows() {
    let (doc, starts) = document();
    let (mut plain, mut spaced) = (
        runtime(doc.clone(), None, None),
        runtime(doc, Some(headings), None),
    );
    let moved: Vec<f32> = starts
        .iter()
        .map(|&at| caret_at(&mut spaced, at)[1] - caret_at(&mut plain, at)[1])
        .collect();
    assert_eq!(
        moved,
        [0.0, 0.0, 30.0, 30.0, 30.0, 30.0],
        "30 above the heading, none above the first block"
    );
    let (p, s) = (find(&plain, "field").rect, find(&spaced, "field").rect);
    assert_eq!(
        s[3] - p[3],
        30.0,
        "the field is taller by the space: {p:?} {s:?}"
    );
    // The caret on the heading after the space: as tall as before, and the IME area with it.
    let (c, d) = (caret_at(&mut spaced, 9), caret_at(&mut plain, 9));
    assert_eq!([c[0], c[2], c[3]], [d[0], d[2], d[3]]);
    let ime = spaced.ime_area.expect("IME area");
    assert!(
        (ime[1] - c[1]).abs() < 0.01 && (ime[3] - c[3]).abs() < 0.01,
        "IME area {ime:?} on the caret {c:?}"
    );
}

#[test]
fn the_first_block_gets_no_space() {
    let doc = TextEdit::from_parts(
        "Title\nbody",
        vec![],
        vec![Block::Heading(1), Block::Paragraph],
    );
    let (mut plain, mut spaced) = (
        runtime(doc.clone(), None, None),
        runtime(doc, Some(everywhere), None),
    );
    assert_eq!(caret_at(&mut spaced, 0)[1], caret_at(&mut plain, 0)[1]);
    assert_eq!(
        caret_at(&mut spaced, 6)[1] - caret_at(&mut plain, 6)[1],
        30.0
    );
}

#[test]
fn zero_space_draws_exactly_as_no_space() {
    // A left-to-right field, one with a Hebrew paragraph, and at 1.125 lines of 22.5 px: a
    // stack of paragraph layouts would draw each differently.
    let mixed = TextEdit::from_parts(
        "Intro\nשלום עולם\nTitle\nend",
        vec![],
        vec![
            Block::Paragraph,
            Block::Paragraph,
            Block::Heading(2),
            Block::Paragraph,
        ],
    );
    for doc in [document().0, mixed] {
        for scale in [1.0, 1.125] {
            let (mut plain, mut zero) = (
                runtime_with(doc.clone(), None, None, None, scale),
                runtime_with(doc.clone(), Some(|_| 0.0), None, None, scale),
            );
            assert_eq!(answers(&mut zero), answers(&mut plain), "scale {scale}");
            assert_eq!(find(&plain, "field").rect, find(&zero, "field").rect);
        }
    }
}

#[test]
fn a_click_in_the_space_lands_on_the_nearer_line() {
    let (doc, _) = document();
    let mut rt = runtime(doc, Some(headings), None);
    let f = find(&rt, "field").rect;
    // The empty line spans 20..40 of the field, the space 40..70, the heading from 70.
    assert_eq!(
        click(&mut rt, f[0] + 1.0, f[1] + 46.0),
        6,
        "the upper half: the empty line above"
    );
    assert_eq!(
        click(&mut rt, f[0] + 1.0, f[1] + 64.0),
        7,
        "the lower half: the heading below"
    );
    assert_eq!(
        click(&mut rt, f[0] + 1.0, f[1] + 75.0),
        7,
        "on the heading itself"
    );
}

#[test]
fn up_and_down_cross_the_space_and_keep_the_column() {
    let mut rt = runtime(TextEdit::new("abcdefgh\nabcdefgh"), Some(everywhere), None);
    let above = caret_at(&mut rt, 4);
    key(&mut rt, Key::Down);
    assert_eq!(rt.state.field.focus, 13);
    let below = find(&rt, "field").caret.unwrap();
    assert_eq!(below[0], above[0], "the same column");
    assert_eq!(below[1] - above[1], 50.0, "a line and the space down");
    key(&mut rt, Key::Up);
    assert_eq!(rt.state.field.focus, 4);
}

#[test]
fn a_selection_across_the_space_leaves_it_bare() {
    let mut rt = runtime(TextEdit::new("abc\ndef"), Some(everywhere), None);
    rt.update(|s, _| s.field.set_selection(0, 7));
    rt.frame(1.0 / 60.0);
    let f = find(&rt, "field").rect;
    let boxes = rects(&rt, selection_ink());
    assert!(boxes.len() >= 2, "a box on each line: {boxes:?}");
    for b in &boxes {
        let (top, bottom) = (b[1] - f[1], b[1] + b[3] - f[1]);
        assert!(
            bottom <= 20.01 || top >= 49.99,
            "box {b:?} reaches into the space 20..50 of {f:?}"
        );
    }
    assert!(
        boxes.iter().any(|b| (b[1] - f[1] - 50.0).abs() < 0.01),
        "the second line's box starts after the space: {boxes:?}"
    );
}

#[test]
fn typing_on_the_last_line_scrolls_it_into_view() {
    let doc = TextEdit::from_parts(
        "A\none\nB\ntwo\nC\nthree",
        vec![],
        vec![
            Block::Heading(2),
            Block::Paragraph,
            Block::Heading(2),
            Block::Paragraph,
            Block::Heading(2),
            Block::Paragraph,
        ],
    );
    let mut rt = runtime(doc, Some(headings), Some(60.0));
    let end = rt.state.field.text.len();
    rt.update(|s, _| s.field.set_selection(end, end));
    rt.frame(1.0 / 60.0);
    rt.ime(Ime::Commit("x".into()));
    rt.frame(1.0 / 60.0);
    let (c, v) = (
        find(&rt, "field").caret.unwrap(),
        find(&rt, "viewport").rect,
    );
    assert!(
        c[1] >= v[1] - 0.01 && c[1] + c[3] <= v[1] + v[3] + 0.01,
        "caret {c:?} inside viewport {v:?}"
    );
    assert!(
        find(&rt, "field").rect[1] < v[1] - 60.0,
        "it scrolled past both spaces"
    );
}

#[test]
fn code_and_quote_runs_stay_one_box_and_one_bar_after_their_space() {
    let blocks = vec![
        Block::Paragraph,
        Block::Code,
        Block::Code,
        Block::Code,
        Block::Quote,
        Block::Quote,
        Block::Quote,
    ];
    let doc = TextEdit::from_parts("p\nc1\nc2\nc3\nq1\nq2\nq3", vec![], blocks);
    let (plain, spaced) = (
        runtime(doc.clone(), None, None),
        runtime(doc, Some(everywhere), None),
    );
    let (pc, sc) = (rects(&plain, code_ink()), rects(&spaced, code_ink()));
    let (pq, sq) = (rects(&plain, bar_ink()), rects(&spaced, bar_ink()));
    assert_eq!(
        (pc.len(), sc.len(), pq.len(), sq.len()),
        (1, 1, 1, 1),
        "one box and one bar: {sc:?} {sq:?}"
    );
    assert_eq!(
        sc[0][1] - pc[0][1],
        30.0,
        "the box starts after the space above the code"
    );
    assert_eq!(sc[0][3], pc[0][3], "no space between its lines");
    assert_eq!(
        sq[0][1] - pq[0][1],
        60.0,
        "the bar after the code's space and its own"
    );
    assert_eq!(sq[0][3], pq[0][3], "no space between its lines");
}

#[test]
fn a_spaced_field_reads_in_the_direction_its_one_layout_does() {
    // No strong character in the first line: the whole text's first, Hebrew, sets the field
    // right to left, as the one layout's automatic direction does.
    let doc = TextEdit::from_parts(
        "123\nשלום\nhello",
        vec![],
        vec![Block::Paragraph, Block::Heading(2), Block::Paragraph],
    );
    let (mut plain, mut spaced) = (
        runtime(doc.clone(), None, None),
        runtime(doc, Some(headings), None),
    );
    for at in [0, 1, 6, 15] {
        assert_eq!(
            caret_at(&mut spaced, at)[0],
            caret_at(&mut plain, at)[0],
            "caret x at byte {at}"
        );
    }
}

/// A right-to-left note with an English paragraph and a heading, as a Hebrew writer has it.
fn rtl_note() -> TextEdit {
    TextEdit::from_parts(
        "שלום עולם\nThe quick brown fox jumps over the lazy dog, again and again, until it wraps.\nכותרת\nסוף",
        vec![],
        vec![
            Block::Paragraph,
            Block::Paragraph,
            Block::Heading(2),
            Block::Paragraph,
        ],
    )
}

/// What a field shows and how it answers: every caret, Up and Down from each place, and what a
/// whole selection draws.
fn answers(rt: &mut Runtime<S>) -> (Vec<[f32; 4]>, Vec<[usize; 2]>, Drawn) {
    let (c, k) = (carets(rt), ups_and_downs(rt));
    let end = rt.state.field.text.len();
    rt.update(|s, _| s.field.set_selection(0, end));
    rt.frame(1.0 / 60.0);
    (c, k, drawn(rt))
}

#[test]
fn a_right_to_left_field_gets_no_space_and_draws_as_its_one_layout() {
    // Right to left from its text's first strong character, and from its element's direction.
    for dir in [None, Some(Direction::Rtl)] {
        let (mut plain, mut spaced) = (
            runtime_with(rtl_note(), None, None, dir, 1.0),
            runtime_with(rtl_note(), Some(everywhere), None, dir, 1.0),
        );
        assert_eq!(answers(&mut spaced), answers(&mut plain), "dir {dir:?}");
        assert_eq!(find(&spaced, "field").rect, find(&plain, "field").rect);
    }
    // A zero space draws as unset there too, by the same one layout.
    let (mut plain, mut zero) = (
        runtime(rtl_note(), None, None),
        runtime(rtl_note(), Some(|_| 0.0), None),
    );
    assert_eq!(answers(&mut zero), answers(&mut plain));
}

#[test]
fn the_space_follows_the_direction_as_the_first_strong_character_is_typed() {
    // An empty first line: the whole text's first strong character, typed later, decides.
    let doc = TextEdit::from_parts("\nTitle", vec![], vec![Block::Paragraph, Block::Heading(2)]);
    let (mut plain, mut spaced) = (
        runtime(doc.clone(), None, None),
        runtime(doc, Some(everywhere), None),
    );
    let type_at_start = |rt: &mut Runtime<S>, s: &str| {
        rt.update(|st, _| st.field.set_selection(0, 0));
        rt.frame(1.0 / 60.0);
        rt.ime(Ime::Commit(s.into()));
        // The field's height moves to its new layout; let it land.
        for _ in 0..40 {
            rt.frame(1.0 / 60.0);
        }
        rt.state.field.text.find('\n').unwrap() + 1
    };
    assert_eq!(shift(&mut plain, &mut spaced, 1), 30.0, "left to right");
    // A Hebrew letter first: the field turns right to left, and the space goes at once.
    let at = type_at_start(&mut spaced, "ש");
    assert_eq!(type_at_start(&mut plain, "ש"), at);
    assert_eq!(answers(&mut spaced), answers(&mut plain), "right to left");
    // An English letter in its place: left to right again, and the space comes back.
    let replace = |rt: &mut Runtime<S>| {
        rt.update(|st, _| st.field.set_selection(0, "ש".len()));
        rt.frame(1.0 / 60.0);
        rt.ime(Ime::Commit("a".into()));
        for _ in 0..40 {
            rt.frame(1.0 / 60.0);
        }
        rt.state.field.text.find('\n').unwrap() + 1
    };
    let at = replace(&mut spaced);
    assert_eq!(replace(&mut plain), at);
    assert_eq!(
        shift(&mut plain, &mut spaced, at),
        30.0,
        "left to right again"
    );
}

#[test]
fn a_field_set_left_to_right_keeps_its_direction_and_gets_the_space() {
    // Its element is left to right: the field is, so it gets the space.
    let doc = TextEdit::from_parts(
        "123\nTitle\nmore",
        vec![],
        vec![Block::Paragraph, Block::Heading(2), Block::Paragraph],
    );
    let ltr = Some(Direction::Ltr);
    let (mut plain, mut spaced) = (
        runtime_with(doc.clone(), None, None, ltr, 1.0),
        runtime_with(doc, Some(everywhere), None, ltr, 1.0),
    );
    for (at, down) in [(0, 0.0), (4, 30.0), (10, 60.0)] {
        let (s, p) = (caret_at(&mut spaced, at), caret_at(&mut plain, at));
        assert_eq!([s[0], s[1] - p[1]], [p[0], down], "caret at byte {at}");
    }
}

#[test]
fn a_line_a_soft_break_started_gets_no_space() {
    // A heading wrapped by Shift+Return, then a paragraph: the space only above that.
    let doc = TextEdit::from_parts(
        "Intro\nTitle\nmore\nbody",
        vec![],
        vec![
            Block::Paragraph,
            Block::Heading(2),
            Block::Heading(2),
            Block::Paragraph,
        ],
    )
    .with_soft_breaks(vec![2]);
    let (mut plain, mut spaced) = (
        runtime(doc.clone(), None, None),
        runtime(doc, Some(everywhere), None),
    );
    let moved: Vec<f32> = [0, 6, 12, 17]
        .iter()
        .map(|&at| shift(&mut plain, &mut spaced, at))
        .collect();
    assert_eq!(moved, [0.0, 30.0, 30.0, 60.0]);
}

#[test]
fn a_negative_nan_or_huge_space_draws_safely() {
    let (doc, starts) = document();
    let mut plain = runtime(doc.clone(), None, None);
    // Each case's space above every block but the first, in points.
    for (space, want) in [
        ((|_| -30.0) as fn(Block) -> f32, 0.0),
        (|_| f32::NAN, 0.0),
        (|_| f32::NEG_INFINITY, 0.0),
        (|_| f32::MAX, 10_000.0),
        (|_| f32::INFINITY, 10_000.0),
    ] {
        let mut rt = runtime(doc.clone(), Some(space), None);
        let gaps = (starts.len() - 1) as f32;
        let moved: Vec<f32> = starts
            .iter()
            .map(|&at| shift(&mut plain, &mut rt, at))
            .collect();
        let want: Vec<f32> = (0..starts.len()).map(|k| k as f32 * want).collect();
        assert_eq!(moved, want, "space {}", space(Block::Paragraph));
        let (p, s) = (find(&plain, "field").rect, find(&rt, "field").rect);
        assert!(s[3].is_finite(), "field height {s:?}");
        assert_eq!(s[3] - p[3], gaps * want[1], "field height");
    }
}

#[test]
fn the_layout_follows_a_change_of_space_and_of_block() {
    let field = TextEdit::from_parts("abc\ndef\nghi", vec![], vec![Block::Paragraph; 3]);
    let mut rt = runtime(field, Some(headings), None);
    let base = caret_at(&mut rt, 9)[1];
    let mut seen = vec![];
    // Paragraph 1 becomes a heading, back, again; then the space function changes, then goes.
    for step in 0..6 {
        rt.update(|s, _| {
            s.field.set_selection(5, 5);
            match step {
                0 => s.field.set_block(Block::Heading(1)),
                1 => s.field.set_block(Block::Paragraph),
                2 => s.field.set_block(Block::Heading(1)),
                3 => s.space = Some(ten),
                4 => s.space = None,
                _ => s.space = Some(headings),
            }
        });
        rt.frame(1.0 / 60.0);
        seen.push(caret_at(&mut rt, 9)[1] - base);
    }
    // A heading line draws taller: the space is the ghi line's shift past that growth.
    let plain_heading = seen[4];
    assert_eq!(seen[1], 0.0, "back to paragraph: no space {seen:?}");
    assert_eq!(seen[0], seen[2], "heading again: the same {seen:?}");
    assert_eq!(
        seen[0] - plain_heading,
        30.0,
        "30 above the heading {seen:?}"
    );
    assert_eq!(
        seen[3] - plain_heading,
        10.0,
        "the space function now 10 {seen:?}"
    );
    assert_eq!(seen[5], seen[0], "back to 30 {seen:?}");
}

#[test]
fn a_click_in_a_rounding_seam_is_no_click_in_a_space() {
    // At 1.125 a line is 22.5 px, so the second paragraph starts on px 23 and leaves a seam
    // under a px above it: 20 to 20.44 points down. That seam is no space: a click in it goes
    // to the line it rounds into, below, even nearer the line above, as a field by paragraph
    // does.
    let doc = TextEdit::from_parts(
        "aaaa\nbbbb\ncccc",
        vec![],
        vec![Block::Paragraph, Block::Paragraph, Block::Heading(4)],
    );
    let mut spaced = runtime_with(doc, Some(headings), None, None, 1.125);
    let x = find(&spaced, "field").rect[0] + 1.0;
    let (first, second) = (caret_at(&mut spaced, 0)[1], caret_at(&mut spaced, 5)[1]);
    let (bottom, top) = (first + 20.0, first + 23.0 / 1.125);
    assert!(
        (second - top).abs() < 0.01,
        "the second line at {second}, not {top}"
    );
    let hits: Vec<(f32, usize)> = [-0.1, 0.05, 0.1, 0.15, 0.2, 0.3, 0.4, 0.5]
        .into_iter()
        .map(|dy| (dy, click(&mut spaced, x, bottom + dy)))
        .collect();
    let want: Vec<(f32, usize)> = hits
        .iter()
        .map(|&(dy, _)| (dy, if dy < 0.0 { 0 } else { 5 }))
        .collect();
    assert_eq!(hits, want);
}

#[test]
fn a_drag_through_the_space_selects_to_the_nearer_line() {
    // The first line spans 0..20 of the field, the space 20..50, the second line from 50.
    let mut rt = runtime(TextEdit::new("abcdefgh\nabcdefgh"), Some(everywhere), None);
    let f = find(&rt, "field").rect;
    for _ in 0..40 {
        rt.frame(1.0 / 60.0);
    }
    rt.pointer_move(f[0] + 1.0, f[1] + 10.0);
    rt.pointer_down(f[0] + 1.0, f[1] + 10.0);
    rt.frame(1.0 / 60.0);
    for (dy, want) in [(30.0, 8), (45.0, 17), (60.0, 17)] {
        rt.pointer_move(f[0] + 200.0, f[1] + dy);
        rt.frame(1.0 / 60.0);
        let e = &rt.state.field;
        assert_eq!((e.anchor, e.focus), (0, want), "dragged to {dy}");
    }
    rt.pointer_up(f[0] + 200.0, f[1] + 60.0);
    rt.frame(1.0 / 60.0);
    assert_eq!((rt.state.field.anchor, rt.state.field.focus), (0, 17));
}

#[test]
fn page_keys_scroll_over_the_space() {
    // Nine lines and eight spaces, 20 + 30 each: 420 points in a 100-point scroll view.
    let doc = TextEdit::new("1\n2\n3\n4\n5\n6\n7\n8\n9");
    let (mut plain, mut spaced) = (
        runtime(doc.clone(), None, Some(100.0)),
        runtime(doc, Some(everywhere), Some(100.0)),
    );
    for rt in [&mut plain, &mut spaced] {
        rt.update(|s, _| s.field.set_selection(0, 0));
        for _ in 0..30 {
            rt.frame(1.0 / 60.0);
        }
    }
    let top = |rt: &Runtime<S>| find(rt, "viewport").rect[1] - find(rt, "field").rect[1];
    let mut pages = vec![];
    for k in [
        Key::PageDown,
        Key::PageDown,
        Key::PageDown,
        Key::PageDown,
        Key::PageDown,
        Key::PageUp,
    ] {
        for rt in [&mut plain, &mut spaced] {
            key(rt, k.clone());
            for _ in 0..30 {
                rt.frame(1.0 / 60.0);
            }
        }
        pages.push((top(&plain), top(&spaced)));
    }
    let (unset, set): (Vec<f32>, Vec<f32>) = pages.iter().copied().unzip();
    assert_eq!(
        set,
        [90.0, 180.0, 270.0, 320.0, 320.0, 230.0],
        "pages of 90 to the end, 320"
    );
    assert_eq!(unset[4], 80.0, "unset: 180 points in 100");
}

#[test]
fn a_left_to_right_field_with_right_to_left_text_gets_no_space() {
    // An English note that ends one paragraph with a Hebrew word, holds a Hebrew pill, or is
    // right to left by its element alone: each draws and answers as its one layout.
    let ends = TextEdit::from_parts(
        "Title\nThis English line ends in Hebrew שלום\nNext\nlast",
        vec![],
        vec![
            Block::Heading(1),
            Block::Paragraph,
            Block::Heading(2),
            Block::Paragraph,
        ],
    );
    let mut pill = TextEdit::from_parts(
        "see  12 ok\nnext line",
        vec![],
        vec![Block::Paragraph, Block::Heading(2)],
    );
    pill.set_selection(4, 4);
    pill.insert_pill("p", "שלום");
    let english = TextEdit::from_parts(
        "Intro\nTitle\nmore",
        vec![],
        vec![Block::Paragraph, Block::Heading(2), Block::Paragraph],
    );
    for (doc, dir) in [(ends, None), (pill, None), (english, Some(Direction::Rtl))] {
        let (mut plain, mut spaced) = (
            runtime_with(doc.clone(), None, None, dir, 1.0),
            runtime_with(doc, Some(everywhere), None, dir, 1.0),
        );
        assert_eq!(answers(&mut spaced), answers(&mut plain));
        assert_eq!(find(&spaced, "field").rect, find(&plain, "field").rect);
    }
}

#[test]
fn a_field_by_paragraph_keeps_each_paragraphs_direction_and_gets_the_space() {
    // By paragraph, each line reads its own way: a Hebrew paragraph from the right, an English
    // one from the left, and each still gets its space.
    let doc = TextEdit::from_parts(
        "שלום עולם\nTitle\nhello",
        vec![],
        vec![Block::Paragraph, Block::Heading(2), Block::Paragraph],
    );
    let by_para = |space: Option<fn(Block) -> f32>| {
        let mut field = doc.clone();
        field.focused = true;
        let mut rt = Runtime::new(
            P { field, space },
            |s: &P, _: &mut rune::Cx<P>| {
                let mut f = rune::text_input(&s.field, TextStyle::new(14.0, 400.0).lh(20.0))
                    .key("field")
                    .w(250.0)
                    .dir_by_paragraph()
                    .on_edit(|s: &mut P| &mut s.field);
                if let Some(space) = s.space {
                    f = f.block_space_above(space);
                }
                col().w_full().h_full().padding(10.0).child(f)
            },
            (300.0, 400.0),
            Env::new(true, 1.0),
        );
        rt.frame(0.0);
        let carets: Vec<[f32; 4]> = [0, 18, 24, 29]
            .into_iter()
            .map(|at| {
                rt.update(|s, _| s.field.set_selection(at, at));
                rt.frame(1.0 / 60.0);
                find_in(&rt).caret.expect("caret")
            })
            .collect();
        carets
    };
    let (plain, spaced) = (by_para(None), by_para(Some(everywhere)));
    assert!(plain[0][0] > 200.0, "the Hebrew line starts at the right");
    for (k, down) in [0.0, 30.0, 60.0, 60.0].into_iter().enumerate() {
        assert_eq!(
            [spaced[k][0], spaced[k][1] - plain[k][1]],
            [plain[k][0], down],
            "caret {k}"
        );
    }
}

struct P {
    field: TextEdit,
    space: Option<fn(Block) -> f32>,
}

fn find_in(rt: &Runtime<P>) -> ElementInfo {
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some("field"))
        .unwrap()
}
