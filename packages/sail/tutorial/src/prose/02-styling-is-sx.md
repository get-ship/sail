---
title: "Styling is Style"
description: "One attribute, style.* chains, when for states — tokens only, never utility strings."
order: 2
slug: "02-styling-is-sx"
prev: "01-a-page-is-markup"
next: "03-state-is-a-controller"
chapter: "surface"
---

When a tag and its props are not enough, there is exactly one place to
go: the `sx` attribute. Paint on it is the **Style** value language —

```
<column sx={style.pad(5).bg(surface-1).borderWidth(1).border(border).radius(lg).shadow(sm).when({ hover: style.shadow(hover) })}>
```

— and every value is a **token from a closed list**. Spacing is the
scale (`pad(4)`, `padX(3)`, `mb(2)`), color is the theme table
(`bg(surface-1)`, `fg(muted-foreground)`, `border(primary)`), type is
the type scale (`fontSize(sm)`, `fontWeight(semibold)`,
`tracking(tight)`). There are no pixel values, no hex colors, no
arbitrary anything.

A chain is `style.setter(token).setter(token)…`. Setters are the product
surface — `bg`, `fg`, `pad`, `radius`, `shadow`, `fontSize` — not Tailwind
class names. Two related ideas stay distinct: `borderWidth(1)` sets the
width; `border(border)` sets the color to the `border` token.

Prefer a scope const when the same paint repeats:

```
<scope>
  const card = style.pad(5).bg(surface-1).borderWidth(1).border(border).radius(lg)
</scope>
<column sx={card}>…</column>
```

Or compose layers with an array: `sx={[base, emphasis]}`. Computed class
strings are not the product — utility-string atoms like
`sx="p-4 bg-muted"` do not compile. Moving a *value* at runtime is a
different tool: `bind(name)`, which the Styling chapter covers.

## The compiler holds the line

This is not a convention — it is enforced. Write an off-system value:

```
<column gap="4" sx={style.pad(12px)}>
```

and the build stops with:

```
Error: line 1 col 21: [sail-sx] style `pad(12px)` — value is not a token on this
setter's scale
  suggest: sx is surface-only: spacing (p/m…, the Tailwind-aligned space ramp
  1..64 plus half-steps), colors (bg/fg), radius, text. Layout is the primitives'
  job (<row>/<column>/<grid>). Raw px/hex are not expressible.
```

Read that message closely — it names the setter and that the scale is
closed. The rejection is the documentation. Lesson 08 is built entirely
on this idea.

`12px` is not on the spacing scale. Pick the token that means what you
meant — here, `pad(4)`.

## States are when({…})

Interaction states are the same vocabulary inside a map — no selectors,
no stylesheet, no `hover:` string prefixes:

- `hover` / `active` / `disabled` — `when({ hover: style.bg(surface-2) })`,
  `when({ hover: style.shadow(hover) })`
- `focus` / `focusWithin` — `when({ focusWithin: style.border(primary) })`
  on a row that contains an input
- `current` — applies when the element carries `aria-current`, which is
  how the solved example styles all three nav pills identically and lets
  the *attribute* pick the active one. Siblings on the ARIA surface work
  the same way: `selected`, `expanded`, `checked`, `pressed`, `busy`,
  `open`
- breakpoints, mobile-first: `when({ tablet: style…, desktop: style… })`
  via Style conditions locked to `breakpoint-*` tokens — **not**
  `tablet:` utility prefixes
- preference media: `dark` / `light` / `motionReduce` / `motionSafe` /
  `forced-colors`
- group descendants: `group-hover` / `group-focus-within` (ancestor
  marked with `style.group()`)

That set is closed too — an unknown key is rejected with the whole
list, the same way an unknown value is.

## Try it

The starter has the structure; the surface is yours:

1. **The pills.** A scope const with `padX(3).padY(1).radius(full)
   .fontSize(sm).fontWeight(semibold)`, muted ink with
   `when({ hover: style.fg(foreground).bg(surface-2), current:
   style.bg(primary).fg(primary-foreground) })`. Only the link with
   `aria-current="page"` lights up.
2. **The card.** `pad(5).bg(surface-1).borderWidth(1).border(border)
   .radius(lg).shadow(sm)`, then add
   `when({ hover: style.shadow(hover) })` and watch it lift.
3. **The alert row.** Put `when({ focusWithin: style.border(primary) })`
   on the row around the input, then click into the input.
4. **Break it on purpose.** Change `pad(5)` to `pad(24px)`, run the
   build, and read your first Style rejection end to end.
