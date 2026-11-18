//! A link in a rich field of more than one paragraph, which lays out by paragraph: it has its own
//! node in the accessibility tree, and its glyphs take the pointer.
mod support;

use rune::{Block, El, Env, Mark, MarkRange, Runtime, TextEdit, TextStyle, col};
use serde_json::Value;
use support::find;

const TEXT: &str = "First paragraph.\nRead the spec now.";
const URL: &str = "https://example.com/spec";

struct S {
    field: TextEdit,
}

fn view(s: &S) -> El {
    col().w_full().h_full().padding(10.0).child(
        rune::text_input(&s.field, TextStyle::new(14.0, 400.0).lh(20.0))
            .key("field")
            .w(300.0)
            .on_edit(|s: &mut S| &mut s.field),
    )
}

fn node<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    if v["key"] == key {
        return Some(v);
    }
    v["children"].as_array()?.iter().find_map(|c| node(c, key))
}

#[test]
fn a_link_in_the_second_paragraph_has_a_node_and_takes_the_pointer() {
    let spec = TEXT.find("spec").unwrap();
    let field = TextEdit::from_parts(
        TEXT,
        vec![MarkRange::new(spec..spec + 4, Mark::Link(URL.into()))],
        vec![Block::Paragraph, Block::Paragraph],
    );
    let mut rt = Runtime::new(S { field }, |s: &S, _: &mut rune::Cx<S>| view(s), (400.0, 200.0), Env::new(true, 2.0));
    rt.frame(0.0);

    let tree = rt.accessibility_json();
    let link = node(&tree, URL).unwrap_or_else(|| panic!("no link node: {tree}"));
    assert_eq!((link["role"].as_str(), link["name"].as_str()), (Some("link"), Some("spec")));

    // Its box sits on the second line of the field, inside it.
    let (field, b) = (find(&rt, "field"), link["bounds"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap() as f32).collect::<Vec<_>>());
    assert!(b[2] > 0.0 && b[3] > 0.0, "an empty box: {b:?}");
    assert!(b[1] >= field.rect[1] + 15.0 && b[1] + b[3] <= field.rect[1] + field.rect[3] + 0.5, "{b:?} is not on the second line of {:?}", field.rect);

    // Its centre is a link hit (`link_at`), hovered as a link; the first paragraph's words are not.
    let (x, y) = rt.find(URL).expect("the link's hit box");
    assert!((x - (b[0] + b[2] / 2.0)).abs() < 1.0 && (y - (b[1] + b[3] / 2.0)).abs() < 1.0, "({x}, {y}) vs {b:?}");
    rt.pointer_move(x, y);
    rt.frame(1.0 / 60.0);
    assert_eq!(find(&rt, URL).hover, 1.0, "the link's glyphs take the pointer");
    rt.pointer_move(field.rect[0] + 20.0, field.rect[1] + 8.0);
    rt.frame(1.0 / 60.0);
    assert_eq!(find(&rt, URL).hover, 0.0, "words of the first paragraph are no link");
}
