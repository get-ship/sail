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
go: the `sx` attribute. It is a list of atoms —

```
<column sx="p-5 bg-surface-1 border border-border rounded-lg shadow-sm hover:shadow-hover">
```

— and every value in it is a **token from a closed list**. Spacing is
the scale (`p-4`, `px-3`, `mb-2`), color is the theme table
(`bg-surface-1`, `text-muted-foreground`, `border-primary`), type is the
type scale (`text-sm`, `font-semibold`, `tracking-tight`). There are no
pixel values, no hex colors, no arbitrary anything.

An atom is `[prefix:]name-value`. The names are Tailwind's; the values
are ours. Several utilities share a name and tell themselves apart by
the value you give them — `text-sm` is a size, `text-muted-foreground`
is a color, `text-center` is an alignment. Two atoms in the line above
look redundant and are not: `border` sets the width, `border-border`
sets the color to the `border` token.

`sx` is static. `sx={expr}` does not compile — a computed class string
is how a closed vocabulary stops being closed. Moving a *value* at
runtime is a different tool: the `(--name)` binding, which the Styling
chapter covers.

## The compiler holds the line

This is not a convention — it is enforced. Write an off-system value:

```
<column gap="4" sx="p-12px">
```

and the build stops with:

```
Error: line 1 col 21: [sail-sx] sx `p-12px` — `12px` is not a token (a value is a
design decision; spend a token). `p-` accepts: 0, 0_75, 1, 10, 11, 12, 14, 16,
1_25, 1_5, 2, 20, 24, 28, 2_5, 3, 32, 36, 3_5, 4, 40, 44, 48, 4_5, 5, 52, 56, 6,
60, 64, 7, 8, 9, density-icon, density-row-gap, density-row-px, density-row-py,
density-text, half, size-fs, size-h, size-px; did you mean `p-12`?
  suggest: sx is surface-only: spacing (p/m…, the Tailwind-aligned space ramp
  1..64 plus half-steps), colors (bg/fg), radius, text. Layout is the primitives'
  job (<row>/<column>/<grid>). Raw px/hex are not expressible.
```

Read that message closely — it carries the entire allowed vocabulary
for the atom you misused, plus a nearest guess. The rejection is the
documentation. Lesson 08 is built entirely on this idea.

Take the guess as a starting point rather than an answer: it offers
`p-12` because that is one character away from `p-12px`, but `12` is a
step on the ramp, not a pixel count. Pick the token that means what you
meant — here, `p-4`.

## States are prefixes

Interaction states are the same vocabulary behind a prefix — no
selectors, no stylesheet:

- `hover:` / `active:` / `disabled:` — `hover:bg-surface-2`,
  `hover:shadow-hover`
- `focus:` / `focus-within:` — `focus-within:border-primary` on a row
  that contains an input
- `current:` — applies when the element carries `aria-current`, which
  is how the solved example styles all three nav pills identically and
  lets the *attribute* pick the active one. Its siblings read the rest
  of the ARIA surface the same way: `selected:`, `expanded:`,
  `checked:`, `pressed:`, `busy:`, `open:`
- breakpoints, mobile-first: `mobile:` / `tablet:` / `desktop:`
- conditions: `dark:` / `light:` / `motion-reduce:` / `motion-safe:` /
  `forced-colors:`
- `group-<state>:` reacts to an ancestor marked `group`

That set is closed too — an unknown prefix is rejected with the whole
list, the same way an unknown value is.

## Try it

The starter has the structure; the surface is yours:

1. **The pills.** Give each link `px-3 py-1 rounded-full text-sm
   font-semibold`, muted ink with `hover:text-foreground`, and
   `current:bg-primary current:text-primary-foreground`. Only the link
   with `aria-current="page"` lights up.
2. **The card.** `p-5 bg-surface-1 border border-border rounded-lg
   shadow-sm`, then add `hover:shadow-hover` and watch it lift.
3. **The alert row.** Put `focus-within:border-primary` on the row
   around the input, then click into the input.
4. **Break it on purpose.** Change `p-5` to `p-24px`, run the build,
   and read your first sx rejection end to end.
