# @ship/l1-conformance

The quality gate for the **L1 language surface**. Renders every L1 element's
state matrix across all **7 themes**, asserts invariants in-page (the gate), and
writes a screenshot for eyeballing. Replaces the throwaway `*-proof.mjs` scripts:
the **audit trail is the versioned specs + probes + the CI gate** — quality is
tracked because the checks live in the repo and run every time, not because
images are committed. Screenshots are a local dev artifact (gitignored,
regenerated each run) — not pixel baselines.

## Run

```sh
node run.mjs          # every spec → assert + write screenshots (CI gate)
node run.mjs chip     # one element, while iterating (dev loop)
```

Exit code is non-zero if any spec's probe reports issues — that's the gate.
Screenshots land in `__screenshots__/<tag>.png` (gitignored) for eyeballing
while you iterate; they are not baselines and not committed.

## Add an element

Create `specs/<tag>.mjs`:

```js
export default {
  tag: 'switch',
  width: 640,
  rows() { return [ /* rows of HTML: one per state group */ ] },
  probe() { /* runs in the page; return { count, bad: [] } */ },
}
```

`rows()` returns the element's states (tones, variants, sizes, behavioural
gates). `probe()` runs the element's `use:` action essentials and asserts the
contract (ARIA, visibility gates, computed chrome) — push to `bad` on any
violation.

## Render modes

- **v1 (here):** injects the real framework CSS (`tokens/themes/contracts/
  element-skins`) and renders each element's lowered-output shape. Faithful for
  **mechanism-1** (compiles-away) and **mechanism-2** (native + skin) L1, whose
  output *is* static HTML + that CSS.
- **Follow-up:** **mechanism-3** composites (shadow DOM + controllers) need real
  mounting — a second render mode in this same harness, not a second framework.

## Fidelity note

v1 specs render the lowered *shape* by hand. The tracked upgrade is to compile a
canonical `.sui` usage snippet with the native `sui-compiler` and assert the
spec's markup matches the real lowered output — so a spec can never silently
drift from the compiler. Until then, keep specs mirrored to the emitted DOM.
