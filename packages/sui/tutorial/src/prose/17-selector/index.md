---
title: "Selection state — keyed by Set membership"
description: "When one signal drives the highlighted row, two rows re-render. Not N."
group: "Tutorial"
order: 17
slug: "17-selector"
prev: "16-effect-cleanup"
next: "18-untrack-batch"
chapter: "reactivity"
---

A common pattern: a list of rows, one of which is "selected". The
naive shape subscribes every row to the `selectedId` signal, so when
the selection changes **every row re-evaluates**:

```sui
{#for r of rows() key={r.id}}
  <li class:selected={r.id === selectedId()}>{r.label}</li>  <!-- every row reads selectedId -->
{/for}
```

The "subscribes every row" claim is what costs paint time on a long
list. The fix is to read a structure that can answer "is `r.id` the
selected id?" with **per-key reactivity** — only the two affected
rows (the one losing selection, the one gaining it) re-evaluate.

Wrap the single selection in a `Set`, expose it as a `derived`, and
use the allowlisted `.has()` method call in the template:

```ts
import { signal, derived } from '@ship/sui-runtime'

const selectedId = signal(0)
const selectedSet = derived(() => new Set([selectedId()]))
```

```sui
{#for r of rows() key={r.id}}
  <li class:selected={selectedSet().has(r.id)}>{r.label}</li>
{/for}
```

`has()` is on the compiler's allowlist because it's read-only and
referentially transparent. The SUI runtime tracks each row's effect
against the *bucket* keyed by `r.id`, so when `selectedId` changes
from A → B only buckets A and B notify — every other row sits cold.

For a 1000-row list with a moving selection: naive = 1000
re-evaluations per selection change; `selectedSet().has(id)` = 2.
The other 998 rows never wake up.

## Try it

The example renders 200 rows and tracks a selection via arrow keys.
Try:

1. **Watch the DOM**: open devtools' Rendering → "Paint flashing".
   Move the selection with `j/k`. Only two rows flash per move.
2. **Compare with naive**: in the starter, the template uses
   `selectedId() === r.id` — every row reads `selectedId` and
   re-evaluates on change. The solved swaps to `selectedSet().has(r.id)`
   and only the two affected rows re-evaluate. Toggle between the
   two and watch the paint area change.
3. **Selection over a derived**: pass a derived signal (one that
   normalises `selectedId` modulo `rows().length`) into the `Set`
   constructor. The bucket reactivity still applies — only the
   affected rows re-fire.
