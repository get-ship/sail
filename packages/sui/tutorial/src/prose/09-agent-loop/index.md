---
title: "The agent authoring loop"
description: "How an AI agent writes a SUI view in one turn."
group: "Tutorial"
order: 9
slug: "09-agent-loop"
prev: "08-sui-primitives"
next: "10-lsp-cli"
chapter: "agents"
---

You'll mostly hand SUI authoring off to an agent. The agent's job is
to produce a `.sui` + `controller.ts` pair that compiles, stays in
scope, and behaves to spec — all in one turn. Ship gives the agent
six surfaces to make that one-turn target reachable.

```
            ┌──────────────────────────────────────┐
            │ 1. Load skill         docs/ship-sui/ │
            │                       SKILL.md       │
            │ 2. Scaffold           scaffold_view  │
            │ 3. Write              view_write_    │
            │                       source         │
            │ 4. Diagnose           sui_diagnostics│
            │ 5. Fix + reloop ↑                    │
            │ 6. Score              sui-eval       │
            └──────────────────────────────────────┘
```

The example to the right is a view with **four deliberate
diagnostics** baked in: an unknown event modifier, a missing prop
declaration, a stale signal reference, and a render-subset escape
(`.toFixed()` in the template). An agent would:

1. **Diagnose** — `cargo run -p sui-lint --example check -- page.sui`
   returns findings, each with `line`, `col`, `severity`, `message`.
2. **Fix** — one round-trip per error category. The agent edits the
   file (here: the editor pane) and re-diagnoses.
3. **Compile-check** — `sui-compiler build page.sui` runs the full
   lowering pipeline and surfaces anything the lint pass missed
   (lowering-only failures). A non-zero exit is a hard failure.
4. **Score** — once the view compiles, `@ship/sui-eval` runs it in
   jsdom against a behavioural spec and returns `{ score, sub, report }`.
   The agent decides whether to retry based on the score.

Try fixing the four diagnostics yourself — the editor surfaces them as
red squiggles, the same way an LSP client would render
`sui_diagnostics` output. **Show solution** reveals one valid fix.

## Why this matters

The point of the loop is **single-turn confidence**: the agent doesn't
guess and submit. It writes, diagnoses, fixes the surfaced issues,
re-diagnoses until clean, then scores. If the score is too low, it
rewrites — but the compile path is never the blocker.

See `docs/ship-sui/references/docs/agent-surfaces.md` for the full tool
inventory. See `docs/ship-sui/references/docs/agent-authoring.md` for the
do-now checklist (filename rules, the `<data>` directive shape,
the right places to put `<style>`).
