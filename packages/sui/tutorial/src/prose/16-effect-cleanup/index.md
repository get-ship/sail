---
title: "Effect cleanup + nested effects"
description: "Per-firing cleanup. Nested effects scoped to their parent."
group: "Tutorial"
order: 16
slug: "16-effect-cleanup"
prev: "15-onmount-oncleanup"
next: "17-selector"
chapter: "lifecycle"
---

You met `effect()` back in step 4. Two refinements that come up
constantly in real code:

## The returned cleanup runs between firings

An `effect()` can return a cleanup function. That function runs:
- **Before** the effect's next firing, AND
- **At unmount** (the last firing's cleanup runs last)

This is how you avoid leaks when an effect re-subscribes to something
because one of its dependencies changed.

```ts
effect(() => {
  const url = endpoint()                        // signal read
  const ctrl = new AbortController()

  fetch(url, { signal: ctrl.signal })
    .then((r) => r.json())
    .then(setData)
    .catch((e) => { if (e.name !== 'AbortError') console.error(e) })

  return () => ctrl.abort()  // aborts the in-flight request when endpoint() changes
})
```

If `endpoint()` changes mid-flight, the cleanup aborts the prior
request before the new one starts. No "stale response races latest"
bug.

## Nested effects scope to their parent

You can declare `effect()` inside another `effect()`. Inner effects
have their **own** cleanup, and that cleanup runs every time the
outer effect re-fires. This is how you express "for each selected
row, run a sub-effect."

```ts
effect(() => {
  for (const id of selectedIds()) {
    effect(() => {
      const row = rowsById()[id]
      if (!row) return
      analytics.track('row-visible', { id, status: row.status })
    })
  }
})
```

When `selectedIds()` changes, the outer cleans up all inner effects
before re-running, so old subscriptions don't pile up.

## Try it

The example is a fake live-data subscription that the user can
re-target with a select. Try:

1. **Watch the console**: each time you change the endpoint, the
   prior abort fires before the new fetch starts.
2. **Slow the network**: set Chrome devtools to throttle ("Slow 3G").
   Switch endpoints rapidly. Verify only the last request's response
   ever lands.
3. **Add a tag effect**: inside the same outer `effect`, log a
   prefixed message — observe both fire on the same dep change, in
   order, and both cleanups run before the next outer firing.
