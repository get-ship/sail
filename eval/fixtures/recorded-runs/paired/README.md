# Recorded paired-task lanes (spike fixtures)

Canned worktree snapshots of two hypothetical eval lanes that built
`views/space-editor/` from `eval/tasks/paired/task-spec.md`. They stand in for
the expensive part of the eval — a dispatched agent working in a worktree — so
the vitest-evals spike (`evals/paired.eval.ts`) can exercise `describeEval` +
judges against a **recorded** run instead of re-dispatching an agent.

- `lane-composed/` — the HIGH-scoring shape: composes `<sui-*>` primitives,
  spends `var(--token)` values, pure template expressions.
- `lane-adhoc/` — the LOW-scoring shape: hand-rolled `display:flex/grid`,
  hex/px literals, raw font stack, and impure template expressions
  (`Date.now()`, in-place `.sort()`, a `const` declaration).

Same spirit as `eval/scripts/fixtures/primitive-adherence/`, but shaped as a
full lane worktree (view dir + controller + manifest + css) so the harness
reads it exactly the way `grade.ts` reads a real lane.
