---
slug: sail-enforcement-architecture
status: drafting
intent: clear
review_required: false
pending-action: write .omo/plans/sail-enforcement-architecture.md
approach: Bottom-up pass over one six-layer enforcement stack (language → IR → capability → severity/policy → harness → culture), five independent components (A doctrine flips, B unified IR substrate, C author capability model incl. sanitization zone, D repo-agent harness gates, E testing-culture remediation). Four structural rulings pending from operator (IR depth/model, zone semantics + ownership, RI merge sequencing, goldens demote-vs-delete) before the plan is written.
---

# Draft: sail-enforcement-architecture

## Components (topology ledger)
<!-- Lock the SHAPE before depth. One row per top-level component that can succeed or fail independently. -->
<!-- id | outcome (one line) | status: active|deferred | evidence path -->

| id | outcome | status | evidence |
|---|---|---|---|
| A | Named doctrine (L0/E rows) promoted to default-preset Error via floor/capability machinery; sail-site corpus complies | active | crates/sail/compiler/src/check/policy.rs (FLOOR_RULES), check/lint_overrides.rs; GAP-TODO L0-1..L0-5, E1-E8 |
| B | Unified kind-tagged IR spine: stable node ids; enforcement passes (lint/tscheck/layout/capability) walk one graph; diagnostics span-keyed | active | compiler/src/ir/{mod,template,expr}.rs; spinnaker/src/ir.rs; bridge check/layout.rs |
| C | Declarative per-kind capability model (location/imports/globals/syntax/size/return-shapes) as SSOT table generating checks + docs/LSP/skill; sanitization-zone chokepoint for ambient authority | active | lint/src/project_map.rs (@/ table), rules/mod.rs dispatch families, runtime author-surface.json; unenforced cells listed in findings |
| D | Repo-agent harness gates: E5 generated-edit refusal, allowlist audit trail, F3 numbered acceptance, F4 done-evidence gate, dead-env scrub | active | hooks/, tests/hygiene/* (17 files), .github/workflows/hygiene.yml, mcp-tools.json |
| E | Testing culture: env-blesses stop gating, baselines collapsed, tombstone/tautology/site-fixture debt burned down per test-doctrine ratchet; doctrine documented in-repo | active | tests/hygiene/test-doctrine.test.ts + baseline json; liveness audit (all 12 offenders STILL LIVE) |

## Open assumptions (announced defaults)
<!-- Record any default you adopt instead of asking, so the user can veto it at the gate. -->
<!-- assumption | adopted default | rationale | reversible? -->

| assumption | adopted default | rationale | reversible? |
|---|---|---|---|
| Severity-flip timing | Doctrine rows already closed as law (L0-4) flip to default Error within this pass, not deferred | Operator law: warn/allowlist/strict-only is not the fix; corpus compliance ships in same pass | yes |
| Test strategy | Trap-first TDD: failing `.bad` fixture + paired happy cell before each rule/enforcement change; behavior tests query role/name/IR (H12) | Doctrine AGENT-AUTHORING tests section | yes |
| Plan shape | ONE plan, five components, phased waves bottom-up (B→C→A/D/E where independent) | ulw-plan invariant; user said "bottom up" | yes |
| npm posture | PENDING ruling — operator floating "sanitization zone" concept instead of picking | operator declined menu; wants discussion | n/a |
| Goldens | PENDING ruling — operator: "generally yes we want golden tests... cart before the horse"; premature while site doesn't build | operator words | n/a |

## Findings (cited - path:lines)

Compiler/severity:
- 4-stage pipeline: natural → overrides → FLOOR promotion → strict/floor global; `compiler/src/check/lint_overrides.rs:106-209`, order at `check/mod.rs:388-398`
- FLOOR_RULES (5, override-proof): `check/policy.rs:78-84`; capability dial has exactly ONE rule (`sail-no-raw-container`) `policy.rs:90`; default no-config ⇒ Off
- Policy registry SPEC-POLICY-1: `compiler/src/policy.rs:114-206` (layout.raw_containers, style_slots, lint.severity.<id>)
- tscheck tsgo type gate OnSave-only, loud-fail missing tsgo: `check/tscheck.rs:105-109,898-1135`
- Layout: unintended-overflow + clipped-text default Error; sail-layout Warn; sub-min-tap experimental `registry.rs:159-168,589-598,1163-1172`
- diag ratchet zero anonymous sites: `diag_ratchet.rs:46`
- SAIL_COMPILER_NO_LIMITS gone from compiler; dead ref only apps/studio/playwright.config.ts:26

IR:
- Compiler IR Program/Instr/Expr: `compiler/src/ir/mod.rs:17`, `ir/template.rs:11-32`, `ir/expr.rs:18`; consumers lower/, emit/, check/ visit_program, LSP
- Spinnaker Style IR separate: `spinnaker/src/ir.rs:7,57,78`; bridge ad hoc `compiler/src/check/layout.rs`
- H12 law wants tests to "query role, name, or IR" ⇒ IR is candidate semi-public surface

Authoring surface:
- FileKind homes + closed 6-home @/ alias table: `lint/src/project_map.rs:184-211,369-380`
- Dispatch families: CONTROLLER_RULES(17) rules/mod.rs:86-125; ACTION_RULES(6, all Error) :240-247; SUI_TEMPLATE_RULES(37) :129-182; APP_SOURCE_RULES(7) :210-232; PACK_SOURCE_RULES(3)
- Grammar SSOT chain manifests→grammar_vocabulary.rs→grammar.js; generated skill `sail-compiler skill` → .claude/skills/sail-authoring/SKILL.md (bin/sail-compiler.rs:1113-1389)
- UNENFORCED cells: component/composition/store/theme placement, route naming, action+composition size limits, action return shape, controller return shape (tsc-only), @/ usage in compositions

Harness:
- Hooks pre-commit (.artifacts ban, rustfmt, sail check; merge: clippy+typecheck) / pre-push (hygiene); 17 hygiene gates incl. emitters.json registry, temp-registry expiry, sheet-var contracts, stdlib-platform-css
- E5 partial (drift --check but no edit refusal); F3 unenforced; F4 partial (sail_task_add_artifact exists, no done-gate); skills folklore
- MCP 50+ tools (mcp-tools.json): sail_check, sail_edit, sail_task_*

Testing culture liveness audit (ALL 12 STILL LIVE):
- SAIL_GOLDEN_UPDATE golden.rs:65; UPDATE_SNAPSHOTS snapshot.rs:118; SAIL_RENDER_UPDATE_GOLDEN render/tests/golden.rs:57 (+10 PNGs)
- ANONYMOUS_BASELINE=0 diag_ratchet.rs:46; BASELINE=0 channel_discipline.rs:144
- tokens.test.ts pins :145-167, tautology :1108, tombstones :1303-1305, decls.size>100 :903
- temp-registry tombstones ×6; calendar it.skip date-math.test.ts:136; concern_drift/concern_language scrub+tombstone+RetiredSetter (spinnaker ir.rs:263)
- style_privilege.rs + type_gate_tsc_parity.rs use live apps/sail-site fixtures; parity.yml stale crates/sail/layout/** paths :15,26
- 96/129 compiler test files emit-shape; test-doctrine ratchet grandfathered ~100 violations; TEST_DOCTRINE_REGENERATE=1 bless hatch

SES/Jessie research:
- Import-graph-as-capability is fully static; Jessie verified by pure ESLint config; mechanisms to steal: per-kind profiles, imports-as-grants, deny-by-default globals via identifier resolution + lib/types starvation, no code-from-string, data-only exports, teaching errors quoting policy, manifest-declared elevated capabilities
- Prior art: Thales total-TS w/ diagnostic codes; Deno perm taxonomy; Figma API-surface-as-capability; workerd module kinds
- clients.ts now stub ("nested Clients bag GONE") — operator proposes sanitization-zone concept "similar to clients"

Context:
- MERGE-PREP.md: mid-flight framework-doover ← job/release-integrate, 15 conflicts, token renames, 14 known lib reds; site reportedly not building (operator)

## Decisions (with rationale)

1. Intent CLEAR, review_required false — outcome known; forks are owner-decisions being discussed in conversation (operator rejected option-menu format).
2. Topology locked: 5 components A-E above.
3. Stack framing adopted: language→IR→capability→severity→harness→culture; severity flips ride existing floor/dial machinery.
4. Capability model direction: SES endowment/Jessie-whitelist made static; extend existing import-convention machinery rather than new mechanism class.
5. Scaffold script delegated 3× and failed on infra (network_error, timeout ×2); draft hand-written to script's exact buildDraft() shape (isUlwArtifact-safe for later no-op rerun).

## Scope IN

- Sail repo head only (crates/sail, packages, apps corpora compliance, hooks, hygiene, CI, MCP, generated surfaces).
- Both audiences explicitly separated: repo dev-agents (D, E) vs end-user authors (A-C).

## Scope OUT (Must NOT have)

- No execution of the RI merge itself (precondition handled separately; plan assumes clean base or explicit wave-zero handling per operator ruling).
- No wasm sandbox implementation (experiments continue out-of-band; design must not foreclose it).
- No new product features beyond enforcement/capability/culture scope (no token vocabulary changes, no gauntlet G-series buildout except where goldens ruling touches reports).
- No deletion of mission scratch or doctrine docs; GAP-TODO remains external backlog home.

## Open questions

1. IR: internal enforcement spine vs semi-public contract (H12 queries/studio)? Style absorption into template spine in wave 1?
2. Sanitization zone: directory/module convention vs runtime construct? Who writes inside it? Naming/convention?
3. Sequencing: start after RI merge lands vs absorb merge as wave zero?
4. Goldens: demote-to-opt-in-reports vs delete outright?

## Approval gate
status: drafting
<!-- When exploration is exhausted and unknowns are answered, set status: awaiting-approval. -->
<!-- That durable record is the loop guard: on a later turn read it and resume at the gate instead of re-running exploration. -->
