- **Web: a key arrives with its own modifiers.** winit's web backend sent a key's
  `ModifiersChanged` after the key, so Rune read the key with the modifiers it last knew. When
  those were stale (Control pressed while focus was outside the canvas, or a pointer event that
  copied the mouse's modifier state), Ctrl+J reached the app as a plain J. `vendor/winit.patch`
  now sends the modifiers first, for presses and releases. `scripts/web-input-smoke.mjs` checks
  Control held before focus moves to the canvas.
