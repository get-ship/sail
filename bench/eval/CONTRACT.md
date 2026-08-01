# RUN CONTRACT v1 — sail eval harness seam

**Status:** normative for HARNESS-2. Fleet destination shape (FLEET-2 / b-12).
**Version string:** `run-contract/1`

This is the product. Core never learns what sail is; adapters never know where
runs live beyond the run manifest. Fleet can replace the interim local core
without touching evaluation code.

## Invocation

```
eval-adapter <phase>
```

- **stdin:** one JSON object — the **Run Manifest** (plus phase-specific fields).
- **stdout:** one JSON object — the **Phase Result**.
- **stderr:** human diagnostics only (not parsed).
- **exit codes:**
  - `0` — phase completed; inspect `ok` / `verdict` in the result body.
  - `2` — contract error (unsupported version, malformed stdin, unknown phase).
  - `1` — adapter internal crash (core maps to `adapter_crash` + named reason).
  - hang — core enforces per-phase timeout; kill → `adapter_timeout`.

Phases: `prepare` | `gate` | `render` | `collect`.

## Version negotiation

Manifest field `contract_version` MUST be `run-contract/1` (or a later version
the adapter declares support for). Core rejects unsupported versions with a
named error before spawning phases beyond version check.

Adapter may echo `contract_version` in every phase result.

## Run Manifest (core → adapter)

```jsonc
{
  "contract_version": "run-contract/1",
  "run_id": "calm-otter",                 // word-pair
  "batch_id": "20260801-120000-smoke",
  "task_id": "parallax-landing",
  "arm": "sail",                          // sail | react | vanilla | …
  "model_slug": "grok",
  "model": "grok-…",                      // resolved; may be null + reason
  "toolchain": "current",                 // stamp or alias
  "profile_hash": "sha256:…",
  "harness_ver": "sail-bench-0.1.0",
  "run_dir": "/abs/path/to/run",          // artifacts live HERE only
  "sandbox_dir": "/abs/path/to/run/sandbox", // agent workspace (often = run_dir or subdir)
  "standards_dir": "/abs/path/to/bench",  // tasks/profiles/payload roots
  "payload_dir": "/abs/…/payload/sail",   // arm base tree (no node_modules)
  "task": { /* Task def */ },
  "profile": { /* Profile def */ },
  "paths": {
    "events": "events.jsonl",             // rel to run_dir
    "manifest": "manifest.json",
    "gate": "gate.json",
    "render": "render.png",
    "source": "sandbox"                   // rel dir holding agent output
  },
  "env": { /* optional string map for tools */ },
  "limits": {
    "phase_timeout_ms": 600000
  },
  "phase": "prepare"                      // echoed by core for clarity
}
```

## Phase results (adapter → core)

Common envelope:

```jsonc
{
  "contract_version": "run-contract/1",
  "phase": "prepare",
  "ok": true,
  "verdict": "pass",                      // pass | fail | skip | error
  "wall_ms": 1234,
  "artifacts": [
    {
      "name": "events.jsonl",
      "rel_path": "events.jsonl",
      "blocking": false,
      "status": "present",                // present | missing | degraded
      "sha256": "…",                      // optional
      "bytes": 0,
      "reason": null                      // named when missing/degraded
    }
  ],
  "metrics": {},
  "flags": [],                            // e.g. "missing_tokens"
  "missing": {},                          // field -> named reason (NULL semantics)
  "detail": {},                           // phase-specific structured payload
  "error": null                           // { "code": "…", "message": "…" }
}
```

### prepare

Materialize sandbox: payload files, deps symlinks (never vendor node_modules),
AGENTS/skill context, hooks. Does **not** run the agent.

`detail`: `{ "sandbox_dir", "files_written": string[], "shared_deps": string[] }`

### gate

Arm completion verdict. Structured pass/fail + diagnostics — never grepped text.

- sail → `sail check` clean on authored sources
- react → `vite build` ok + `dist/index.html`
- vanilla → static/html structure gate

`detail`: arm-specific gate JSON; `ok` maps to store `gate_ok` 0/1/NULL.

### render

Build → serve → chromium screenshot → `run_dir/render.png`.
**Blocking:** no browser render ⇒ failed run. Write buffers directly to disk.
Timeout/degrade ⇒ `status: degraded` + named reason on the artifact; do not
silently skip.

`detail`: `{ "method": "browser"|"takumi"|"none", "url", "viewport" }`

### collect

Post-run metrics: tokens (agent trace), tool counts, event file pointers.
Blobs stay files; return pointers + numbers only.

`detail`: `{ "session_id", "usage", "events_rel", "source_rel" }`

## Failure algebra

| Condition | Core mapping |
|---|---|
| adapter exit 0, `ok: true` | phase success |
| adapter exit 0, `ok: false` | phase fail; run may continue or fail per phase policy |
| adapter exit 2 | contract error; abort run |
| adapter exit 1 / signal | `adapter_crash`; abort run |
| timeout | kill process group; `adapter_timeout` |
| batch cancel | kill in-flight adapter; `cancelled` |

Phase policy (v1):
- `prepare` fail → abort (no sandbox)
- `gate` fail → record, continue to render/collect (still want artifacts)
- `render` fail/missing blocking → run `status=failed` (HARNESS-ARCH: no render = failed)
- `collect` fail → record flags, do not invent metrics

## FS layout (core-owned, non-negotiable)

```
runs/<toolchain>/<task>/<model>/<word-pair>/
  manifest.json
  events.jsonl          # blob on disk
  gate.json
  render.png            # blocking browser ground truth
  sandbox/              # agent workspace / source
  …
```

- Word-pair run ids (`calm-otter`); never nanoids/timestamps as names.
- Batches = db rows + manifest; not directory levels.
- Artifacts stay inside the run dir; reports **reference**, never copy.
- Missing data = explicit NULL + named reason (store `missing_reasons_json` / flags).

## Conformance

Fleet owns the contract + suite (b-12). Sail adapters are acceptance fixtures.
Both interim local core and fleet core must pass the same suite.

## Interim local core

Until fleet b-12 lands, `sail-bench` ships an interim **local core** (TS/bun,
`bun:sqlite`) in this package that speaks the same contract by spawning
`eval-adapter`. Lift target: fleet Rust core (`bridge eval` reshape).
