---
title: "Composing UI primitives"
description: "Using the @ship/primitives pack inside your app."
group: "Tutorial"
order: 8
slug: "08-sui-primitives"
prev: "07-event-modifiers"
next: "09-agent-loop"
chapter: "components"
---

Ship's primitive set ships as **`@ship/primitives`** — about 80 custom
elements (`<sui-button>`, `<sui-card>`, `<sui-input>`, `<sui-tabs>`,
`<sui-status-dot>`, `<sui-alert>`, …) each with a stable prop / event /
slot contract. Charts live in a separate pack, **`@ship/charts`**
(`<sui-area-chart>`, `<sui-bar-chart>`, `<sui-donut>`, …). Both
packs are surfaced in the workspace catalog at
`mcp__ship__ReadMcpResource(ship://components)` and in the live
component picker.

You don't write `import { Button } from '@ship/primitives'` and you
don't declare what tags your template uses. You import the pack once
at the app entrypoint (or rely on the daemon's auto-register in dev),
and from then on every `<sui-*>` tag in your template just works:

```ts
// app entry — registers every <sui-*> in the pack
import '@ship/primitives'
import '@ship/charts'        // optional
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
- **Projected children fill regions.** Most components accept named regions (`slot="header"`, `slot="footer"`, `slot="leading"`); the unnamed region takes the rest. The component's body has a matching `<slot name="…">` for each region; the runtime (`@ship/sui-runtime/slot-project.ts`) projects consumers' `slot="…"` children into the matching wrapper at connect. The component owns the chrome — you own the content. See [tutorial 12 (Projection)](../12-slots/) for the full model.

## Packs vs. the umbrella

Ship used to ship a single `@ship/ui-components` package. That's been
decomposed into **independent workspace packs** under
`packages/components/<pack>/`, each with its own `package.json`,
bundle, and `custom-elements.json`. Two projections:

| Catalog | Contents | Use |
|---|---|---|
| `workspace.json` | Every pack in the repo (public + private) | Tooling, agents, picker, design-system audits |
| `published.json` | Only `!private` packs | Library consumers, npm publishing |

The daemon serves per-pack bundles at `/_deck/packs/<pack>/<bundle>.js`
and the picker groups by pack origin instead of regex heuristics.

For an authoring agent: read `ship://components` to get the merged
manifest, then filter by `pack` if you only want primitives or only
charts.

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
