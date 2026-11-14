//! `Block::Heading`: Return, Backspace, level toggles, undo and HTML; and `on_input_change`
//! running on a format-only edit.
use rune::tok::TextStyle;
use rune::{
    Block, ClipboardContent, Cx, El, Env, Key, KeyEvent, Mark, Mods, Runtime, Span, TextEdit, col,
    text_input,
};

fn press(e: &mut TextEdit, key: Key) {
    e.key(
        &KeyEvent {
            key,
            mods: Mods::default(),
        },
        None,
    );
}

/// A rich two-line field: "Title" as an H1 over a paragraph, caret at the end of `Title`.
fn titled() -> TextEdit {
    let mut e = TextEdit::from_parts(
        "Title\nbody",
        vec![],
        vec![Block::Heading(1), Block::Paragraph],
    );
    e.set_selection(5, 5);
    e
}

#[test]
fn return_at_the_end_of_a_heading_starts_a_paragraph() {
    let mut e = titled();
    press(&mut e, Key::Enter);
    assert_eq!(e.text, "Title\n\nbody");
    assert_eq!(
        e.blocks(),
        [Block::Heading(1), Block::Paragraph, Block::Paragraph]
    );
}

#[test]
fn return_inside_a_heading_keeps_both_halves_headings() {
    let mut e = titled();
    e.set_selection(2, 2);
    press(&mut e, Key::Enter);
    assert_eq!(e.text, "Ti\ntle\nbody");
    assert_eq!(
        e.blocks(),
        [Block::Heading(1), Block::Heading(1), Block::Paragraph]
    );
}

#[test]
fn backspace_at_the_start_of_a_heading_makes_it_a_paragraph() {
    let mut e = titled();
    e.set_selection(0, 0);
    press(&mut e, Key::Backspace);
    assert_eq!(e.text, "Title\nbody", "no text is deleted");
    assert_eq!(e.blocks(), [Block::Paragraph, Block::Paragraph]);
}

#[test]
fn toggling_a_heading_level_switches_it_then_clears_it() {
    let mut e = titled();
    e.toggle_block(Block::Heading(2));
    assert_eq!(
        e.blocks()[0],
        Block::Heading(2),
        "an H1 toggled to H2 becomes an H2"
    );
    e.toggle_block(Block::Heading(2));
    assert_eq!(
        e.blocks()[0],
        Block::Paragraph,
        "the same level toggles off"
    );
    e.toggle_block(Block::Heading(9));
    assert_eq!(e.blocks()[0], Block::Heading(6), "levels clamp to 1..=6");
    assert!(
        Block::Heading(1).same_kind(Block::Heading(1))
            && !Block::Heading(1).same_kind(Block::Heading(2))
    );
}

#[test]
fn undo_and_redo_restore_a_heading() {
    let mut e = titled();
    e.set_selection(7, 7);
    e.set_block(Block::Heading(3));
    assert_eq!(e.blocks(), [Block::Heading(1), Block::Heading(3)]);
    e.undo();
    assert_eq!(e.blocks(), [Block::Heading(1), Block::Paragraph]);
    e.redo();
    assert_eq!(e.blocks(), [Block::Heading(1), Block::Heading(3)]);

    // A deleted heading line comes back as a heading.
    let mut e = titled();
    e.set_selection(0, 6);
    press(&mut e, Key::Backspace);
    assert_eq!(e.text, "body");
    e.undo();
    assert_eq!(
        (e.text.as_str(), e.blocks()),
        ("Title\nbody", &[Block::Heading(1), Block::Paragraph][..])
    );
}

#[test]
fn html_headings_round_trip_without_a_bold_mark() {
    let e = TextEdit::from_html("<h1>Big</h1><h3>Small <em>it</em></h3><p>text</p>");
    assert_eq!(e.text, "Big\nSmall it\ntext");
    assert_eq!(
        e.blocks(),
        [Block::Heading(1), Block::Heading(3), Block::Paragraph]
    );
    assert!(
        e.marks().iter().all(|m| m.mark != Mark::Bold),
        "{:?}",
        e.marks()
    );
    let html = e.to_html();
    assert_eq!(html, "<h1>Big</h1><h3>Small <em>it</em></h3><p>text</p>");
    let back = TextEdit::from_html(&html);
    assert_eq!(
        (back.text.as_str(), back.blocks(), back.marks()),
        (e.text.as_str(), e.blocks(), e.marks())
    );

    // A heading inside a list item can't be a heading line, so it stays a bold list item.
    let li = TextEdit::from_html("<ul><li><h2>Item</h2></li></ul>");
    assert_eq!(li.blocks(), [Block::Bullet(0)]);
    assert!(li.marks().iter().any(|m| m.mark == Mark::Bold));
}

#[test]
fn pasting_an_h2_lands_a_heading_line() {
    let mut e = TextEdit::from_parts("one\n", vec![], vec![Block::Paragraph, Block::Paragraph]);
    e.set_selection(4, 4);
    e.paste_html("<h2>Pasted</h2>", "Pasted");
    assert_eq!(e.text, "one\nPasted");
    assert_eq!(e.blocks(), [Block::Paragraph, Block::Heading(2)]);
}

#[derive(Default)]
struct S {
    field: TextEdit,
    changes: u32,
}

fn view(s: &S, _: &mut Cx<S>) -> El {
    col().size(300.0, 200.0).child(
        text_input(&s.field, TextStyle::new(14.0, 400.0))
            .key("field")
            .size(250.0, 120.0)
            .on_edit(|s: &mut S| &mut s.field)
            .on_input_change(|s: &mut S, _| s.changes += 1),
    )
}

fn field_runtime() -> Runtime<S> {
    let mut field = TextEdit::from_parts("hello world", vec![], vec![Block::Paragraph]);
    field.focused = true;
    let mut rt = Runtime::new(
        S { field, changes: 0 },
        view,
        (300.0, 200.0),
        Env::new(true, 1.0),
    );
    rt.frame(0.0);
    rt
}

fn send(rt: &mut Runtime<S>, key: Key, mods: Mods) {
    rt.key(KeyEvent { key, mods });
    rt.frame(1.0 / 60.0);
}

#[test]
fn a_format_only_edit_calls_on_input_change() {
    let mut rt = field_runtime();
    rt.state.field.set_selection(0, 5);
    send(&mut rt, Key::Command('b'), Mods::default());
    assert!(
        rt.state.field.has_mark(&Mark::Bold),
        "⌘B bolds the selection"
    );
    assert_eq!(rt.state.field.text, "hello world");
    assert_eq!(
        rt.state.changes, 1,
        "bold with no text change calls the hook once"
    );

    send(&mut rt, Key::Command('z'), Mods::default());
    assert!(rt.state.field.marks().is_empty(), "undo removes the bold");
    assert_eq!(
        rt.state.changes, 2,
        "undo of a format-only edit calls it again"
    );

    send(
        &mut rt,
        Key::Command('z'),
        Mods {
            shift: true,
            ..Mods::default()
        },
    );
    assert!(
        !rt.state.field.marks().is_empty(),
        "redo brings the bold back"
    );
    assert_eq!(rt.state.changes, 3, "and so does redo");
}

#[test]
fn tab_indent_and_backspace_on_a_list_item_call_on_input_change() {
    let mut rt = field_runtime();
    rt.state.field.toggle_block(Block::Bullet(0));
    rt.invalidate();
    rt.frame(1.0 / 60.0);
    assert_eq!(
        rt.state.changes, 0,
        "an app's own edit outside a key isn't the field's change"
    );
    send(&mut rt, Key::Tab, Mods::default());
    assert_eq!(rt.state.field.blocks(), [Block::Bullet(1)]);
    assert_eq!(rt.state.changes, 1, "Tab indents with no text change");
    rt.state.field.set_selection(0, 0);
    send(&mut rt, Key::Backspace, Mods::default());
    assert_eq!(
        (rt.state.field.text.as_str(), rt.state.field.blocks()),
        ("hello world", &[Block::Bullet(0)][..])
    );
    assert_eq!(
        rt.state.changes, 2,
        "Backspace at a list item's start outdents with no text change"
    );
}

#[test]
fn a_rich_paste_lands_a_heading_and_calls_on_input_change() {
    let mut rt = field_runtime();
    send(
        &mut rt,
        Key::Enter,
        Mods {
            shift: true,
            alt: true,
            ..Mods::default()
        },
    );
    assert_eq!(rt.state.changes, 1);
    rt.paste_content(ClipboardContent::new("Pasted").with_html("<h2>Pasted</h2>"));
    rt.frame(2.0 / 60.0);
    assert_eq!(rt.state.field.text, "hello world\nPasted");
    assert_eq!(
        rt.state.field.blocks(),
        [Block::Paragraph, Block::Heading(2)]
    );
    assert_eq!(rt.state.changes, 2);
}

#[test]
fn a_caret_move_alone_does_not_call_on_input_change() {
    let mut rt = field_runtime();
    send(&mut rt, Key::Left, Mods::default());
    send(
        &mut rt,
        Key::Left,
        Mods {
            shift: true,
            ..Mods::default()
        },
    );
    send(&mut rt, Key::Home, Mods::default());
    assert_eq!(rt.state.changes, 0);
    send(&mut rt, Key::Text("!".into()), Mods::default());
    assert_eq!(rt.state.changes, 1, "typing still calls it");
}

#[derive(Default)]
struct Look {
    field: TextEdit,
    decorate: bool,
}

fn look(s: &Look, _: &mut Cx<Look>) -> El {
    let deco = if s.decorate {
        vec![Span::new(0, 5).size(14.0)]
    } else {
        Vec::new()
    };
    col().size(300.0, 200.0).child(
        text_input(&s.field, TextStyle::new(14.0, 400.0))
            .key("field")
            .size(250.0, 150.0)
            .on_edit(|s: &mut Look| &mut s.field)
            .decorations(deco),
    )
}

/// The drawn caret's height at byte `at` of "Title\nbody" (an H1 over a paragraph).
fn caret_height(decorate: bool, at: usize) -> f32 {
    let mut field = titled();
    field.set_selection(at, at);
    field.focused = true;
    let mut rt = Runtime::new(
        Look { field, decorate },
        look,
        (300.0, 200.0),
        Env::new(true, 1.0),
    );
    rt.frame(0.0);
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some("field"))
        .and_then(|e| e.caret)
        .expect("a caret")[3]
}

#[test]
fn a_heading_draws_larger_unless_the_app_decorates_it() {
    let (heading, body) = (caret_height(false, 3), caret_height(false, 8));
    assert!(
        heading > body * 1.4,
        "an H1 line draws at 1.6x: caret {heading} vs {body}"
    );
    let decorated = caret_height(true, 3);
    assert!(
        (decorated - body).abs() < 0.5,
        "a decoration's size wins: caret {decorated} vs {body}"
    );
}

#[test]
fn return_at_the_start_of_a_heading_leaves_a_paragraph_above_in_one_undo_step() {
    let mut e = titled();
    e.set_selection(0, 0);
    press(&mut e, Key::Enter);
    assert_eq!(e.text, "\nTitle\nbody");
    assert_eq!(
        e.blocks(),
        [Block::Paragraph, Block::Heading(1), Block::Paragraph]
    );
    assert_eq!(e.range(), (1, 1), "the caret stays on the heading");
    e.undo();
    assert_eq!(
        (e.text.as_str(), e.blocks()),
        ("Title\nbody", &[Block::Heading(1), Block::Paragraph][..])
    );
}

#[test]
fn return_before_a_soft_break_in_a_heading_keeps_both_halves_headings() {
    let mut e = TextEdit::from_parts("Title", vec![], vec![Block::Heading(1)]);
    e.set_selection(2, 2);
    e.key(
        &KeyEvent {
            key: Key::Enter,
            mods: Mods {
                shift: true,
                ..Mods::default()
            },
        },
        None,
    );
    assert_eq!((e.text.as_str(), e.soft_breaks()), ("Ti\ntle", &[1][..]));
    let soft = e.clone();

    // "Ti" ends a `\n` line but not the heading, which goes on after the soft break.
    e.set_selection(2, 2);
    press(&mut e, Key::Enter);
    assert_eq!(e.text, "Ti\n\ntle");
    assert_eq!(e.blocks(), [Block::Heading(1); 3]);
    assert_eq!(e.soft_breaks(), [2]);

    // The end of the soft-broken heading is its end.
    let mut e = soft;
    e.set_selection(6, 6);
    press(&mut e, Key::Enter);
    assert_eq!(e.text, "Ti\ntle\n");
    assert_eq!(
        e.blocks(),
        [Block::Heading(1), Block::Heading(1), Block::Paragraph]
    );
}

#[test]
fn a_field_without_headings_pastes_a_heading_as_a_bold_paragraph() {
    let mut e =
        TextEdit::from_parts("a", vec![], vec![Block::Paragraph]).allow_blocks([Block::Bullet(0)]);
    e.set_selection(1, 1);
    e.paste_html("<h2>X</h2>", "X");
    assert_eq!(e.text, "aX");
    assert_eq!(e.blocks(), [Block::Paragraph]);
    assert_eq!(
        e.marks(),
        [rune::MarkRange {
            start: 1,
            end: 2,
            mark: Mark::Bold
        }]
    );

    // Heading levels don't matter, as list levels don't.
    let mut e =
        TextEdit::from_parts("a", vec![], vec![Block::Paragraph]).allow_blocks([Block::Heading(2)]);
    e.toggle_block(Block::Heading(4));
    assert_eq!(e.blocks(), [Block::Heading(4)]);
}

#[test]
fn a_caret_key_after_the_app_writes_text_does_not_call_on_input_change() {
    // A rich field whose app adds a line to `text`: its blocks are one short until synced.
    let mut rt = field_runtime();
    rt.state.field.text.push_str("\nmore");
    rt.invalidate();
    rt.frame(1.0 / 60.0);
    send(&mut rt, Key::Left, Mods::default());
    assert_eq!(rt.state.changes, 0, "a rich field's caret key");

    // A plain field whose app writes two lines over one.
    let mut rt = field_runtime();
    rt.state.field = TextEdit::new("one");
    rt.state.field.focused = true;
    rt.invalidate();
    rt.frame(1.0 / 60.0);
    rt.state.field.text = "one\ntwo".into();
    rt.invalidate();
    rt.frame(2.0 / 60.0);
    send(&mut rt, Key::Right, Mods::default());
    assert_eq!(rt.state.changes, 0, "a plain field's caret key");
    send(&mut rt, Key::Text("!".into()), Mods::default());
    assert_eq!(rt.state.changes, 1, "typing still calls it");
}
