# RESULT — L1 surface scaffold (B4 display/feedback peel)

**Date:** 2026-08-11  
**Branch:** `job/release-integrate`  
**SPEC:** SPEC-L1-SURFACE-SCAFFOLD (B4 peel)  
**Base tip:** `a9ad1e92` (after B3)

## Landed

### B4 — display / feedback peel batch
First-class packages under contract + peeled from `_legacy`:

| Tag | Axes (prop-registry only) | Faces |
|-----|---------------------------|-------|
| **a** | variant default\|solid\|soft\|outline\|ghost · tone optional · disabled | *(none — freeform paints component.button.*; no component.a.\*)* |
| **kbd** | *(empty — stdlib props:{})* | component.kbd.bg / border / fg |
| **prose** | *(empty — scope host)* | component.prose.fg |
| **code-block** | code · lang (default sail) | component.code-block.bg / fg |
| **empty-state** | variant default\|inline\|placeholder\|connecting · size md\|sm\|lg · tone optional | component.empty-state.bg / border / fg / title |
| **error-state** | variant default\|inline · size md\|sm\|lg · code | component.error-state.bg / border / fg |
| **error-boundary** | message · onErrorReset (event) · onErrorCaught (event) | component.error-boundary.bg / border / fg / icon-fg / message-fg / retry-bg / retry-fg |

Each host has: `contract.ts`, `doc-<slug>.sail` (specimen left / knobs right ~320px), meta `host: doc-<slug>` + `stageOwnsKnobs: true`, shell `DOC_CE_SLUGS` + mount, visual expanded for `visual.cross`.

### Visual verification
| Tag | `bun run visual` |
|-----|------------------|
| a | ✓ green (lowered freeform stamps; dual-arm rest fill soft-probe) |
| kbd | ✓ green |
| prose | ✓ green (type scale h1>h2>h3≥h4≥p) |
| code-block | ✓ green |
| empty-state | ✓ green (s-empty-displaynone gates + full matrix) |
| error-state | ✓ green (anatomy toggle + code/details gates) |
| error-boundary | ✓ green (render probe + catchRender behavior) |

Checker: `bun apps/sail-site/scripts/check-l1-surface-contract.mjs` → **ok (27 contracts)**.

### Checker notes
- Zero-axis hosts (`kbd`, `prose`) with stdlib `props:{}` are omitted from prop-registry gen — checker allows missing registry when contract axes are empty.
- KNOB_LABELS extended: `lang`, `code`, `message`, `connecting`.

### Browser smoke (`http://127.0.0.1:7892`)
- Run when site is up; expect each B4 detail to mount `doc-*` with knobs/specimen.

## Matrix
All B4 hosts → `scaffold_ready` / operator pending. Pairing queue: [PAIRING-QUEUE.md](./PAIRING-QUEUE.md).

## Residuals / blockers for operator
1. Multi-theme taste approval (operator gate) — especially kbd keycap aesthetics, prose type brand
2. Playwright knobs→paint structural tests (later)
3. No PE invent: component.a.* leaves, kbd/prose axes
4. Freeform dual-arm stamp: solid rest without tone can lose fill to unset --tone arm (PE/runtime residual; probe documents)
5. B5+ peels still legacy residual

## No invent
- Axes from prop-registry only (kbd/prose empty)
- Faces only existing token-registry leaves
- a faces empty (button-surface paint, not component.a.*)

## No push
Local commits only on `job/release-integrate`.
