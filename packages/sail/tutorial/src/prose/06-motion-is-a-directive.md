---
title: "Motion is a directive"
description: "Triggers say when, presets say what, inline values say exactly how much — enter, exit-hold, stagger, loop, and the reduced-motion contract."
order: 6
slug: "06-motion-is-a-directive"
prev: "05-data-and-forms"
next: "07-themes-are-one-attribute"
chapter: "system"
---

There is exactly one animation surface in the language:

```
fx:TRIGGER={{ property: 'preset' }}
```

The **trigger** says when: `hover`, `press`, `focus`, `focus-visible`
(interaction), `enter`, `exit` (lifecycle), `reveal`, `scroll`
(scroll-timeline), `loop` (ambient). The **body** says what:
`scale`, `scale-x`, `scale-y`, `opacity`, `translate-x`, `translate-y`,
`rotate`, each mapped to a preset — `fade`, `fade-out`, `press`,
`release`, `pulse`, `spin`, `bounce`, `marquee`, `marquee-reverse`, and
the four springs (`spring:snappy`, `spring:gentle`, `spring:bouncy`,
`spring:deliberate`). An unknown preset name fails the build.

The solved tray deliberately varies the vocabulary — rotate on the
restock hint, scale on the trigger button, pulse looping on the unread
dot, translate-x on the panel, staggered rows. Five fade-up-from-below
animations in a row is a design smell; the vocabulary is wider than
that.

## Presets are defaults, not a cage

A preset is the teachable starting point. When you need a specific
animation, write it:

```
style({ animate: loop({ translate-x: { from: 0, to: '120px' },
                        rotate: { to: '15deg' },
                        duration: 3600, easing: [0.6, 0, 0.2, 1] }) })
```

One rule governs every value: **quoted is a literal, unquoted is code.**
A bare number is milliseconds for `duration`/`delay`, and the channel's
native unit for a magnitude (px for translate, deg for rotate, unitless
for scale and opacity). A quoted string carries its own unit — `'-12%'`,
`'0.25turn'`. `easing` takes a keyword (`'in-out'`, `'linear'`) or an
inline cubic-bezier. You can also start from a preset and override one
field: `scale: { preset: 'pulse', to: 1.9 }`.

This is where `fx:` differs from `sx` on purpose. sx is closed
because colour and spacing are theming integrity — an off-system colour
is a bug. A 3600ms decorative loop is not a theming decision, so it does
not need a token to exist before you can write it.

## Signals: config that follows state

Any field can be an expression, which is how a signal gets in:

```
style({ animate: loop({ translate-x: { to: dist() }, duration: speed() }) })
```

When the signal changes, the animation is re-created with the new
values, and it is cleaned up when the element goes away. Note what this
is: reactive **config** — the signal says what the animation *is*. A
signal that drives a value every *frame* is the other lane: derive it in
the `<controller>` and reflect it through sx with `(--name)`. Reach for
that one to scrub; each change on `fx:` costs an animation teardown.

## exit is why the framework is involved

CSS can animate an element in; it cannot animate one *out*, because by
then the node is gone. `style({ animate: exit({…}) })` is the exception
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
hand-authored cases, sx has `motion-reduce:` and `motion-safe:`
prefixes.)

## Try it

The starter works perfectly and moves not at all. Add only directives —
the controller does not change:

1. **Make dismiss graceful** (TODO 5). `style({ animate: exit({ opacity:
   'fade-out' }) })` on the row. Before you add it, dismiss one and watch
   it snap; after, watch exit-hold do its job.
2. **Make arrival a cascade** (TODO 5). `style({ animate: enter({…}) })`
   with a spring on `translate-x` and `stagger: '50ms'`, then hit Restock.
3. **Panel lifecycle** (TODO 4). Enter and exit on the `<if {open()}>`
   panel — toggle the tray both ways.
4. **The small stuff** (TODO 1–3). Rotate the restock hint on hover,
   press-scale the trigger, pulse the unread dot with
   `style({ animate: loop({…}) })`.
5. **Try to cheat.** Set a preset to `'whoosh'` and read the rejection —
   it lists the entire legal set, and points at the inline form.
6. **Then stop cheating and just write it.** Replace one preset with an
   inline object — `scale: { from: 1, to: 1.2 }, duration: 900` — and
   watch it need no new vocabulary at all.
