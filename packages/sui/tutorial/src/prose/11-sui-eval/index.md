---
title: "Scoring with sui-eval"
description: "Regression-quality assertions on agent-authored views."
group: "Tutorial"
order: 11
slug: "11-sui-eval"
prev: "10-lsp-cli"
next: "12-slots"
chapter: "agents"
---

`@ship/sui-eval` runs an agent's output against a behavioural spec
and returns a structured quality score. The sub-scores tell you which
gate (compile / scope / behaviour) the agent missed, so the next
turn's prompt can target the failure.

```ts
import { runEval, type EvalTask, loadCatalog } from '@ship/sui-eval'

const task: EvalTask = {
  description: 'A counter with +1/-1 buttons and a reset.',
  catalog: await loadCatalog(),       // the component catalog
  spec: {
    interactions: [
      { action: 'click', selector: 'button[data-test="bump-up"]' },
      { assert: 'text',  selector: 'strong',                value: '1' },
      { action: 'click', selector: 'button[data-test="reset"]' },
      { assert: 'text',  selector: 'strong',                value: '0' },
    ],
  },
}

const result = await runEval(task, { model: 'sonnet' })
// → {
//     score: 0.92,
//     sub: { compile: 1, scope: 1, behaviour: 0.83 },
//     report: '…',
//     output: { template, controller },
//   }
```

## Sub-scores

| Sub-score | What it checks | Failure looks like |
|---|---|---|
| `compile` | Does the generated `.sui` + controller compile via the wasm pipeline? | Compile errors from `sui-compiler` or controller parse errors. |
| `scope`   | Does every signal, directive, and tag the template references actually resolve? | `unknown identifier`, `<sui-foo>` not in catalog, `bind:value` against a non-signal. |
| `behaviour` | Does the mounted view in jsdom pass every interaction + assert in the spec? | Click → no DOM change, assertion text mismatch, signal never updated. |

The overall `score` is a weighted average (compile is hardest-gated:
compile=0 caps the whole score regardless of behaviour).

## sniffEscapeHatches

`sniffEscapeHatches(output)` returns a separate array of policy
violations — things that compile but shouldn't ship:

```ts
import { sniffEscapeHatches } from '@ship/sui-eval'

const hatches = sniffEscapeHatches(result.output)
// → [
//     { kind: 'raw-script', line: 14, msg: '<script> in template' },
//     { kind: 'ts-ignore', line: 22, msg: '@ts-ignore in controller' },
//   ]
```

Detected hatches today:
- raw `<script>` tags in templates
- `@ts-ignore` / `@ts-expect-error` in controllers
- `eval()` calls anywhere
- imports outside the harness's importmap (e.g. node-only modules)

## Try it

The example to the right is a counter with `data-test` attributes
positioned so the spec above would actually pass. Try:

1. **Read the spec mentally** — three interactions (`click +`,
   `click reset`, `assert text`), the view satisfies all three.
2. **Break behaviour** — remove the `data-test="bump-up"` attribute
   on the `+1` button. The eval would fail the first interaction
   (selector miss), behaviour sub-score → 0, overall ≤ 0.66.
3. **Break scope** — rename `count` → `total` in the controller but
   not the template. Compile fails, scope=0, behaviour skipped,
   overall = 0.

The playground here just compiles and runs the view — it doesn't run
the eval. For actual scoring runs, hand it off to a node process with
`@ship/sui-eval` installed, or wire it into your CI via the agent.

## Why this matters

`sui-eval` is the **assertion** an agent makes about its own output
before reporting done. Diagnostics tell you "this compiles"; eval
tells you "this *works*". The combination is what makes one-turn
authoring viable.

See `docs/ship-sui/references/docs/agent-surfaces.md` § 3 for the full
API surface.

## Where next from here

You finished the agent-tooling tier (steps 9–11: the agent loop, the
LSP, and eval). Next up is the **composition** tier — slots, refs,
class/style directives, controller hooks, and forms (steps 12–21) —
then the **lifecycle** tier (steps 15–16 in the middle) and the
**views** tier with live data and morphs (steps 22–25).

Other places to go:
- **Build a real view** — `scaffold_view` to start, `agent-authoring.md`
  as the checklist.
- **Set up agent CI** — wrap `runEval` in your CI to gate PRs that
  touch `.sui` files.
- **Browse the primitives** — every `<sui-*>` has a dedicated page
  under `/components/`.