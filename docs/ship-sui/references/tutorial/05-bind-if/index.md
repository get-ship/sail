---
title: "Conditional rendering"
description: "Branch templates with {#if}, {:else if}, {:else}."
group: "Tutorial"
order: 5
slug: "05-bind-if"
prev: "04-effect"
next: "06-bind-value"
chapter: "templating"
---

`{#if expr}` keeps a block in the DOM only while `expr` is truthy.
When the condition flips, SUI tears down the inactive branch and
mounts the active one — cleaning up effects, removing listeners, and
running any `onCleanup` hooks on the way out.

```sui
{#if count() > 0}
  <p>count is {count()}</p>
{:else if count() < 0}
  <p>count went negative</p>
{:else}
  <p>at zero — try +1</p>
{/if}
```

Three things to know:

- **Branches are scopes.** Each branch can declare its own controller
  state via nested components, and that state is discarded when the
  branch unmounts.
- **No virtual DOM diff.** Switching branches removes the old block
  and creates the new one, but doing the same work twice in a row is
  the same as a no-op — `count.set(5)` then `count.set(6)` doesn't
  re-mount a `> 0` branch that's already mounted.
- **The condition is reactive.** Any signal read inside the `{#if}`
  expression subscribes the branch evaluator, so flipping a signal is
  all it takes to switch branches.

## Try it

The example shows a tri-state form: an idle button, a "loading"
spinner, and a "done!" message. Open the **controller** tab. Try:

1. **Add an error branch.** Introduce a `state` signal of type
   `'idle' | 'loading' | 'done' | 'error'`, and add `{:else if
   state() === 'error'}` showing a retry button.
2. **Reset on done.** Add a "start over" button that fires inside the
   done branch and flips `state` back to `'idle'`.
3. **Observe lifecycle.** Wrap each branch's body in a `<section>`
   with an `effect(() => console.log('mounted:', state()))` inside a
   nested controller — watch the console as branches mount/unmount.
