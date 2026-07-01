# Design reference — Paired editing (the 2-window model)

Distilled from the Ship "Spaces" interaction mock (`scene-paired`). Behavioral
source of truth — re-express in SUI idioms, do not match pixels.

## The model: the hub edits, the space lives

Anything you build in Ship — a space's chrome, a pipeline, a skill — has two
faces. You **edit the source/config in the hub**; the **result runs live in a
space window beside it**. Every edit streams into the running space immediately,
shown by a "sync pill" (`live · <name>` · `<n> edits · <latency>ms`).

Tagline from the mock: **"The hub edits. The space lives."**

## Apply-our-insight: collapse two OS windows → one split view

The mock shows two separate OS windows (hub + space). A SUI view is a single
surface, so for this task re-express the interaction as **one split view with
two panes**: an **editor pane** (left) and a **live preview pane** (right).
Editing a control in the editor updates the preview instantly. That captures the
behavioral essence — *change it here, see it there* — which is the whole point.

Build the **space chrome editor** slice (the mock's clearest paired example):

### Editor pane (left) — an appearance inspector

Sections + controls (single-select segmented groups):

- **Theme** — `ship-light` · `paper` · `ship-dark` · `halo`  (default `paper`)
- **Nav** — `rail` · `top` · `none`  (default `top`)
- **Dock** — `left` · `center` · `right` · `off`  (default `off`)
- **Density** — `tight` · `cozy` · `loose`  (default `cozy`)
- **Glyph** — a small set of single-char tiles (`P ◆ ✦ ▲ M`), single-select
- **Accent** — a row of color circles, single-select

A header shows the space name and an **unsaved** indicator once anything changes
(`● <n> unsaved`), plus a **Save** button.

### Preview pane (right) — the live space

A card representing the running space that **reflects the current selection**:

- shows the chosen **theme** (e.g. a `paper` / `ship-dark` badge),
- shows the chosen **nav** placement (e.g. a top nav bar vs a left rail),
- shows the chosen **dock**,
- a status line / sync indicator: **live · <space>** with an edits count.

The preview is **derived** from the editor state — changing Theme to `ship-dark`
immediately flips the preview to dark; changing Nav to `rail` moves the nav.

## Data — define-and-fake

No real SDK needed for the core loop: the config is **local editor state**
(a signal), and the preview is `derived` from it. "Save" calls a faked
`ship.spaces.saveConfig({...})` (no-op in tests). The starting config is the
defaults above.

```ts
interface SpaceChrome {
  theme: 'ship-light' | 'paper' | 'ship-dark' | 'halo'
  nav: 'rail' | 'top' | 'none'
  dock: 'left' | 'center' | 'right' | 'off'
  density: 'tight' | 'cozy' | 'loose'
  glyph: string
  accent: string
}
```
