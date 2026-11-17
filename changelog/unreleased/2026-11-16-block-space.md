- **`El::block_space_above`: extra space above a rich text field's blocks, by kind.**
  `text_input(..).block_space_above(|b| if let Block::Heading(1..=3) = b { 30.0 } else { 0.0 })`
  puts 30 points above each heading of level 1 to 3, as CSS's `margin-top` on a
  `contenteditable`'s blocks. The field's first paragraph gets none, a line a Shift+Return break
  started gets none, and a quote or code line under one of its own kind gets none, so a run of
  them still draws as one box and one bar. A negative or NaN space is none, and one over 10,000
  points is 10,000. The caret, a click (in the space it lands on the nearer line), Up and Down
  (the column is kept), a selection (it leaves the space bare), the field's height, caret
  following, block decorations and accessibility line rectangles all follow the space. A field
  with the space set and more than one paragraph lays out one paragraph at a time, all in the
  direction of its one layout, unless `El::dir_by_paragraph` is set. A field with right-to-left
  text in it (a Hebrew or Arabic letter anywhere), or right to left by its element's direction,
  gets no space and lays out as before, unless it is by paragraph. Unset, or when every space
  comes to 0, a field lays out and draws as before.
