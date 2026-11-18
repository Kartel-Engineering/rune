- **Web: a key that hands the page's field the browser's focus no longer leaves a modifier
  held.** The hand-over now runs after the key is handled (`activateSoon`), not inside it. winit
  reports a key's modifiers after the key, and the canvas's blur clears them, so a Control press
  that moved focus left Control held: the next key on the canvas was a shortcut (a letter typed
  after a click on a button was lost to Select All). A press still hands focus over inside the
  event, as a phone's keyboard needs. `scripts/web-input-smoke.mjs` checks it.
