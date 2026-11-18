- **Web: typing through the page's own field is one undo step, as typing keys is.** Each edit
  the field reports now says its `inputType`, and `TextEdit` records typed text, line breaks and
  deletions as their keys do (`history::step`); a paste, a drop or a spelling fix is a step of
  its own. Before, every character was its own step, and an app's handler that changed the text
  on a redo (a Markdown list prefix) cut the redo short. `scripts/web-input-smoke.mjs` checks it.
