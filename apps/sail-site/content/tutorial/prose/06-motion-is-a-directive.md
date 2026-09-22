---
title: "Motion is a directive"
description: "Triggers say when, presets say what, inline values say exactly how much — enter, exit-hold, stagger, loop, and the reduced-motion contract."
order: 6
slug: "06-motion-is-a-directive"
prev: "05-data-and-forms"
next: "07-themes-are-one-attribute"
chapter: "system"
---

There is exactly one animation surface in the language — Style's
`animate` channel:

```
style({ animate: { enter: play(fade), exit: play(fade) } })
style({ animate: { hover: play(fade), press: play(fade) } })
style(name, { animate: { loop: play(fade) } })
```

The **trigger** is the map key: `transition` (props on state change),
`enter`, `exit`, `reveal`, `loop`, `scroll` — plus listed interaction
keys `hover` / `press` / `focus`. The **concern** inside is `play`:
`play(fade)`, `play(slide-up)`, `play(colors, fast)`. An unknown play
name fails the build. Hover/press *paint* stays `when: { hover: … }`
plus `animate: { transition: play(colors, fast) }`.

The solved tray deliberately varies the vocabulary — rotate on the
restock hint, scale on the trigger button, pulse looping on the unread
dot, translate-x on the panel, staggered rows. Five fade-up-from-below
animations in a row is a design smell; the vocabulary is wider than
that.

## Presets are defaults, not a cage

A preset is the teachable starting point. When you need a specific
animation, write it:

```
style({ animate: { enter: play(slide-up), transition: play(colors, fast) } })
```

`play` is the only author concern. Theme plays (`fade`, `slide-up`,
`fade-scale-in`, `colors`) own the from/to. A second argument is a
duration token on `transition`: `play(colors, fast)`.

This is where `animate` differs from paint on purpose. Paint is closed
for theming integrity — an off-system colour is a bug. Motion is the
same closed table: you pick a play, you do not invent a second syntax.

## Signals: config that follows state

Any field can be an expression, which is how a signal gets in:

```
style({ animate: { loop: play(fade) } })
```

When the signal changes, the animation is re-created with the new
values, and it is cleaned up when the element goes away. Note what this
is: reactive **config** — the signal says what the animation *is*. A
signal that drives a value every *frame* is the other lane: derive it in
the `<controller>` and reflect it through Style `bind` (or a length
channel). Reach for that one to scrub; each change on `animate` config
costs an animation teardown.

## exit is why the framework is involved

CSS can animate an element in; it cannot animate one *out*, because by
then the node is gone. `style({ animate: { exit: play(fade) } })` is the exception
the framework owns: it **holds the element in the tree** until the exit
animation finishes, then removes it. Dismiss a notification in the solved
example — the row fades, then leaves. That is exit-hold.

## stagger is a relationship

`stagger: '50ms'` (or `stagger: 50`) is a reserved key, and its value is
a **duration**, not a preset — the delay step *between siblings*
arriving together. It is what turns a list that appears into a list that
arrives. Distinct from `delay`, which is one animation's own offset; set
both and they compose.

## The reduced-motion contract

You write none of this conditionally. Under
`prefers-reduced-motion`, ambient `loop` animations stop and
transitions collapse toward instant — the contract lives in the system,
not in each call site. This holds for inline durations too: reduced
motion is enforced once, in the runtime, so a hand-written `duration:
3600` is stopped by exactly the same authority as a preset. (For
hand-authored cases, Style `when` can still gate motion-safe/reduce
overrides when you need an explicit branch.)

## Try it

The starter works perfectly and moves not at all. Add only directives —
the controller does not change:

1. **Make dismiss graceful** (TODO 5). `style({ animate: { exit: play(fade) } })`
   on the row. Before you add it, dismiss one and watch
   it snap; after, watch exit-hold do its job.
2. **Make arrival a cascade** (TODO 5). `style({ animate: { enter: play(slide-up) } })`
   on the row, then hit Restock.
3. **Panel lifecycle** (TODO 4). Enter and exit on the `<if {open()}>`
   panel — toggle the tray both ways.
4. **The small stuff** (TODO 1–3). Hover play on the restock hint,
   press play on the trigger, loop play on the unread dot with
   `style({ animate: { loop: play(fade) } })`.
5. **Try to cheat.** Write `play(whoosh)` and read the rejection —
   unknown plays fail closed.
