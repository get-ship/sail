---
title: "Refs with bind:this"
description: "Capture a DOM element into a signal."
group: "Tutorial"
order: 13
slug: "13-refs"
prev: "12-slots"
next: "14-class-style"
chapter: "composition"
---

Sometimes you need a handle on the actual DOM node — to focus an
input, measure something, observe scroll, hand a canvas to a chart
library. `bind:this={someSignal}` captures the element ref into the
signal at mount time, and clears it (back to `null`) on unmount.

```sui
<input bind:this={inputEl} placeholder="…" />
```

```ts
const inputEl = signal<HTMLInputElement | null>(null)

// later: read the ref inside an effect
effect(() => {
  inputEl()?.focus()
})
```

Three things to know:

- **Null until mount.** The ref starts `null` and is only populated
  once the runtime attaches. Wrap reads in `?.`.
- **Cleared on unmount.** When the element leaves the DOM (a `{#if}`
  branch swap, a `{#for}` row removed), the signal goes back to
  `null` — automatically.
- **Read in `effect` to react.** Reading the ref inside `effect()`
  subscribes — your code runs once when it lands and again whenever
  it changes (re-mount = new instance).

## Try it

The example focuses an input on mount and tracks its width via
`ResizeObserver`. Try:

1. **Focus on a button click**: add a "focus" button that calls
   `inputEl()?.focus()` on click. Compare with the effect-on-mount
   approach.
2. **Read selection**: log `inputEl()?.selectionStart` every time
   the user types — wire it via an `effect` that reads the input's
   value signal and the ref.
3. **Conditional ref**: wrap the input in `{#if showInput()}` —
   observe the ref nulls out when the branch unmounts and re-binds
   to a fresh element when it re-mounts.
