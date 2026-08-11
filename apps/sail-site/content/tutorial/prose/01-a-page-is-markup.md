---
title: "A page is markup"
description: "Layout keywords and component props carry a whole screen — zero JavaScript."
order: 1
slug: "01-a-page-is-markup"
prev: null
next: "02-styling-is-sx"
chapter: "surface"
---

In SAIL, a screen is **tags all the way down**. Structure comes from the
layout keywords — `<row>`, `<column>`, `<grid>`, `<center>`, `<stack>` —
and appearance comes from component props. The solved example is a
complete settings screen, and its controller file does not exist: there
is nothing for it to say.

## Layout is elements, not CSS

```
<row align="center" justify="between" gap="4">
  <h2>Workspace settings</h2>
  <button variant="solid" tone="primary">Save changes</button>
</row>

<grid cols="2" gap="4">
  <card padding="lg">…</card>
  <card padding="lg">…</card>
</grid>
```

The layout keywords take `gap`, `align`, and `justify`; `<grid>` adds
`cols` and `rows` (a fixed track count, 1–12). Rows wrap by default —
`<row nowrap>` is the single-line row. That is nearly the whole layout
API. An attribute the keyword does not recognize is a **compile
error** — the language never silently drops what you wrote.

## Props carry the design system

Components speak two shared contracts:

- **tone** — `primary` / `info` / `success` / `warning` /
  `destructive` / `accent` / `muted`. Omit the attribute for rest paint.
  Every toned component reads the same token table, so `tone="destructive"`
  means the same thing on a `<badge>`, a `<button>`, a `<card>`, and a
  `<switch>`.
- **variant** (where it applies) — `solid`, `soft`, `outline`, `ghost` —
  how loudly the tone speaks. `size` scales controls (`sm`/`md`/`lg`).

Text needs no styling at all: `<h1>`–`<h6>`, `<p>`, `<small>`,
`<strong>` come off the type scale automatically.

## Try it

The starter is an empty `<column>`. Build the screen:

1. **The header row.** A `<row justify="between" align="center">` with a
   title column on the left and a `tone="primary"` save button on the
   right.
2. **Two cards in a grid.** `<grid cols="2" gap="4">` holding a Profile
   card (an `<avatar>`, a name, an email) and a Notifications card
   (three `<row justify="between">`s, each pairing a label column with a
   `<switch>`).
3. **The danger zone.** A `<card tone="destructive" variant="outline">`
   with a matching outline button. Notice you never chose a red — the
   tone did.

If you find yourself wanting a stylesheet, stop: the next lesson is the
one styling surface the language has.
