# Task: build the Authority approval view

Build `views/authority/` — the hub surface where a human approves or denies the
capability requests and command approvals that running spaces (agents) raise.
This is real, shippable work: the hub's authority queue + the capability-request
modal.

The design source is `eval/tasks/authority/reference.md` (read it). It is
behavioral, not pixel-exact — re-express it in SUI idioms. There is NO mockup
image; the reference text is the spec.

## File scope

```
views/authority/
  page.sui            # the queue + the capability modal
  controller.ts       # pending/grants live data, the decide() handlers, modal open state
  manifest.jsonc      # { name, path: "/authority", title, layout: "default", auth: "public" }
  style.css           # tokens only — no raw hex / px in themed properties
  _components/         # OPTIONAL view-private widgets; tag MUST start with `authority-`
```

Don't add new primitives under `packages/components/sui/primitives/`. Keep everything scoped
to this view.

## Data — define-and-fake

There is no `authority` SDK domain yet. Code the view against the shape in
`reference.md` (`ship.authority.pending()` live, `ship.authority.decide(...)`
mutation). The behavioral tests inject a fake `ship` at that boundary, so you do
NOT need a real daemon op — author as if it exists, read it through `<data live>`
+ `live()`, and guard the three states (loading / ok / error).

## Behavior requirements

1. **Queue** — title "Authority", subtitle, and the "Replay decisions" / "Policy…"
   header buttons. A "Awaiting your decision" section with a `{n} pending` counter
   and one card per pending request.
2. **Pending card** — shows `<Agent> · <Space>`, the kind line (`wants to run` /
   `requests capability`), the command/scope body, a `Why · …` line, the risk tag,
   and three actions: **Deny**, **Allow once**, **Allow this session**.
3. **Deciding** — clicking any of the three actions calls `ship.authority.decide`
   and removes that card from the queue; the pending counter decrements.
4. **Capability modal** — a pending *capability* request can open a modal with
   **SCOPE**, **WHY**, and **FOR HOW LONG** (three single-select options;
   "Until I close <Space>" selected by default), plus **Deny** / **Allow**.
5. **Empty state** — when nothing is pending, show a calm "You're all caught up"
   (or similar) instead of an empty grid.
6. No-op is fine for "Replay decisions" / "Policy…" (empty handlers).

## Hard rules (linted + compiled — failure floors the score)

- Templates declare, controllers do. No `<script>` in `.sui`.
- Read signals by call (`pending()`), write via `state.x = v` / `sig.set(v)` only in `on:` handlers.
- No raw hex / px / rgb in `style.css` for themed properties — use `var(--token)`.
- Block syntax for control flow; `{#for}` MUST declare `key={…}`.
- Buttons and inputs must have accessible names/labels (the behavioral tests
  query by visible text + role — unlabeled `<div>` buttons will fail).
- No `el.innerHTML` / `outerHTML` / `insertAdjacentHTML` in `controller.ts`.

## Acceptance

The behavioral acceptance is `eval/tasks/authority/authority.behavior.jsonc` — your
view must satisfy those scenarios (queue shows pending, deciding removes a card,
modal shows SCOPE/WHY/FOR HOW LONG, empty state). Build to make them pass.

## Verification before completion

```sh
cargo run -p sui-compiler -- build views/authority/page.sui
# plus each _components/<tag>/<tag>.sui --component-name <tag>
```

Both must exit 0 (sui-lint runs inside the compiler). Then commit cleanly.
