# Task: build the Space Chrome editor (paired editing)

Build `views/space-editor/` — the hub surface where you edit a space's
appearance and see it reflected live. This is the "hub edits, the space lives"
model from the Ship Spaces design, collapsed into one split view (editor pane +
live preview pane).

The design source is `eval/tasks/paired/reference.md` (read it).
Behavioral, not pixel-exact — re-express in SUI idioms. No mockup image; the
reference is the spec.

## File scope

```
views/space-editor/
  page.sui            # split view: appearance inspector (left) + live preview (right)
  controller.ts       # the chrome config signal, derived preview, change/save handlers
  manifest.jsonc      # { name, path: "/space-editor", title, layout: "default", auth: "public" }
  style.css           # tokens only — no raw hex / px in themed properties
  _components/         # OPTIONAL view-private widgets; tag MUST start with `space-editor-`
```

Keep everything scoped to this view; don't add primitives under
`packages/components/sui/primitives/`.

## Data — define-and-fake

No real SDK needed for the core loop. The chrome config is **local editor state**
(a signal seeded with the defaults in `reference.md`); the preview is `derived`
from it. "Save" calls a faked `ship.spaces.saveConfig({...})` (no-op in tests).

## Behavior requirements

1. **Editor pane** — an **Appearance** inspector with single-select groups:
   **Theme** (ship-light / paper / ship-dark / halo), **Nav** (rail / top / none),
   **Dock** (left / center / right / off), **Density** (tight / cozy / loose),
   plus a **Glyph** row and an **Accent** color row. Defaults per `reference.md`.
2. **Live preview pane** — a card representing the running space that is
   **derived** from the editor state: it shows the chosen theme (a visible
   theme badge/label), the chosen nav placement, and the chosen dock.
3. **Edit reflects instantly** — choosing **ship-dark** in Theme updates the
   preview to show dark; choosing **rail** in Nav updates the preview's nav.
   No save needed for the preview to update (it's reactive).
4. **Unsaved + Save** — once any control changes, show an unsaved indicator;
   a **Save** button calls the faked save (no-op).
5. **Sync indicator** — a small "live · <space>" indicator near the preview.

## Hard rules (linted + compiled — failure floors the score)

- Templates declare, controllers do. No `<script>` in `.sui`.
- Read signals by call, write via `state.x = v` / `sig.set(v)` only in `on:` handlers.
- No raw hex / px / rgb in `style.css` for themed properties — use `var(--token)`.
- Block syntax for control flow; `{#for}` MUST declare `key={…}`.
- Controls must have accessible names/labels (behavioral tests query by visible
  text + role).
- No `el.innerHTML` / `outerHTML` / `insertAdjacentHTML` in `controller.ts`.

## Acceptance

`eval/tasks/paired/paired.behavior.jsonc` — your view must
satisfy those scenarios (inspector renders, preview reflects theme + nav choices,
unsaved appears on change). Build to make them pass.

## Verification before completion

```sh
cargo run -p sui-compiler -- build views/space-editor/page.sui
# plus each _components/<tag>/<tag>.sui --component-name <tag>
```

Both must exit 0 (sui-lint runs inside the compiler). Then commit cleanly.
