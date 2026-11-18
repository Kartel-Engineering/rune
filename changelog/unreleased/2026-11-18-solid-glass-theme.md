- **`rune-theme-solid-glass`: Solid Glass's smoky glass panes as data and a rune-kit theme.**
  `Look { dark, opaque }.material(Pane::Card)` gives one `Material { fill, blur, layers }` per
  pane (panel, card, popover, sidebar, control, the one state and armed, primary at rest, hover
  and pressed, field at rest and in focus, groove and knob), and `apply` paints it on an `El`:
  the fill, a backdrop blur of 26 points (14 for a control) and the box shadows of
  `solid-glass.css`, in Rune's paint order. Opaque drops the blur and swaps in the
  reduced-transparency fills, keeping the edges. `kit(dark)` themes rune-kit components with it:
  a submit button is the ink pane, a field the clear one. `examples/solid-glass` shows every pane
  in dark, light and opaque, and renders headless with `--snap`. At 2x it lands within 4 of 255
  of the same page drawn by Chrome from the stylesheet, at 51 points on rims, hairlines, faces and
  shadows.
