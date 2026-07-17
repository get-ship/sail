---
title: "Styling is sx"
description: "One attribute, a closed token vocabulary, and states as prefixes."
order: 2
slug: "02-styling-is-sx"
prev: "01-a-page-is-markup"
next: "03-state-is-a-controller"
chapter: "surface"
---

When a tag and its props are not enough, there is exactly one place to
go: the `sx` attribute. It is a list of `prop:token` pairs —

```
<column sx="p:5 bg:surface-1 border:border radius:lg shadow:sm hover:shadow:hover">
```

— and every value in it is a **token from a closed list**. Spacing is
the scale (`p:4`, `px:3`, `mb:2`), color is the theme table
(`bg:surface-1`, `fg:muted-foreground`, `border:primary`), type is the
type scale (`text:sm`, `weight:semibold`, `tracking:tight`). There are
no pixel values, no hex colors, no arbitrary anything.

## The compiler holds the line

This is not a convention — it is enforced. Write an off-system value:

```
<column gap="4" sx="p:12px">
```

and the build stops with:

```
Error: line 4 col 21: [sail-sx] sx `p:12px` — `12px` is not a token (raw values are
not allowed; spend a token). allowed: 0, 0_75, 1, 1_25, 1_5, 2, 2_5, 3, 3_5, 4,
4_5, 5, 6, 7, 8, 9, half, size-px; did you mean `p:1`?
  suggest: sx is surface-only: spacing 0–8 (p/m…), colors (bg/fg), radius, text.
  Layout is the primitives' job (<row>/<column>/<grid>). Raw px/hex are not expressible.
```

Read that message closely — it carries the entire allowed vocabulary
for the prop you misused, plus a nearest guess. The rejection is the
documentation. Lesson 08 is built entirely on this idea.

## States are prefixes

Interaction states are the same vocabulary behind a prefix — no
selectors, no stylesheet:

- `hover:` / `active:` / `disabled:` — `hover:bg:surface-2`,
  `hover:shadow:hover`
- `focus:` / `focus-within:` — `focus-within:border:primary` on a row
  that contains an input
- `current:` — applies when the element carries `aria-current`, which
  is how the solved example styles all three nav pills identically and
  lets the *attribute* pick the active one
- conditions: `dark:` / `light:` / `motion-reduce:` / `forced-colors:`

## Try it

The starter has the structure; the surface is yours:

1. **The pills.** Give each link `px:3 py:1 radius:full text:sm
   weight:semibold`, muted ink with `hover:fg:foreground`, and
   `current:bg:primary current:fg:primary-foreground`. Only the link
   with `aria-current="page"` lights up.
2. **The card.** `p:5 bg:surface-1 border:border radius:lg shadow:sm`,
   then add `hover:shadow:hover` and watch it lift.
3. **The alert row.** Put `focus-within:border:primary` on the row
   around the input, then click into the input.
4. **Break it on purpose.** Change `p:5` to `p:24px`, run the build,
   and read your first sx rejection end to end.
