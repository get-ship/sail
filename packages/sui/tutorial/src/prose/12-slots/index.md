---
title: "Slots"
description: "Project content into a component's named regions."
group: "Tutorial"
order: 12
slug: "12-slots"
prev: "11-sui-eval"
next: "13-refs"
chapter: "composition"
---

Slots are how a component **invites you** into its chrome. The
component declares regions (`<slot name="header">`, default `<slot>`,
`<slot name="footer">`) and you fill them with arbitrary content from
the call site. SUI uses standard web-component slot semantics —
resolved in light DOM at mount time, so the slotted content stays
under the call-site scope (your signals, your handlers).

```sui
<sui-card>
  <sui-chip slot="header" tone="primary">draft</sui-chip>
  <p>This goes in the default slot.</p>
  <button slot="footer" on:click={save}>Save</button>
</sui-card>
```

Three things to know:

- **Reactive in the slotted scope.** A signal read inside a `slot=`
  child re-renders the same way it would outside — the slot doesn't
  break reactivity.
- **`reflect-empty`.** Components that care whether a slot was filled
  (sui-card, sui-alert) set a host attribute when the slot is empty
  (`data-no-header`, `data-no-footer`) and trim chrome accordingly.
- **Default slot is the catch-all.** Anything without a `slot=`
  attribute lands in the unnamed slot.

## Try it

The example uses `<sui-card>` and `<sui-alert>` to build a small
notification widget. Try:

1. **Add a footer**: slot a `<button slot="footer">` into the
   card so it gets the dedicated footer chrome.
2. **Empty header**: remove the `<sui-chip slot="header">` and
   observe the card collapses the header region via the
   `reflect-empty` mechanism.
3. **Nested**: put a second `<sui-card>` inside the default slot. It
   renders inline, with its own chrome — composition all the way down.
