---
title: "Template hooks — <use-*>"
description: "Element-shaped lifecycle / side effect, declared in the template."
group: "Tutorial"
order: 19
slug: "19-template-hooks"
prev: "18-untrack-batch"
next: "20-controller-hooks"
chapter: "composition"
---

A **template hook** is a custom element of the form `<use-name>`.
It doesn't render to the DOM — it attaches a host-side side effect
(or contributes a scope identifier) and tears down on unmount.

```sui
<use-title text="SUI tutorial · step 19" />

<use-kbd>
  <on key="/"      action="toggleSearch" />
  <on key="cmd+n"  action="newAgent" />
  <on key="escape" action="clearSelection" />
</use-kbd>

<dialog bind:this={dialogEl}>…</dialog>
<use-click-outside bind:target={dialogEl} action="close" />
```

Three patterns:

| Pattern | Example | What it does |
|---|---|---|
| **Literal attr** | `<use-title text="…" />` | Hook receives a string verbatim. Applied on mount, restored on unmount. |
| **Element ref** | `<use-click-outside bind:target={ref} action="close" />` | Hook gets a handle to a controller-side element / value. |
| **Template-fragment children** | `<use-kbd><on key="…" /></use-kbd>` | The hook walks its children for declarative configuration. |

The compiler treats `<use-*>` as a registered hook lookup — unknown
hook names are a compile error, not a silent no-op. Today's hooks:

- `<use-title>` — set `document.title` on mount, restore on unmount
- `<use-kbd>` — document-scoped keyboard shortcuts
- `<use-click-outside>` — close-on-outside-click
- `<use-focus-trap>` — trap tab focus inside a region
- `<use-clipboard>` — copy / paste plumbing
- `<use-scroll-anchor>` — preserve scroll position across re-renders
- `<use-intersection>` — IntersectionObserver wrapper

### Reactive title? Use `effect`, not the hook

`<use-title>` is for **lifecycle-bound** side effects. If you want a
title that follows a signal, the right primitive is `effect` in the
controller (covered in step 4):

```ts
import { controller, signal, effect } from '@ship/sui-runtime'

export default controller<Scope>({
  scope: () => {
    const count = signal(0)
    effect(() => { document.title = `count: ${count()}` })
    return { count }
  },
})
```

The hook and the effect have different jobs: the hook fires once on
mount and tears down on unmount; the effect re-runs on every
dependency change. Picking the wrong one is a leak or a missed
re-render — not a syntax error, which is why it's worth learning
the boundary.

## Try it

The example uses `<use-title>` to set the browser tab title and
`<use-kbd>` for keyboard shortcuts. Try:

1. **Look at the tab title**: the title is set once on mount. It
   doesn't follow `count()` — that's the lifecycle-bound contract.
2. **Press the shortcuts**: `j`/`k` to move the count, `r` to reset.
   The bindings live in the template, not in a `useEffect` /
   `addEventListener` you wrote by hand.
3. **Make the title reactive**: add `effect(() => { document.title =
   \`count: ${count()}\` })` in the controller. Now the title follows
   the counter. Compare this with `<use-title text="…">` — the
   effect is the right primitive when the value tracks a signal.
4. **Add `<use-clipboard />`** somewhere and try the copy hook —
   another platform-bridge hook that contributes a controller-callable
   helper without manual setup.
