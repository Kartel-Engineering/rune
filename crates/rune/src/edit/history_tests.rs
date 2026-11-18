use super::*;

#[test]
fn a_group_opened_on_a_secure_field_records_from_when_it_turns_plain() {
    let mut e = TextEdit::new("pw").secure();
    e.undo_group(|e| {
        e.insert("1");
        assert!(matches!(e.group, Group::Unrecorded));
        e.set_secure(false);
        e.insert("a");
        e.set_secure(false);
        e.insert("b");
    });
    assert_eq!(e.text, "pw1ab");
    e.undo();
    assert_eq!(e.text, "pw1", "one step, from when the field turned plain, not moved by a second set_secure(false)");
    e.undo();
    assert_eq!(e.text, "pw1");
}

#[test]
fn a_group_is_closed_in_clones_and_holds_undo_until_it_closes() {
    let mut e = TextEdit::new("a");
    e.undo_group(|e| {
        e.insert("b");
        assert!(e.group.is_open() && !e.clone().group.is_open());
        e.undo();
        assert_eq!(e.text, "ab", "undo waits for the group to close");
        e.set_secure(true);
        assert!(matches!(e.group, Group::Unrecorded), "turning secure drops the baseline's text");
        e.set_secure(false);
        e.replace_text("c");
    });
    assert!(!e.group.is_open());
    e.undo();
    assert_eq!(e.text, "c", "replace_text in the group discards the history before it");
}

#[test]
fn typing_through_the_page_field_is_one_step_and_redo_brings_it_all_back() {
    let mut e = TextEdit::new("a");
    e.set_selection(1, 1);
    // What the page's field reports for each key: the bytes it replaced, as a selection, and
    // its `inputType`.
    let type_in = |e: &mut TextEdit, text: &str, input: &str| {
        let was = (e.anchor, e.focus);
        e.browser_input(text, input, was);
    };
    for c in [" ", "q", "u", "i", "e", "t"] { type_in(&mut e, c, "insertText"); }
    type_in(&mut e, "\n", "insertText");
    for c in ["-", " ", "t", "e", "a"] { type_in(&mut e, c, "insertText"); }
    assert_eq!(e.text, "a quiet\n- tea");
    assert_eq!(e.history_len(), (1, 0), "a typed run is one step, line break included");
    // Backspace from a caret: the field reports the character before it as the range.
    for _ in 0..2 {
        let (a, _) = e.range();
        e.set_selection(a - 1, a);
        e.browser_input("", "deleteContentBackward", (a, a));
    }
    assert_eq!(e.text, "a quiet\n- t");
    assert_eq!(e.history_len(), (2, 0), "deletions from a caret are one more step");
    type_in(&mut e, "\u{2019}", "insertReplacementText");
    assert_eq!(e.history_len(), (3, 0), "a spelling fix is a step of its own");
    for _ in 0..3 { e.undo(); }
    assert_eq!(e.text, "a");
    for _ in 0..3 { e.redo(); }
    assert_eq!(e.text, "a quiet\n- t\u{2019}");
}
