---
title: "Projection"
description: "Project content into a component's named regions."
group: "Tutorial"
order: 12
slug: "12-projection"
prev: "11-sui-eval"
next: "13-refs"
chapter: "composition"
---

Projection is how a component **invites you** into its chrome. The
component declares regions in its **anatomy manifest** (and a
`<slot>` element in its body for each region), and you fill them
with arbitrary content from the call site. The runtime projects
children into the matching `<slot>` wrapper at connect time, in
light DOM — there is no shadow root, no native slot mechanics.

```sui
<sui-card>
  <sui-chip slot="header" tone="primary">draft</sui-chip>
  <p>This is the body — it lands in the default region.</p>
  <sui-button slot="footer" on:click={save}>Save</sui-button>
</sui-card>
```

Three things to know:

- **Reactive in the projected scope.** A signal read inside a
  projected child re-renders the same way it would outside — the
  projection doesn't break reactivity.
- **Empty-region reflection.** Components that care whether a region
  was filled (`sui-card`, `sui-alert`) declare a `<slot reflect-empty="…">`
  directive in the body. The runtime helper `bindSlotEmpty`
  (`packages/sui/runtime/src/slot-empty.ts`) toggles a host
  attribute based on whether the slot is empty, and the skin trims
  chrome accordingly. `[SPEC-PENDING: reflect-empty-attribute-name]`
  — the previous `data-no-header` / `data-no-footer` host-attribute
  pattern is replaced by a `reflect-empty="…"` directive on the
  `<slot>` element, and the compiler emits a `bindSlotEmpty` call
  per directive. The final attribute name and shape are the design
  call.
- **Default region is the catch-all.** Anything projected without a
  `slot="…"` attribute lands in the unnamed `<slot>`.

## Try it

The example uses `<sui-card>` and `<sui-alert>` to build a small
notification widget. Try:

1. **Add a footer**: project a `<sui-button slot="footer">` into
   the card so it gets the dedicated footer chrome.
2. **Empty header**: remove the `<sui-chip slot="header">` and
   observe the card collapses the header region via the
   `reflect-empty` mechanism.
3. **Nested**: put a second `<sui-card>` inside the default region.
   It renders inline, with its own chrome — composition all the way
   down.

---

## The model in one screen

- **Component declares regions in its anatomy manifest AND in its
  body.** A `<sui-card>` manifest lists `header`, `footer`, and
  `default` as named regions it accepts projection into. Its body
  carries a `<slot name="header">`, `<slot>`, and `<slot
  name="footer">` at the positions the projected children should
  land. The regions are the *published contract*; consumers see
  them in the component's custom-elements manifest, just like
  props and events.
- **Consumer opts in by naming the region.** A projected child
  carries a `slot="<name>"` attribute (matching the corresponding
  `<slot name="<name>">`). An un-named child goes to the default
  `<slot>`.
- **The runtime projects at connect.** At connect time, the
  runtime reads the slot attributes on the host's captured
  children and moves each node into the matching `<slot>` wrapper
  (replacing the slot's fallback content). The children live in
  the host's subtree from first paint; projection is a one-shot
  rearrange, not a redistribution.
- **The host is the element.** `<sui-card>` is just a `<sui-card>`
  in the page's tree. Its children are *its* children in the light
  DOM — queryable with a normal `querySelector`, addressable by
  `label[for]` / `aria-describedby`, participating in forms
  natively. No shadow root, no slot mechanics, no
  `formAssociated` plumbing.

## What this is not

- **Not a native slot mechanism.** Native `<slot>` only works
  inside a shadow root. Under light DOM, `<slot>` is a
  runtime-managed projection wrapper (the light contracts sheet
  sets `slot { display: contents }` so the wrapper is
  transparent). The runtime reimplements the assignment semantics
  in light DOM; the *contract* (the `slot=` attribute, the named /
  default / fallback model) is the same as web components, the
  *implementation* lives in `packages/sui/runtime/src/slot-project.ts`.
- **Not event retargeting.** Native slot semantics retarget event
  targets to the host. Under light DOM there is nothing to retarget;
  events fire on the children as expected. If a consumer relies on
  the host firing the event instead, attach the listener on the
  host element — the standard DOM way.
- **Not style isolation by encapsulation.** Component rules compile
  to `@scope (sui-x) to ([data-sui])` donuts so a component styles
  its own subtree and stops at nested component boundaries. The
  isolation is **scope**, not a shadow boundary. `[SPEC-PENDING:
  scope-anchor]` — the `data-sui` attribute the scope closes on is
  a convention the compiler is responsible for placing.

## Reading the anatomy manifest

A `<sui-card>` component's anatomy (CEM-enriched
`custom-elements.json`) carries a regions block:

```jsonc
{
  "name": "sui-card",
  "anatomy": {
    "regions": [
      { "name": "header",  "description": "Optional header chrome." },
      { "name": "footer",  "description": "Optional footer chrome." },
      { "name": "default", "description": "Card body. Catch-all." }
    ]
  }
}
```

Consumers read this the same way they read `props` and `events` —
the LSP and `@ship/sui-eval` can validate projection declarations
against the manifest. An undeclared region is a compile error in
strict mode and a warning otherwise.

`[SPEC-PENDING: anatomy-manifest-shape]`. The exact JSON shape
above is illustrative; the manifest schema (CEM-extension vs. a
sui-side schema) is its own design work.

## Common recipes

### Card with header / body / footer

```sui
<sui-card>
  <sui-chip slot="header" tone="primary">draft</sui-chip>
  <p>This is the body — it lands in the default region.</p>
  <sui-button slot="footer" on:click={save}>Save</sui-button>
</sui-card>
```

### Empty-state component

```sui
<sui-empty-state>
  <sui-icon slot="icon" name="terminal"></sui-icon>
  <span slot="title">No sessions running</span>
  <p>Create your first one.</p>
</sui-empty-state>
```

### Alert with an action button

```sui
<sui-error-state title="Couldn't load items">
  {err.message}
  <sui-button slot="action" on:click={retry}>Retry</sui-button>
</sui-error-state>
```

The action button ends up in the alert's footer chrome because
`action` is the region the manifest binds to the footer slot; the
attribute name is consumer-facing, the wiring is internal to the
component.

### Button with leading icon

```sui
<sui-button>
  <sui-icon slot="leading" name="plus"></sui-icon>
  New agent
</sui-button>
```

## How the runtime carries it out

The runtime reimplements the `slot=` / `<slot>` contract in light
DOM (`packages/sui/runtime/src/slot-project.ts`). The semantics
match shadow slotting one-for-one:

- a node with `slot="<name>"` goes to `<slot name="<name>">`
- nodes without a `slot` attribute (elements and non-empty text)
  go to the default `<slot>`
- nodes with no matching slot are removed (unslotted content
  does not render in shadow DOM either)
- slot fallback content is dropped when anything is assigned, kept
  when nothing is

The projection runs once at connect. Children appended to the host
later are not re-projected — the dynamic composition path in SUI
templates goes through compiled call sites, not raw DOM appends,
so this covers the authored surface.
