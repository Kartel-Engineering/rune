//! `El::blank_line_height`: an empty line between two lines with words takes part of a line.
//! The caret, clicks, Up and Down, a selection and the field's height follow, and typing in the
//! short line gives it its full line.
use rune::runtime::ElementInfo;
use rune::{
    Block, Color, Direction, El, Env, Ime, Key, KeyEvent, Runtime, TextEdit, TextStyle, col,
};

type Height = fn(Block, Block) -> f32;

struct S {
    field: TextEdit,
    blank: Option<Height>,
    dir: Option<Direction>,
}

fn caret_ink() -> Color {
    rune::color::hex("#ff0077").unwrap()
}
fn selection_ink() -> Color {
    rune::color::hex("#00ff55").unwrap()
}

fn half(_: Block, _: Block) -> f32 {
    0.5
}
/// Half a line between two paragraphs, a full one beside a heading.
fn by_neighbour(above: Block, below: Block) -> f32 {
    if matches!(above, Block::Heading(_)) || matches!(below, Block::Heading(_)) {
        1.0
    } else {
        0.5
    }
}

fn view(s: &S) -> El {
    let mut field = rune::text_input(&s.field, TextStyle::new(14.0, 400.0).lh(20.0))
        .key("field")
        .w(250.0)
        .caret(caret_ink(), selection_ink())
        .on_edit(|s: &mut S| &mut s.field);
    if let Some(blank) = s.blank {
        field = field.blank_line_height(blank);
    }
    let page = col().w_full().h_full().padding(10.0).child(field);
    match s.dir {
        Some(dir) => page.dir(dir),
        None => page,
    }
}

fn runtime(field: TextEdit, blank: Option<Height>) -> Runtime<S> {
    runtime_with(field, blank, None, 1.0)
}

fn runtime_with(
    mut field: TextEdit,
    blank: Option<Height>,
    dir: Option<Direction>,
    scale: f32,
) -> Runtime<S> {
    field.focused = true;
    let mut rt = Runtime::new(
        S { field, blank, dir },
        |s: &S, _: &mut rune::Cx<S>| view(s),
        (300.0, 400.0),
        Env::new(true, scale),
    );
    rt.frame(0.0);
    rt
}

fn find(rt: &Runtime<S>) -> ElementInfo {
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some("field"))
        .unwrap()
}

/// The caret line at byte `at`, `[x, y, w, h]` in window points.
fn caret_at(rt: &mut Runtime<S>, at: usize) -> [f32; 4] {
    rt.update(|s, _| s.field.set_selection(at, at));
    rt.frame(1.0 / 60.0);
    find(rt).caret.expect("caret")
}

/// The caret's top at each byte, from the field's top.
fn tops(rt: &mut Runtime<S>, at: &[usize]) -> Vec<f32> {
    let top = find(rt).rect[1];
    at.iter().map(|&b| caret_at(rt, b)[1] - top).collect()
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

/// Every caret, Up and Down from each place, and what a whole selection draws.
type Answers = (
    Vec<[f32; 4]>,
    Vec<[usize; 2]>,
    Vec<([f32; 4], [f32; 4])>,
    [f32; 4],
);

fn answers(rt: &mut Runtime<S>) -> Answers {
    let text = rt.state.field.text.clone();
    let places: Vec<usize> = (0..=text.len())
        .filter(|&i| text.is_char_boundary(i))
        .collect();
    let carets = places.iter().map(|&at| caret_at(rt, at)).collect();
    let mut moves = Vec::new();
    for &at in &places {
        let mut to = [0; 2];
        for (j, k) in [Key::Up, Key::Down].into_iter().enumerate() {
            rt.update(|s, _| s.field.set_selection(at, at));
            rt.frame(1.0 / 60.0);
            key(rt, k);
            to[j] = rt.state.field.focus;
        }
        moves.push(to);
    }
    rt.update(|s, _| s.field.set_selection(0, text.len()));
    rt.frame(1.0 / 60.0);
    (
        carets,
        moves,
        rt.instances.iter().map(|i| (i.rect, i.fill)).collect(),
        find(rt).rect,
    )
}

/// "abc", a blank line, "def", two blank lines, "ghi": each line's first byte.
fn note() -> (TextEdit, [usize; 6]) {
    (TextEdit::new("abc\n\ndef\n\n\nghi"), [0, 4, 5, 9, 10, 11])
}

#[test]
fn a_blank_line_between_two_paragraphs_takes_half_a_line() {
    let (doc, starts) = note();
    let (mut plain, mut short) = (runtime(doc.clone(), None), runtime(doc, Some(half)));
    assert_eq!(
        tops(&mut plain, &starts),
        [0.0, 20.0, 40.0, 60.0, 80.0, 100.0]
    );
    // A run of two blank lines keeps its full lines.
    assert_eq!(
        tops(&mut short, &starts),
        [0.0, 20.0, 30.0, 50.0, 70.0, 90.0]
    );
    assert_eq!(
        find(&plain).rect[3] - find(&short).rect[3],
        10.0,
        "the field is 10 shorter"
    );
    let c = caret_at(&mut short, 4);
    assert_eq!(c[3], 10.0, "the caret is as tall as its short line: {c:?}");
    let ime = short.ime_area.expect("IME area");
    assert!(
        (ime[3] - c[3]).abs() < 0.01,
        "IME area {ime:?} on the caret {c:?}"
    );
}

#[test]
fn unset_one_and_nan_draw_exactly_as_before() {
    let doc = TextEdit::from_parts(
        "abc\n\ndef\n\nTitle\nend",
        vec![],
        vec![
            Block::Paragraph,
            Block::Paragraph,
            Block::Paragraph,
            Block::Paragraph,
            Block::Heading(2),
            Block::Paragraph,
        ],
    );
    for scale in [1.0, 1.125] {
        let mut plain = runtime_with(doc.clone(), None, None, scale);
        let want = answers(&mut plain);
        for blank in [(|_, _| 1.0) as Height, |_, _| f32::NAN] {
            assert_eq!(
                answers(&mut runtime_with(doc.clone(), Some(blank), None, scale)),
                want,
                "scale {scale}"
            );
        }
    }
}

#[test]
fn a_blank_line_beside_a_heading_keeps_what_the_app_gives_it() {
    let doc = TextEdit::from_parts(
        "abc\n\ndef\n\nTitle\n\nend",
        vec![],
        vec![
            Block::Paragraph,
            Block::Paragraph,
            Block::Paragraph,
            Block::Paragraph,
            Block::Heading(2),
            Block::Paragraph,
            Block::Paragraph,
        ],
    );
    let (mut plain, mut short) = (runtime(doc.clone(), None), runtime(doc, Some(by_neighbour)));
    let starts = [0, 4, 5, 9, 10, 16, 17];
    let (p, s) = (tops(&mut plain, &starts), tops(&mut short, &starts));
    let moved: Vec<f32> = p.iter().zip(&s).map(|(p, s)| p - s).collect();
    assert_eq!(
        moved,
        [0.0, 0.0, 10.0, 10.0, 10.0, 10.0, 10.0],
        "only the blank between two paragraphs is short"
    );
}

#[test]
fn empty_list_items_soft_breaks_and_edges_keep_their_lines() {
    // An empty bullet between two, a blank line a Shift+Return break ends, and blank first and
    // last lines.
    let bullets = TextEdit::from_parts("one\n\ntwo", vec![], vec![Block::Bullet(0); 3]);
    // A plain field draws as plain text, soft breaks and all: a heading makes it rich.
    let soft = TextEdit::from_parts(
        "T\nabc\n\ndef",
        vec![],
        vec![
            Block::Heading(2),
            Block::Paragraph,
            Block::Paragraph,
            Block::Paragraph,
        ],
    )
    .with_soft_breaks(vec![3]);
    let edges = TextEdit::new("\nabc\n");
    for doc in [bullets, soft, edges] {
        let mut plain = runtime(doc.clone(), None);
        assert_eq!(
            answers(&mut runtime(doc.clone(), Some(half))),
            answers(&mut plain),
            "{:?}",
            doc.text
        );
    }
}

#[test]
fn a_click_in_the_short_line_lands_in_it() {
    let (doc, _) = note();
    let mut rt = runtime(doc, Some(half));
    let f = find(&rt).rect;
    // abc 0..20, the short line 20..30, def from 30.
    assert_eq!(
        click(&mut rt, f[0] + 1.0, f[1] + 25.0),
        4,
        "in the short line"
    );
    assert_eq!(
        click(&mut rt, f[0] + 1.0, f[1] + 33.0),
        5,
        "on def below it"
    );
    assert_eq!(
        click(&mut rt, f[0] + 1.0, f[1] + 18.0),
        0,
        "on abc above it"
    );
}

#[test]
fn up_and_down_step_through_the_short_line() {
    let (doc, _) = note();
    let mut rt = runtime(doc, Some(half));
    caret_at(&mut rt, 2);
    key(&mut rt, Key::Down);
    assert_eq!(rt.state.field.focus, 4, "down into the short line");
    key(&mut rt, Key::Down);
    assert_eq!(rt.state.field.focus, 7, "down to def, the column kept");
    key(&mut rt, Key::Up);
    key(&mut rt, Key::Up);
    assert_eq!(rt.state.field.focus, 2, "back up to abc");
}

#[test]
fn a_selection_box_on_the_short_line_stays_in_it() {
    let (doc, _) = note();
    let mut rt = runtime(doc, Some(half));
    rt.update(|s, _| s.field.set_selection(0, 8));
    rt.frame(1.0 / 60.0);
    let top = find(&rt).rect[1];
    let want = selection_ink().premul(1.0);
    let boxes: Vec<[f32; 4]> = rt
        .instances
        .iter()
        .filter(|i| i.fill.iter().zip(want).all(|(a, b)| (a - b).abs() < 1e-3))
        .map(|i| i.rect)
        .collect();
    assert!(boxes.len() >= 3, "a box on each line: {boxes:?}");
    let blank: Vec<_> = boxes
        .iter()
        .filter(|b| b[1] - top >= 19.99 && b[1] - top < 29.99)
        .collect();
    assert!(
        !blank.is_empty() && blank.iter().all(|b| b[1] + b[3] - top <= 30.01),
        "the short line's box is inside 20..30: {boxes:?}"
    );
}

#[test]
fn typing_in_the_short_line_gives_it_a_full_line_at_once() {
    let (doc, _) = note();
    let (mut plain, mut short) = (runtime(doc.clone(), None), runtime(doc, Some(half)));
    for rt in [&mut plain, &mut short] {
        caret_at(rt, 4);
        rt.ime(Ime::Commit("x".into()));
        rt.frame(1.0 / 60.0);
    }
    let starts = [0, 4, 6, 10, 11, 12];
    assert_eq!(tops(&mut short, &starts), tops(&mut plain, &starts));
    assert_eq!(find(&short).rect, find(&plain).rect);
    // Deleting the letter makes it short again.
    caret_at(&mut short, 5);
    key(&mut short, Key::Backspace);
    assert_eq!(tops(&mut short, &[5]), [30.0]);
}

#[test]
fn a_right_to_left_field_keeps_its_blank_lines() {
    let doc = TextEdit::new("שלום\n\nעולם");
    for dir in [None, Some(Direction::Rtl)] {
        let mut plain = runtime_with(doc.clone(), None, dir, 1.0);
        assert_eq!(
            answers(&mut runtime_with(doc.clone(), Some(half), dir, 1.0)),
            answers(&mut plain),
            "dir {dir:?}"
        );
    }
}

#[test]
fn heights_out_of_range_are_clamped() {
    let (doc, _) = note();
    for (blank, want) in [
        ((|_, _| 0.0) as Height, 2.0),
        (|_, _| -3.0, 2.0),
        (|_, _| 50.0, 200.0),
        (|_, _| f32::INFINITY, 200.0),
    ] {
        let mut rt = runtime(doc.clone(), Some(blank));
        assert_eq!(
            tops(&mut rt, &[5])[0] - 20.0,
            want,
            "the short line's height for {}",
            blank(Block::Paragraph, Block::Paragraph)
        );
    }
}
