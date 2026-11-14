//! Safe HTML subset import and export for [`TextEdit`].
//!
//! Supported elements:
//! - Inline: `<b>`, `<strong>`, `<i>`, `<em>`, `<u>`, `<ins>`, `<s>`, `<del>`, `<strike>`,
//!   `<a href="...">`, `<code>`, and CSS `style` attributes (`font-weight`, `font-style`,
//!   `text-decoration`, `text-decoration-line`), and `<span data-pill="id">` for
//!   [`Mark::Pill`].
//! - Block: `<p>`, `<div>`, `<br>`, `<ul>`, `<ol>`, `<li>`, `<blockquote>`, `<pre>`, and
//!   `<h1>`…`<h6>` as [`Block::Heading`].
//! - Stripped with content: `<script>`, `<style>`, `<head>`.
//! - Unknown tags drop their wrapper and keep their text. Nothing is executed, and dangerous
//!   link schemes (`javascript:`, `vbscript:`, `data:`) are rejected.

use super::{Block, MAX_LIST_LEVEL, Mark, MarkRange, TextEdit};

mod serialize;
pub(crate) use serialize::serialize_html;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ListKind {
    Bullet,
    Numbered,
}

#[derive(Clone)]
struct ActiveMark {
    mark: Mark,
    start: usize,
}

/// Safe URL check for `<a href="...">`: allows `http:`, `https:`, `mailto:` and `tel:`, and
/// scheme-less URLs. Browsers drop ASCII tabs and newlines anywhere in a URL and leading
/// controls and spaces, so `java\tscript:` is checked as `javascript:`; anything else with a
/// scheme (`javascript:`, `data:`, `file:`, app-specific schemes) is rejected.
fn is_safe_href(url: &str) -> bool {
    let cleaned: String = url
        .chars()
        .filter(|c| !c.is_control() && !c.is_whitespace())
        .take(64)
        .map(|c| c.to_ascii_lowercase())
        .collect();
    if cleaned.is_empty() {
        return false;
    }
    let scheme_len = cleaned
        .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.')))
        .unwrap_or(cleaned.len());
    if !cleaned[scheme_len..].starts_with(':') {
        // No scheme: a relative path, `www.example.com` or `#fragment`.
        return true;
    }
    matches!(&cleaned[..scheme_len], "http" | "https" | "mailto" | "tel")
}

/// Decodes HTML entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&nbsp;`, `&#123;`, `&#x1F600;`).
pub(crate) fn decode_entities(input: &str) -> String {
    if !input.contains('&') {
        return input.to_owned();
    }
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp + 1..];
        // Entities are short: look for `;` in the next 33 bytes only, so a run of bare `&`s
        // stays linear.
        let window = &rest.as_bytes()[..rest.len().min(33)];
        let Some(semi) = window.iter().position(|&b| b == b';') else {
            out.push('&');
            continue;
        };
        let entity = &rest[..semi];
        if entity.is_empty() || entity.contains(char::is_whitespace) {
            out.push('&');
            continue;
        }
        let decoded = if let Some(hex) = entity.strip_prefix("#x").or_else(|| entity.strip_prefix("#X")) {
            u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
        } else if let Some(dec) = entity.strip_prefix('#') {
            dec.parse::<u32>().ok().and_then(char::from_u32)
        } else {
            match entity {
                "amp" | "AMP" => Some('&'),
                "lt" | "LT" => Some('<'),
                "gt" | "GT" => Some('>'),
                "quot" | "QUOT" => Some('"'),
                "apos" | "#39" => Some('\''),
                "nbsp" | "NBSP" => Some('\u{00a0}'),
                "copy" => Some('©'),
                "mdash" => Some('—'),
                "ndash" => Some('–'),
                "hellip" => Some('…'),
                "lsquo" => Some('‘'),
                "rsquo" => Some('’'),
                "ldquo" => Some('“'),
                "rdquo" => Some('”'),
                "bull" => Some('•'),
                // Browsers decode these in an `href` too, so `javascript&colon;` is still a scheme.
                "colon" => Some(':'),
                "Tab" => Some('\t'),
                "NewLine" => Some('\n'),
                _ => None,
            }
        };
        if let Some(ch) = decoded {
            out.push(ch);
            rest = &rest[semi + 1..];
        } else {
            out.push('&');
        }
    }
    out.push_str(rest);
    out
}

/// Extracts an HTML attribute's value from a tag tail (`href="..."`, `style='...'`, `href=foo`).
fn extract_attr(attrs: &str, name: &str) -> Option<String> {
    let bytes = attrs.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let key_start = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'-' | b'_' | b':')) {
            i += 1;
        }
        if i == key_start {
            i += 1;
            continue;
        }
        let key = &attrs[key_start..i];
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b'=' {
            continue;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let val = if bytes[i] == b'"' || bytes[i] == b'\'' {
            let quote = bytes[i];
            i += 1;
            let val_start = i;
            while i < bytes.len() && bytes[i] != quote {
                i += 1;
            }
            let v = &attrs[val_start..i];
            if i < bytes.len() {
                i += 1;
            }
            v
        } else {
            let val_start = i;
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' && bytes[i] != b'/' {
                i += 1;
            }
            &attrs[val_start..i]
        };
        if key.eq_ignore_ascii_case(name) {
            return Some(decode_entities(val));
        }
    }
    None
}

/// Parses inline CSS `style="..."` for `font-weight`, `font-style`, and `text-decoration`.
fn parse_style_marks(style: &str, add: &mut Vec<Mark>, remove: &mut Vec<Mark>) {
    for decl in style.split(';') {
        let Some((prop, val)) = decl.split_once(':') else { continue };
        let prop = prop.trim();
        let val = val.trim();
        if prop.eq_ignore_ascii_case("font-weight") {
            let v = val.split_whitespace().next().unwrap_or("");
            if v.eq_ignore_ascii_case("bold")
                || v.eq_ignore_ascii_case("bolder")
                || v.parse::<f32>().is_ok_and(|w| w >= 600.0)
            {
                add.push(Mark::Bold);
            } else if v.eq_ignore_ascii_case("normal")
                || v.eq_ignore_ascii_case("lighter")
                || v.parse::<f32>().is_ok_and(|w| w < 600.0)
            {
                remove.push(Mark::Bold);
            }
        } else if prop.eq_ignore_ascii_case("font-style") {
            let v = val.split_whitespace().next().unwrap_or("");
            if v.eq_ignore_ascii_case("italic") || v.eq_ignore_ascii_case("oblique") {
                add.push(Mark::Italic);
            } else if v.eq_ignore_ascii_case("normal") {
                remove.push(Mark::Italic);
            }
        } else if prop.eq_ignore_ascii_case("text-decoration")
            || prop.eq_ignore_ascii_case("text-decoration-line")
        {
            for tok in val.split_whitespace() {
                if tok.eq_ignore_ascii_case("underline") {
                    add.push(Mark::Underline);
                } else if tok.eq_ignore_ascii_case("line-through") {
                    add.push(Mark::Strike);
                } else if tok.eq_ignore_ascii_case("none") {
                    remove.push(Mark::Underline);
                    remove.push(Mark::Strike);
                }
            }
        }
    }
}

const MAX_DEPTH: usize = 512;

struct TagFrame {
    tag: String,
    added_marks: Vec<Mark>,
    suppressed_marks: Vec<Mark>,
}

pub(crate) fn parse_html(html: &str) -> TextEdit {
    let mut text = String::new();
    let mut marks: Vec<MarkRange> = Vec::new();
    let mut blocks: Vec<Block> = vec![Block::Paragraph];
    let mut soft_breaks: Vec<u32> = Vec::new();

    let mut stack: Vec<TagFrame> = Vec::new();
    // Open frames per tag name, so a stray closing tag never scans the whole stack.
    let mut open_tags: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut active_marks: Vec<ActiveMark> = Vec::new();
    let mut list_stack: Vec<ListKind> = Vec::new();
    let mut in_pre: usize = 0;
    let mut in_quote: usize = 0;
    let mut in_li: usize = 0;
    // Whether the current line has been claimed by a block tag or text, whether that block is
    // still open awaiting its first content, and whether text/br has been written on this line.
    let mut line_has_block = false;
    let mut line_open = false;
    let mut line_has_content = false;

    let current_default_block = |list_stack: &[ListKind], in_pre: usize, in_quote: usize, in_li: usize| -> Block {
        if in_pre > 0 {
            Block::Code
        } else if in_li > 0 && let Some(&kind) = list_stack.last() {
            let level = (list_stack.len() - 1).min(MAX_LIST_LEVEL as usize) as u8;
            match kind {
                ListKind::Bullet => Block::Bullet(level),
                ListKind::Numbered => Block::Numbered(level),
            }
        } else if in_quote > 0 {
            Block::Quote
        } else {
            Block::Paragraph
        }
    };

    let flush_active_marks = |active: &mut [ActiveMark], marks: &mut Vec<MarkRange>, pos: usize, in_pre: usize| {
        if in_pre == 0 {
            for a in active.iter_mut() {
                if a.start < pos {
                    marks.push(MarkRange {
                        start: a.start,
                        end: pos,
                        mark: a.mark.clone(),
                    });
                }
                a.start = pos;
            }
        } else {
            for a in active.iter_mut() {
                a.start = pos;
            }
        }
    };

    let start_new_paragraph = |text: &mut String,
                               blocks: &mut Vec<Block>,
                               active: &mut [ActiveMark],
                               marks: &mut Vec<MarkRange>,
                               line_has_block: &mut bool,
                               line_open: &mut bool,
                               line_has_content: &mut bool,
                               block: Block,
                               in_pre: usize| {
        if (!*line_has_block || *line_open) && !*line_has_content {
            if let Some(last) = blocks.last_mut() {
                *last = block;
            }
            *line_has_block = true;
            *line_open = true;
            return;
        }
        let pos = text.len();
        flush_active_marks(active, marks, pos, in_pre);
        text.push('\n');
        blocks.push(block);
        let new_pos = text.len();
        for a in active.iter_mut() {
            a.start = new_pos;
        }
        *line_has_block = true;
        *line_open = true;
        *line_has_content = false;
    };

    let push_br = |text: &mut String,
                   blocks: &mut Vec<Block>,
                   soft_breaks: &mut Vec<u32>,
                   active: &mut [ActiveMark],
                   marks: &mut Vec<MarkRange>,
                   in_pre: usize| {
        flush_active_marks(active, marks, text.len(), in_pre);
        let blk = blocks.last().copied().unwrap_or(Block::Paragraph);
        text.push('\n');
        blocks.push(blk);
        if in_pre == 0 {
            soft_breaks.push((blocks.len() - 1) as u32);
        }
        for a in active.iter_mut() {
            a.start = text.len();
        }
    };
    // A `<br>` (text and block counts where it fell) waiting to see whether text follows it.
    let mut deferred_br: Option<(usize, usize)> = None;

    let mut i = 0;
    let bytes = html.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'<' {
            // Comment `<!-- ... -->`
            if html[i..].starts_with("<!--") {
                if let Some(end) = html[i + 4..].find("-->") {
                    i += 4 + end + 3;
                } else {
                    break;
                }
                continue;
            }
            // Processing instruction or doctype `<!...>` / `<?...>`
            if html[i..].starts_with("<!") || html[i..].starts_with("<?") {
                if let Some(end) = html[i + 2..].find('>') {
                    i += 2 + end + 1;
                } else {
                    break;
                }
                continue;
            }
            let Some(close_angle) = html[i + 1..].find('>') else {
                // Unclosed `<`, treat as literal text
                let decoded = decode_entities(&html[i..]);
                text.push_str(&decoded);
                break;
            };
            let raw_tag = html[i + 1..i + 1 + close_angle].trim();
            i += 1 + close_angle + 1;

            if raw_tag.is_empty() {
                continue;
            }
            let is_closing = raw_tag.starts_with('/');
            let body = raw_tag.trim_start_matches('/').trim_end_matches('/').trim();
            let tag_end = body
                .find(|c: char| c.is_whitespace() || c == '/')
                .unwrap_or(body.len());
            let tag = body[..tag_end].to_ascii_lowercase();
            let attrs = &body[tag_end..];
            let self_closing = raw_tag.ends_with('/')
                || matches!(
                    tag.as_str(),
                    "area" | "base" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta" | "source" | "track" | "wbr"
                );

            if !is_closing && matches!(tag.as_str(), "script" | "style" | "head") {
                if !self_closing {
                    // Skip until matching `</tag>` (case-insensitive)
                    let pat = format!("</{tag}");
                    let mut search = i;
                    let mut found = None;
                    while search + pat.len() <= bytes.len() {
                        if html[search..].len() >= pat.len()
                            && html.is_char_boundary(search)
                            && html.is_char_boundary(search + pat.len())
                            && html[search..search + pat.len()].eq_ignore_ascii_case(&pat)
                        {
                            // No `>` after this means none after any later match either.
                            found = html[search + pat.len()..].find('>').map(|gt| search + pat.len() + gt + 1);
                            break;
                        }
                        search += 1;
                    }
                    i = found.unwrap_or(bytes.len());
                }
                continue;
            }

            if !is_closing && tag == "br" {
                if deferred_br == Some((text.len(), blocks.len())) {
                    push_br(&mut text, &mut blocks, &mut soft_breaks, &mut active_marks, &mut marks, in_pre);
                    line_has_block = true;
                    line_open = true;
                }
                deferred_br = None;
                if in_pre > 0 {
                    push_br(&mut text, &mut blocks, &mut soft_breaks, &mut active_marks, &mut marks, in_pre);
                    line_has_content = false;
                } else {
                    // A `<br>` only breaks the line when something follows it in the same block,
                    // as browsers draw it: `<p>A<br></p>` is one line, `<p><br></p>` one blank line.
                    deferred_br = Some((text.len(), blocks.len()));
                    line_has_block = true;
                    line_open = true;
                    line_has_content = true;
                }
                continue;
            }

            // Deeper nesting than any real document keeps its text but not its tags, so a flood of
            // opening tags can't grow the stack without bound.
            if !is_closing && !self_closing && stack.len() >= MAX_DEPTH {
                continue;
            }

            if !is_closing {
                match tag.as_str() {
                    "ul" => {
                        list_stack.push(ListKind::Bullet);
                        // The first item takes a line still waiting for content (`<li><ul>`), but
                        // never a finished one, even if it was blank (`<p></p><ul>`).
                        if line_has_content {
                            line_open = false;
                        }
                    }
                    "ol" => {
                        list_stack.push(ListKind::Numbered);
                        // The first item takes a line still waiting for content (`<li><ul>`), but
                        // never a finished one, even if it was blank (`<p></p><ul>`).
                        if line_has_content {
                            line_open = false;
                        }
                    }
                    "li" => {
                        in_li += 1;
                        let blk = current_default_block(&list_stack, in_pre, in_quote, in_li);
                        start_new_paragraph(
                            &mut text,
                            &mut blocks,
                            &mut active_marks,
                            &mut marks,
                            &mut line_has_block,
                            &mut line_open,
                            &mut line_has_content,
                            blk,
                            in_pre,
                        );
                    }
                    "blockquote" => {
                        in_quote += 1;
                        let blk = current_default_block(&list_stack, in_pre, in_quote, in_li);
                        start_new_paragraph(
                            &mut text,
                            &mut blocks,
                            &mut active_marks,
                            &mut marks,
                            &mut line_has_block,
                            &mut line_open,
                            &mut line_has_content,
                            blk,
                            in_pre,
                        );
                    }
                    "pre" => {
                        in_pre += 1;
                        start_new_paragraph(
                            &mut text,
                            &mut blocks,
                            &mut active_marks,
                            &mut marks,
                            &mut line_has_block,
                            &mut line_open,
                            &mut line_has_content,
                            Block::Code,
                            in_pre,
                        );
                    }
                    "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        let mut blk = current_default_block(&list_stack, in_pre, in_quote, in_li);
                        // A heading outside a list, quote or code block is a heading line.
                        if blk == Block::Paragraph
                            && let Some(level) = tag.strip_prefix('h').and_then(|n| n.parse().ok())
                        {
                            blk = Block::Heading(level);
                        }
                        start_new_paragraph(
                            &mut text,
                            &mut blocks,
                            &mut active_marks,
                            &mut marks,
                            &mut line_has_block,
                            &mut line_open,
                            &mut line_has_content,
                            blk,
                            in_pre,
                        );
                    }
                    _ => {}
                }

                let mut to_add = Vec::new();
                let mut to_suppress = Vec::new();
                match tag.as_str() {
                    "b" | "strong" => to_add.push(Mark::Bold),
                    // A heading in a list item or quote can't be a heading line, so it stays bold.
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" if in_li > 0 || in_quote > 0 => {
                        to_add.push(Mark::Bold)
                    }
                    "i" | "em" => to_add.push(Mark::Italic),
                    "u" | "ins" => to_add.push(Mark::Underline),
                    "s" | "del" | "strike" => to_add.push(Mark::Strike),
                    "code" if in_pre == 0 => to_add.push(Mark::Code),
                    "a" => {
                        if let Some(href) = extract_attr(attrs, "href")
                            && is_safe_href(&href)
                        {
                            to_add.push(Mark::Link(href.trim().to_owned()));
                        }
                    }
                    "span" => {
                        if let Some(id) = extract_attr(attrs, "data-pill") {
                            to_add.push(Mark::Pill(id));
                        }
                    }
                    _ => {}
                }
                if let Some(style) = extract_attr(attrs, "style") {
                    parse_style_marks(&style, &mut to_add, &mut to_suppress);
                    // The element's own style wins over its tag: Google Docs wraps a whole
                    // paste in `<b style="font-weight:normal">`.
                    to_add.retain(|m| !to_suppress.iter().any(|r| r.same_kind(m)));
                }

                let pos = text.len();
                let mut frame = TagFrame {
                    tag: tag.clone(),
                    added_marks: Vec::new(),
                    suppressed_marks: Vec::new(),
                };

                for rem in to_suppress {
                    if let Some(idx) = active_marks.iter().position(|a| a.mark.same_kind(&rem)) {
                        let popped = active_marks.remove(idx);
                        if in_pre == 0 && popped.start < pos {
                            marks.push(MarkRange {
                                start: popped.start,
                                end: pos,
                                mark: popped.mark.clone(),
                            });
                        }
                        frame.suppressed_marks.push(popped.mark);
                    }
                }
                for m in to_add {
                    if let Some(idx) = active_marks.iter().position(|a| a.mark.same_kind(&m)) {
                        if active_marks[idx].mark != m {
                            let old = active_marks.remove(idx);
                            if in_pre == 0 && old.start < pos {
                                marks.push(MarkRange {
                                    start: old.start,
                                    end: pos,
                                    mark: old.mark.clone(),
                                });
                            }
                            frame.suppressed_marks.push(old.mark);
                            active_marks.push(ActiveMark { mark: m.clone(), start: pos });
                            frame.added_marks.push(m);
                        }
                    } else {
                        active_marks.push(ActiveMark { mark: m.clone(), start: pos });
                        frame.added_marks.push(m);
                    }
                }

                if !self_closing {
                    *open_tags.entry(tag.clone()).or_default() += 1;
                    stack.push(frame);
                }
            } else {
                // Closing tag
                let pos = text.len();
                let open = open_tags.get(&tag).is_some_and(|&n| n > 0);
                if let Some(frame_idx) = open.then(|| stack.iter().rposition(|f| f.tag == tag)).flatten() {
                    while stack.len() > frame_idx {
                        let frame = stack.pop().unwrap();
                        if let Some(n) = open_tags.get_mut(&frame.tag) {
                            *n -= 1;
                        }
                        for m in frame.added_marks.into_iter().rev() {
                            if let Some(idx) = active_marks.iter().rposition(|a| a.mark == m) {
                                let ended = active_marks.remove(idx);
                                if in_pre == 0 && ended.start < pos {
                                    marks.push(MarkRange {
                                        start: ended.start,
                                        end: pos,
                                        mark: ended.mark,
                                    });
                                }
                            }
                        }
                        for sup in frame.suppressed_marks {
                            if !active_marks.iter().any(|a| a.mark.same_kind(&sup)) {
                                active_marks.push(ActiveMark { mark: sup, start: pos });
                            }
                        }
                        match frame.tag.as_str() {
                            "ul" | "ol" => {
                                list_stack.pop();
                                line_open = false;
                            }
                            "li" => {
                                in_li = in_li.saturating_sub(1);
                                line_open = false;
                            }
                            "blockquote" => {
                                in_quote = in_quote.saturating_sub(1);
                                line_open = false;
                            }
                            "pre" => {
                                in_pre = in_pre.saturating_sub(1);
                                line_open = false;
                            }
                            "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                                line_open = false;
                            }
                            _ => {}
                        }
                    }
                }
            }
        } else {
            // Plain text node up to next `<`
            let next_lt = html[i..].find('<').map_or(bytes.len(), |off| i + off);
            let raw_slice = &html[i..next_lt];
            i = next_lt;
            let decoded = decode_entities(raw_slice);
            if in_pre > 0 {
                // Inside `<pre>`, preserve whitespace and split newlines into `Block::Code` paragraphs.
                let mut chars = decoded.chars().peekable();
                while let Some(c) = chars.next() {
                    if c == '\r' {
                        if chars.peek() == Some(&'\n') {
                            continue;
                        }
                        text.push('\n');
                        blocks.push(Block::Code);
                        line_has_block = true;
                        line_open = false;
                        line_has_content = false;
                    } else if c == '\n' {
                        // Ignore a single leading newline immediately after `<pre>`
                        if line_open && !line_has_content {
                            line_open = false;
                            continue;
                        }
                        text.push('\n');
                        blocks.push(Block::Code);
                        line_has_block = true;
                        line_open = false;
                        line_has_content = false;
                    } else if c == '\t' || !c.is_control() {
                        text.push(c);
                        line_has_block = true;
                        line_open = false;
                        line_has_content = true;
                    }
                }
            } else {
                // Outside `<pre>`: ignore pure inter-block whitespace if we're between closed blocks and a `<` follows
                if !line_open && decoded.chars().all(char::is_whitespace) && (!line_has_block || line_has_content) {
                    continue;
                }
                if line_open && !line_has_content && decoded.chars().all(char::is_whitespace) && i < bytes.len() && html[i..].starts_with('<') {
                    // Check if the next tag is a block child like `<p>` or `<li>` inside `<blockquote>` or `<ul>`
                    let rest_tag = html[i + 1..].trim_start();
                    let next_is_block = ["p", "div", "ul", "ol", "li", "blockquote", "pre", "h1", "h2", "h3", "h4", "h5", "h6"]
                        .iter()
                        .any(|b| {
                            rest_tag.len() >= b.len()
                                && rest_tag.is_char_boundary(b.len())
                                && rest_tag[..b.len()].eq_ignore_ascii_case(b)
                                && rest_tag[b.len()..].starts_with(|c: char| c.is_whitespace() || c == '>' || c == '/')
                        });
                    if next_is_block {
                        continue;
                    }
                }
                if !line_open && !decoded.is_empty() && (!line_has_block || line_has_content) {
                    let blk = current_default_block(&list_stack, in_pre, in_quote, in_li);
                    if !line_has_block {
                        if let Some(last) = blocks.last_mut() {
                            *last = blk;
                        }
                        line_has_block = true;
                        line_open = true;
                    } else if !decoded.chars().all(char::is_whitespace) {
                        // Loose text after a closed block (`<p>A</p>B`) starts its own line.
                        start_new_paragraph(
                            &mut text,
                            &mut blocks,
                            &mut active_marks,
                            &mut marks,
                            &mut line_has_block,
                            &mut line_open,
                            &mut line_has_content,
                            blk,
                            in_pre,
                        );
                    }
                }
                // Collapse ASCII whitespace runs (newlines/tabs/spaces) to single spaces in HTML flow
                if decoded.chars().all(char::is_whitespace) {
                    if deferred_br == Some((text.len(), blocks.len())) {
                        continue;
                    }
                } else {
                    if deferred_br == Some((text.len(), blocks.len())) {
                        push_br(&mut text, &mut blocks, &mut soft_breaks, &mut active_marks, &mut marks, in_pre);
                    }
                    deferred_br = None;
                }
                let mut prev_space = text.is_empty() || text.ends_with('\n') || text.ends_with(' ');
                for ch in decoded.chars() {
                    if ch == '\u{00a0}' {
                        text.push('\u{00a0}');
                        prev_space = false;
                        line_has_block = true;
                        line_open = true;
                        line_has_content = true;
                    } else if ch.is_whitespace() {
                        if !prev_space {
                            text.push(' ');
                            prev_space = true;
                            line_has_block = true;
                            line_open = true;
                            line_has_content = true;
                        }
                    } else if !ch.is_control() {
                        text.push(ch);
                        prev_space = false;
                        line_has_block = true;
                        line_open = true;
                        line_has_content = true;
                    }
                }
            }
        }
    }

    let end_pos = text.len();
    flush_active_marks(&mut active_marks, &mut marks, end_pos, in_pre);

    // Strip a single trailing newline inside `<pre>` if it left an empty trailing line
    if blocks.len() > 1
        && blocks.last() == Some(&Block::Code)
        && text.ends_with('\n')
        && !soft_breaks.contains(&((blocks.len() - 1) as u32))
    {
        text.pop();
        blocks.pop();
    }

    let mut edit = TextEdit::from_parts(text, marks, blocks);
    edit.soft_breaks = soft_breaks;
    edit
}
