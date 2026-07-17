---
title: "Motion is a directive"
description: "Triggers say when, presets say what — enter, exit-hold, stagger, loop, and the reduced-motion contract."
order: 6
slug: "06-motion-is-a-directive"
prev: "05-data-and-forms"
next: "07-themes-are-one-attribute"
chapter: "system"
---

There is exactly one animation surface in the language:

```
motion:TRIGGER={{ property: 'preset' }}
```

The **trigger** says when: `hover`, `press`, `focus`, `focus-visible`
(interaction), `enter`, `exit` (lifecycle), `view`, `scroll`
(scroll-timeline), `loop` (ambient). The **body** says what:
`scale`, `opacity`, `translate-x`, `translate-y`, `rotate`, each mapped
to a preset from a closed set — `fade`, `fade-out`, `press`, `release`,
`pulse`, and the four springs (`spring:snappy`, `spring:gentle`,
`spring:bouncy`, `spring:deliberate`). An unknown preset fails the
build, exactly like an off-system sx token.

The solved tray deliberately varies the vocabulary — rotate on the
restock hint, scale on the trigger button, pulse looping on the unread
dot, translate-x on the panel, staggered rows. Five fade-up-from-below
animations in a row is a design smell; the vocabulary is wider than
that.

## exit is why the framework is involved

CSS can animate an element in; it cannot animate one *out*, because by
then the node is gone. `motion:exit` is the exception the framework
owns: it **holds the element in the tree** until the exit animation
finishes, then removes it. Dismiss a notification in the solved example
— the row fades, then leaves. That is exit-hold.

## stagger is a relationship

`stagger: '50ms'` is a reserved key, and its value is a **duration**,
not a preset — the delay step *between siblings* arriving together.
It is what turns a list that appears into a list that arrives.

## The reduced-motion contract

You write none of this conditionally. Under
`prefers-reduced-motion`, ambient `loop` animations stop and
transitions collapse toward instant — the contract lives in the system,
not in each call site. (For hand-authored cases, sx has `motion-reduce:`
and `motion-safe:` prefixes.)

## Try it

The starter works perfectly and moves not at all. Add only directives —
the controller does not change:

1. **Make dismiss graceful** (TODO 5). `motion:exit={{ opacity:
   'fade-out' }}` on the row. Before you add it, dismiss one and watch
   it snap; after, watch exit-hold do its job.
2. **Make arrival a cascade** (TODO 5). `motion:enter` with a spring on
   `translate-x` and `stagger: '50ms'`, then hit Restock.
3. **Panel lifecycle** (TODO 4). Enter and exit on the `{#if open()}`
   panel — toggle the tray both ways.
4. **The small stuff** (TODO 1–3). Rotate the restock hint on hover,
   press-scale the trigger, pulse the unread dot with `motion:loop`.
5. **Try to cheat.** Set a preset to `'whoosh'` and read the rejection —
   it lists the entire legal set.
