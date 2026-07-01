# Task: build the space creation flow

You are building the first-run **create-a-space** wizard for Ship — the flow a
user hits the very first time they open the app. Four screens, end-to-end.

Authoritative inputs are four mockups, in order, under
`eval/mockups/`:

1. `0-welcome.png` — Welcome / "what is Ship" intro screen
2. `1-name-icon.png` — Step 1 of 3: name + glyph + accent picker
3. `2-theme-chrome.png` — Step 2 of 3: theme + navigation + dock picker
4. `3-registered.png` — Step 3 of 3: "your space is installed" summary

Open each mockup, read the layout, copy and labels. They are the spec — your
output should look like a faithful first pass. You will not match pixel-for-pixel
and you are not expected to; you ARE expected to render the same structure, the
same controls, in roughly the same visual hierarchy.

## File scope

Author a new view at:

```
views/create-space/
  page.sui            # wizard shell — declares step state + four-screen body
  controller.ts       # step state, form state, navigation handlers
  manifest.jsonc      # { name, path: "/create-space", title, layout, auth: "public" }
  style.css           # page-scoped (tokens only — no raw hex / px in themed properties)
  _components/         # OPTIONAL — view-private sub-widgets, tag MUST start with `create-space-`
```

You MAY decompose pickers into private sub-components under `_components/` if
that's cleaner. Don't reach into `packages/components/sui/primitives/` to add new built-in
primitives — keep everything you author scoped to this view.

## Behavior requirements

1. **Welcome screen** — title + a one-paragraph explanation of what a Space is
   (read the mockup), two CTAs at the bottom: "Skip" (no-op for this task) and
   "Create your first space" (advances to step 1).
2. **Step 1 — name + glyph + accent** — text input for name (default
   "Brand Voice"), a row of glyph tiles (use the letters/symbols visible in the
   mockup; pick any reasonable 8–10 set), a row of accent color circles
   (8 colors; any reasonable palette). Selection is single-select within each
   group. A small preview card at the top shows the current selection.
   "Back" returns to welcome; "Continue" advances to step 2.
3. **Step 2 — theme + navigation + dock** — three single-select rows.
   Themes: light, dark, paper, halo (default), console, contrast.
   Navigation: rail (default), top, none. Dock: left, centered (default),
   right, off. Tip line at the bottom: "You can change all of this later, in
   Studio." "Back" returns to step 1; "Continue" advances to step 3.
4. **Step 3 — registered** — summary card showing the chosen name + glyph +
   accent + theme, followed by a vertical "Where you'll find it" list with
   FOUR rows: Applications (macOS dock), Spotlight, Ship hub, Shell PATH.
   "Done" and "Launch <space-name>" CTAs at the bottom (handlers can be no-op).
5. **No-op handlers are fine** — the launch / install commands aren't expected
   to actually register a space with the daemon. The flow is the deliverable;
   the side-effects can be empty arrow-fn handlers.

## Hard rules (will be linted + compiled — failure = score floor)

- Templates declare, controllers do. No `<script>` block in `.sui`.
- Template expressions are PURE functions of scope signals —
  `map`/`filter`/arithmetic/ternary/member access only. A `.sui` file
  containing a capability (`fetch`/IO/`eval`/`localStorage`/`Date.now`/
  `Math.random`/non-owned mutation such as `arr.sort()`) OR a
  statement/declaration (`const`/`let`/`function`) is a FAIL: promote it to a
  controller `derived`/`live`/`on`, reached through the ship SDK. (Background:
  `docs/ship-sui/references/docs/authoring-model.md`.)
- Read signals by call (`name()`), write via assignment (`state.name = v`)
  or `name.set(v)` only inside `on:` handlers.
- No raw hex / px / rgb in `style.css` for themed properties — use
  `var(--token)` (foreground, background, primary, accent, border, etc.).
- Block syntax for control flow: `{#if}`, `{#match}` over `step()`.
- `{#for}` blocks MUST declare `key={…}` with a stable identifier.
- No `el.innerHTML` / `outerHTML` / `insertAdjacentHTML` in `controller.ts`.

## Verification before completion

Run the project compiler against your view:

```sh
cargo run -p sui-compiler -- build views/create-space/page.sui
cargo run -p sui-compiler -- build views/create-space/_components/<tag>/<tag>.sui --component-name <tag>
```

Both must exit 0. sui-lint runs as a library inside `sui-compiler build` —
the compile step IS the lint step. There is no `sui-lint` binary.

When the view is registered, the structure is in place, and both verifications
are green, commit cleanly — NEVER push (the operator owns remotes). (If you're a pipeline-step
agent with `SHIP_ACTOR_ID` set, `ship ops call pipelines.step_done --json
'{"slug":"'$SHIP_ACTOR_ID'","summary":"<one-liner>"}'` advances the step
monitor.)
