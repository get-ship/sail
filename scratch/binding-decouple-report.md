# binding-decouple report

**Branch:** `job/binding-decouple`  
**Commit:** `f0e2aaf74559125c1196e4dedfadfcd02ecb4911` (feature); report commit is branch tip after this file  
**Worktree:** `/home/madvib/dev/Ship/worktrees/binding-decouple`

## What changed

1. **Parser** — `prop:` / `attr:` refused with teaching text (plain `name={expr}`). Removed from `DIRECTIVE_PREFIXES` / `DIRECTIVES`; added to `RETIRED_DIRECTIVES`. Grammar regenerated (`grammar.js` + highlights).
2. **IR** — deleted `AttrInstr::Prop` and `AttrInstr::DynamicAttr`. Only `AttrInstr::Expr` for dynamic fields.
3. **Web lowering (backend, not author API)** — `plain_field_binder(tag, name)`:
   - custom element (`-` in tag) → `bindDynamic`
   - native + closed IDL table (`value`, `checked`, `indeterminate`, `selected`, `open`, `disabled`, …) → `bindProp`
   - else → `bindAttr`
4. **Migrate** — all in-tree `.sail` `prop:`/`attr:` → plain; docs/comments rewritten.
5. **Bench** — removed `data-selected`; single harness `use:benchDanger` classList path with foreign-driver comment.
6. **scope_manifest** — identity props: plain `name={expr}`, not `prop:`.
7. **Lint** — `sail-attr-bool-prop` / `sail-attr-presence` are stable no-op ids (prefixes gone).

## Gates

| Gate | Exit |
|------|------|
| `cargo test -p sail-compiler --lib` | 0 (439 passed) |
| `cargo test -p sail-compiler --test snapshot` | 0 (27 passed) |
| `cargo test -p sail-lint --lib` | 0 (393 passed) |
| `cargo test -p sail-compiler --test prop_binding` | 0 (7 passed) |
| `cargo test -p sail-compiler --test check_all` | 0 (10 passed) |
| `rg 'prop:|attr:' --glob '*.sail'` | empty (zero) |

## Remaining web seams (honest)

These are **not** solved by this cut; still target/backend surface:

- `bind:this` / `use:` still hand DOM nodes (target-specific handles).
- `reflect-empty="…"` still names a host attribute string.
- Free-form `data-*` / `aria-*` still authorable as plain fields on HTML tags (native surface) — prefer components/semantics; no Sail `data-selected` product pattern.
- SSR string vs live object for chart series — both plain fields; runtime distinguishes by value type (`bindDynamic`).
- Benchmark `.danger` class string — eternal foreign constraint (harness-only `classList` via `use:benchDanger`).
- Runtime helpers `bindProp` / `bindAttr` / `bindDynamic` remain as **backend** ABI; authors never spell the channel.
- Hydrate marker attribute names may still look like `attr:name` strings in SSR markup (mechanism, not author directive).
- Optional stretch not done: compile-time `PropKind::Identity` → direct `bindProp` for known child components (v1 uses `bindDynamic` on CEs).

## Non-goals / not pushed

Local commit only. No push. No land.
