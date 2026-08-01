---
name: sail-output
description: Complete-output enforcement for sail authoring. Bans placeholder patterns, truncation, and declaration gaps. Apply to any sail task requiring exhaustive implementation.
---

# Full-Output Enforcement

A partial output is a broken output. No optimizing for brevity.

## Banned patterns (hard failures)

In code: `// ...`, `// rest of code`, `// implement here`, `// TODO`,
`// similar to above`, `// continue pattern`, bare `...` standing in
for omitted markup or controller logic.

In prose: "let me know if you want me to continue", "for brevity",
"the rest follows the same pattern", "and so on" replacing content.

Structural shortcuts: a skeleton when the request was a full page;
first and last section with the middle skipped; one example standing
in for repeated structure; describing what the sail should do instead
of writing it.

## Process

1. Scope — count the distinct deliverables (routes, sections, states,
   controller behaviors). Lock the number.
2. Build — every deliverable complete: every section authored, every
   state wired (loading/empty/error/active), every requirement in the
   task present and functional.
3. Cross-check — re-read the request; deliverable count must match.
4. Verify — `sail check` clean is the floor, not the finish. The task
   is done when the gate passes, not when the file exists.

## Long outputs

Never compress to fit. Write at full quality to a clean breakpoint
(end of an element, end of a section), then:

```
[PAUSED — X of Y complete. Send "continue" to resume from: <next section>]
```

On continue: resume exactly there. No recap, no repetition.

## Final check

- No banned patterns anywhere.
- Every requested item present and finished.
- Every `.sail` file is real, complete markup — not a description of
  markup.
