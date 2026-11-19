- **`El::blank_line_height`: a shorter blank line between two blocks of a rich text field.**
  `text_input(..).blank_line_height(|_, _| 0.55)` draws an empty paragraph between two lines with
  words at 0.55 of a line, as CSS's `margin` between `<p>`s sets paragraphs apart, while the text
  keeps its blank line. The function gets the blocks above and below, so a blank line beside a
  heading can keep its full line. A run of empty lines, one a Shift+Return break starts or ends,
  an empty list item, quote, code or heading line, and the first and last lines keep their
  lines. The caret in the short line is as tall as it; a click, Up and Down, and a selection
  follow it, and typing in it gives it its full line at once. A NaN or 1 is a full line; other
  heights are 0.1 to 10 lines. Right-to-left fields keep their lines, as with
  `El::block_space_above`. Unset, a field lays out and draws as before.
