---
title: "Styling is Style"
description: "Object Style, a flat chain, when for states — tokens only, never utility strings."
order: 2
slug: "02-styling-is-sx"
prev: "01-a-page-is-markup"
next: "03-state-is-a-controller"
chapter: "surface"
---

When a tag and its props are not enough, attach **Style** as a call on the
host — not an HTML attribute:

```
<column style({ pad(5).bg(surface-1).border({ color: border, width: 1 }).radius(lg).elevation(sm), when: { hover: elevation(hover) } })>
  …
</column>
```

— and every value is a **token from a closed list**. Spacing is the
scale (`pad(4)`, `pad({ x: 3 })`, `margin({ b: 2 })`), color is the theme table
(`bg(surface-1)`, `fg(muted-foreground)`, `border({ color: primary })`), type is
the type scale (`type({ size: sm })`, `type({ weight: semibold })`,
`type({ tracking: tight })`). There are no pixel values, no hex colors, no
arbitrary anything.

Product form is the **object Style**: `style({ pad(4).bg(surface-1),
animate: … })` with optional keys for conditionality — `when:`, `match:`,
`compound:`. Setters chain — `bg`, `fg`, `pad`, `margin`, `position`, `type`,
`border({…})`, `radius`, `elevation` — not Tailwind class names. Two related
ideas stay distinct: `border({ width: 1 })` sets the width; `border({ color: border })` sets the color
to the `border` token.

Prefer a scope Style declaration when the same paint repeats:

```
<scope>
  style card = { pad(5).bg(surface-1).border({ color: border, width: 1 }).radius(lg) }
</scope>
<column style(card)>
  …
</column>
```

Or compose Style pieces as args: `style(base, emphasis)` — and in scope,
`style quiet = host, { bg(transparent) }`. That is Style
composition on a host, not an in-file `<partial>` (template reuse). Computed
class strings are not the product — utility-string atoms like
`style="p-4 bg-muted"` do not compile.

## Values that track state: `bind(name)`

When a style value *is* a prop, signal, derived, token, or action-written
channel, name it — never invent CSS:

```
// number prop → length (px)
style({ position({ t: bind(offset) }) })

// setup-returned derived (annotate number for length setters)
const doubled = derived((): number => offset() * 2)
// …
style({ size({ w: bind(doubled) }) })

// action write (manifest `writes`)
<div use:anchor style({ position({ l: bind(anchorLeft), t: bind(anchorTop) }) })>
```

`name` is an identifier only — no `bind(derived(…))`, no `bind(a + b)`.
Number→length means px; number→color is a compile error. Enumerating a
prop as three-plus compound rows that only vary a setter arg is a lint
(`sail-style-value-enumeration`); collapse it to `bind`.

## The compiler holds the line

This is not a convention — it is enforced. Write an off-system value:

```
<column gap="4">
  style({ pad(12px) })
```

and the build stops with:

```
Error: line 1 col 21: [sail-style] Style error: style `pad(12px)` — value is not a token on this
setter's scale
```

Read that message closely — it names the setter and that the scale is
closed. The rejection is the documentation. Lesson 08 is built entirely
on this idea.

`12px` is not on the spacing scale. Pick the token that means what you
meant — here, `pad(4)`.

## States are when: {…}

Interaction states are the same vocabulary inside a map — no selectors,
no stylesheet, no `hover:` string prefixes:

- `hover` / `active` / `disabled` — `style({ when: { hover: bg(surface-2) } })`,
  `style({ when: { hover: elevation(hover) } })`
- `focus` / `focusWithin` — `style({ when: { focusWithin: border({ color: primary }) } })`
  on a row that contains an input
- `current` — applies when the element carries `aria-current`, which is
  how the solved example styles all three nav pills identically and lets
  the *attribute* pick the active one. Siblings on the ARIA surface work
  the same way: `selected`, `expanded`, `checked`, `pressed`, `busy`,
  `open`
- breakpoints, mobile-first: `style({ when: { tablet: { … }, desktop: { … } } })`
  via Style conditions locked to `breakpoint-*` tokens — **not**
  `tablet:` utility prefixes
- preference media: `dark` / `light` / `motionReduce` / `motionSafe` /
  `forced-colors`
- group descendants: `group-hover` / `group-focus-within` (ancestor
  marked with `style({ group() })`)

That set is closed too — an unknown key is rejected with the whole
list, the same way an unknown value is.

## Try it

The starter has the structure; the surface is yours:

1. **The pills.** A scope const with `pad({ x: 3, y: 1 })`,
   `radius(full)`, `type({ size: sm, weight: semibold })`, muted `fg` with
   `when: { hover: fg(foreground).bg(surface-2), current:
   bg(primary).fg(primary-foreground) }`. Only the link with
   `aria-current="page"` lights up.
2. **The card.** `pad(5)`, `bg(surface-1)`,
   `border({ color: border, width: 1 })`, `radius(lg)`, `elevation(sm)`, then add
   `when: { hover: elevation(hover) }` and watch it lift.
3. **The alert row.** Put `when: { focusWithin: border({ color: primary }) }`
   on the row around the input, then click into the input.
4. **Break it on purpose.** Change `pad(5)` to `pad(24px)`, run the
   build, and read your first Style rejection end to end.
