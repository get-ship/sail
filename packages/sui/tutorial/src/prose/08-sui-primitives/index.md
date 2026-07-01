---
title: "Composing UI primitives"
description: "Using the @ship/sui-primitives pack inside your app."
group: "Tutorial"
order: 8
slug: "08-sui-primitives"
prev: "07-event-modifiers"
next: "09-agent-loop"
chapter: "components"
---

Ship's primitive set ships as **`@ship/sui-primitives`** — about 80 custom
elements (`<sui-button>`, `<sui-card>`, `<sui-input>`, `<sui-tabs>`,
`<sui-status-dot>`, `<sui-alert>`, …) each with a stable prop / event /
slot contract. Charts live in a separate pack, **`@ship/sui-charts`**
(`<sui-area-chart>`, `<sui-bar-chart>`, `<sui-donut>`, …). Browse every
primitive, with live previews, in the [component library](/components).

You don't write `import { Button } from '@ship/sui-primitives'` and you
don't declare what tags your template uses. You import the pack once
at the app entrypoint (or rely on the daemon's auto-register in dev),
and from then on every `<sui-*>` tag in your template just works:

```ts
// app entry — registers every <sui-*> in the pack
import '@ship/sui-primitives'
import '@ship/sui-charts'        // optional
```

```sui
<sui-card>
  <sui-chip tone="primary" slot="header">draft</sui-chip>
  <p>{body()}</p>
  <sui-button on:click={save} tone="primary" slot="footer">Save</sui-button>
</sui-card>
```

Three things that are good to know:

- **Props are reactive.** Just like native attributes — `<sui-button
  disabled={busy()}>` re-evaluates on every change.
- **Events look native.** `<sui-button on:click={…}>`, no special
  prefix. Component-specific events are also `on:`-prefixed:
  `on:sui-change`, `on:sui-tab-change`.
- **Slots project.** Most components define named slots
  (`slot="header"`, `slot="footer"`, `slot="leading"`); the default
  slot takes the rest. The component owns the chrome — you own the
  content.

## Try it

The dashboard composes `<sui-card>`, `<sui-stat>`, `<sui-status-dot>`,
and `<sui-button>` into a small ops widget. Try:

1. **Pulse on alarm.** When `errorRate() > 5`, set the status dot's
   `tone="destructive"` and `pulse` attribute so it animates.
2. **Add a tone toggle.** Two `<sui-button>`s that flip the card's
   `tone` between `"default"` and `"warning"` — observe how the
   card chrome re-skins to match.
3. **Slot a sui-alert.** When `errorRate() > 5`, render a `<sui-alert
   tone="destructive">` inside the card with a "rate limit exceeded"
   message.
