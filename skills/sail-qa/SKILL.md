---
name: sail-qa
description: Grade a sail app like a user — drive the built output in a real browser, verify every requirement mechanically, capture evidence, and produce a structured rubric verdict. Use for eval grading, pre-ship QA of sail apps, and done-ness verification.
---

# Sail QA

You grade; you never fix. Drive the app's BUILT output in a real
browser like a user: click every control, fill every form, resize,
scroll, disable JS. Produce a structured verdict with evidence.
Screenshots are claims only when captured — a QA statement without an
artifact is worthless.

## Setup

1. Build and serve the app (`sail build` → serve the output dir, or
   the running dev server URL). Note the base URL.
2. Read the task/requirements if present (task.jsonc `requirements`,
   or the user's brief). Each requirement becomes a probe below.
3. Evidence goes in a named `evidence/` dir beside the work — never
   temp files, never copied elsewhere.

## Phase 1 — Static honesty

- Load with JavaScript DISABLED. Every content section must be present
  and readable. Content that only exists after JS/scroll is a critical
  finding (sail's no-JS contract).
- Screenshot the no-JS load: `evidence/static-nojs.png`.

## Phase 2 — Requirement probes (one per requirement)

For each requirement, write the click-script first, then execute it.
Examples of the shape:
- Dependent form: change country → assert the dependent field
  resets/relabels. Screenshot before + after.
- Adjustable shell: drag the divider → assert panels resize. Screenshot
  mid-drag end state.
- Parallax/landing: screenshot at top, mid-scroll, and bottom — reveals
  must fire; content must exist at every scroll position.
- Routing app: navigate each route directly by URL AND by in-app nav;
  assert content and 404 behavior.

Each probe verdict: PASS / PARTIAL / FAIL + evidence path + console
state. Any console error during a probe is a finding (severity by
impact).

## Phase 3 — Rubric (four lenses)

1. **Wrong moves** — raw escapes where vocabulary exists, imperative
   hacks around declarative gaps, broken static states, declaration
   gaps (claimed done, isn't).
2. **Missing reaches** — L1 elements/presets/tokens that exist and
   weren't used where they obviously apply.
3. **Missing primitives** — things the language should offer and
   doesn't (evidence the workaround; these become language-gap items).
4. **Controller issues** — state shaped wrong, effects fighting the
   declarative layer, JS doing the language's job.

Each finding: severity (critical = broken requirement; high = wrong
move; medium = missing reach; low = detail), the lens, evidence
pointer, and a classification: context / tool / compiler / language /
defaults.

## Phase 4 — Verdict

- Numbers: requirements passed/total, findings by severity, console
  error count.
- **Done** requires: all probes PASS, zero console errors, static
  honesty intact, no open critical/high findings.
- Otherwise: **Not done**, with the ordered fix list (highest severity
  first). Never "done with notes".

## Hard rules

- Never edit the app under test. Report; the author fixes.
- Evidence or it didn't happen: every claim cites a screenshot or a
  console capture.
- Grade what shipped, not what was intended.
