//! The space an app puts between a field's blocks: [`El::block_space_above`](crate::El::block_space_above)
//! above a block, and [`El::blank_line_height`](crate::El::blank_line_height) for an empty line
//! between two.

use super::*;

/// Per paragraph of `spec`, the device px above it and the height its empty line takes, in lines
/// (0: its own line height). Empty when every one is the default, so the field lays out exactly
/// as with neither set.
pub(super) fn spacing(spec: &TextSpec, scale: f32) -> Vec<[f32; 2]> {
    if !spec.block_spaced() {
        return Vec::new();
    }
    let lines: Vec<&str> = spec.content.split('\n').collect();
    let out: Vec<[f32; 2]> = (0..lines.len())
        .map(|k| [space_above(spec, k, scale), blank_line(spec, &lines, k)])
        .collect();
    if out.iter().all(|&s| s == [0.0, 0.0]) {
        Vec::new()
    } else {
        out
    }
}

/// The device px [`El::block_space_above`](crate::El::block_space_above) puts above paragraph
/// `k`: none above the first, a line a soft break started, or a quote or code line under one of
/// its own.
fn space_above(spec: &TextSpec, k: usize, scale: f32) -> f32 {
    let (Some(space), Some(j)) = (spec.cold.block_space_above, k.checked_sub(1)) else {
        return 0.0;
    };
    let blocks = &spec.cold.blocks;
    let block = blocks.get(k).copied().unwrap_or(Block::Paragraph);
    let joined = matches!(block, Block::Quote | Block::Code) && blocks.get(j) == Some(&block);
    if joined || spec.cold.soft_breaks.binary_search(&(k as u32)).is_ok() {
        return 0.0;
    }
    // A negative or NaN space is none; one over 10,000 points, infinity too, is 10,000.
    let px = (space(block).clamp(0.0, 10_000.0) * scale).round();
    if px.is_finite() { px } else { 0.0 }
}

/// The lines [`El::blank_line_height`](crate::El::blank_line_height) gives paragraph `k` when it's
/// an empty paragraph between two with words, by their blocks; 0 for any other, and for 1.
fn blank_line(spec: &TextSpec, lines: &[&str], k: usize) -> f32 {
    let (Some(height), Some(j)) = (spec.cold.blank_line_height, k.checked_sub(1)) else {
        return 0.0;
    };
    let words = |i: usize| {
        lines
            .get(i)
            .is_some_and(|l| !l.trim_end_matches('\r').is_empty())
    };
    let block = |i: usize| spec.cold.blocks.get(i).copied().unwrap_or(Block::Paragraph);
    let soft = |i: usize| spec.cold.soft_breaks.binary_search(&(i as u32)).is_ok();
    // A run of empty lines, and one a soft break starts or ends, keep their lines.
    // ponytail: a plain field (no marks, every block a paragraph) draws as plain text and passes
    // no soft breaks here, so its empty line a Shift+Return break made is short too. Pass them
    // with `blank_line_height` set when an app needs that line kept.
    if !lines[k].trim_end_matches('\r').is_empty()
        || block(k) != Block::Paragraph
        || !words(j)
        || !words(k + 1)
        || soft(k)
        || soft(k + 1)
    {
        return 0.0;
    }
    // A NaN height is a line; any other is 0.1 to 10 lines.
    let h = height(block(j), block(k + 1));
    if h.is_nan() || h == 1.0 {
        0.0
    } else {
        h.clamp(0.1, 10.0)
    }
}
