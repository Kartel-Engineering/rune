- **A field laid out by paragraph finds each paragraph's spans by binary search.** Keying and
  shaping a paragraph read the spans over it by filtering every span of the field, so a long
  formatted note cost paragraphs times spans on each key (a 300 KB note with 33,000 lines and
  formats: 2.6 s a key). A text's spans are in order (`flatten_spans`), so its paragraph's spans
  are one run found with `partition_point` (`spans_in`); a debug assertion checks the order.
  The same spans come out in the same order: layouts, carets and pixels are unchanged.
