# L1 controller chip — spike report

**Branch:** `job/l1-controller-chip`  
**Spec:** Part B of `use-and-l1-controller-spec.md`  
**Date:** 2026-07-20

## What shipped

1. **Element mode allows `<controller>`**
   - `compile_element` strips the inline block (same SFC extract as `compile()`),
     captures the body on `ElementOutput.controller_src`, and emits static
     structure/skin/manifest as before.
   - Template seam test unchanged: no control flow / dynamic attrs/text.
   - When a controller is present, the structure root is stamped
     `data-sail-el="<tag>"` and the manifest carries
     `"controller": true` + `"controllerDiscovery": "data-sail-el"`.
   - CLI element mode writes `<name>.controller.ts` when present.

2. **Chip migration**
   - `crates/sail/compiler/stdlib/chip.sail`: root is bare `<span>` (no
     `use:chip`); owns a `<controller>` block documenting the brain.
   - Executable attach path: `packages/sail/runtime/src/element-controllers/chip.ts`
     registered in `ELEMENT_CONTROLLERS` and attached by
     `attachElementControllers(root)`.
   - `use:chip` remains as a **generic mix-in** for non-chip hosts
     (`actions/chip.ts` reuses the same controller impl). Not deprecated —
     still a valid mix-in; L1 chip no longer stamps it.
   - `action_manifest` summary updated to say mix-in / non-chip hosts.

3. **Runtime discovery attach**
   - `mount()` and CE `component` setup call `attachElementControllers` after
     DOM is live; dispose tears controllers down.
   - WeakSet guard → second attach on the same node is a no-op (hydration /
     remount safety).

4. **Docs**
   - `docs/ARCHITECTURE.md` element-mode section: controller **allowed**;
     `use:` is mix-in API, not L1 brain channel.

## PE / hydration answers (spec open questions)

### How is the controller module discovered for a lowered L1 tag in SSR HTML?

**Compile time:** element-mode structure includes `data-sail-el="chip"` on the
root when the `.sail` has a `<controller>`. That attribute survives splice into
any host template and SSR HTML.

**Runtime:** the stdlib ships a tag→controller registry
(`ELEMENT_CONTROLLERS` in `@ship/sail-runtime`). After route/component mount,
`attachElementControllers(root)` queries `[data-sail-el]` and runs the matching
controller. No per-page import of chip behavior is required for tags in the
registry — same "toolchain owns the brain" model as skins in `element-skins.css`.

**Not chosen for this spike:** import-map-by-tag lazy loading. Registry is
enough for stdlib L1; author elements can grow a register API later.

### Does first paint without JS still show correct *structure* for chip?

**Yes.** Structure HTML + framework skin CSS are static. Without JS the pill
renders with tone/variant/size/selected/removable CSS gates. Missing without JS:
role/tabindex/aria-pressed sync, keyboard activate/remove, remove-button event.
Same PE honesty as Actions (they also need JS).

### Hydration: double-binding risk?

**Mitigated.** Attach is idempotent per node via `WeakSet`. Mount and CE paths
both call attach; a second pass skips already-enhanced nodes. Dispose removes
listeners and clears the set for those nodes so a true remount can re-attach
cleanly.

**Residual:** if SSR HTML already had ARIA attributes hand-written and the
controller also sets them, values converge to controller truth (overwrite) —
not double listeners. Controllers should remain attach-once for listeners
(which they are).

## Tests

```
cargo test -p sail-compiler --test element_mode   # 11 ok (incl. controller + stdlib chip)
cargo test -p sail-compiler --test abi_boundary   # 6 ok
cargo test -p sail-compiler --lib parse::preprocess  # 55 ok (filter)
cd packages/sail/runtime && bunx vitest run test/chip-controller.test.ts  # 8 ok
```

Parity covered: role/aria-pressed/tabindex, disabled, selected mutation sync,
Enter/Space activate, Backspace/Delete + remove click → `sail-remove`,
idempotent attach.

## Non-goals (held)

- Full stdlib migration (alert, avatar, field, tabs, …) — separate jobs.
- Moving language tags to L2.
- Dogfooding chip skin off raw `<style>` onto sx (follow-on).
- Dropping `use:chip` from the action registry (kept as generic mix-in).

## Files touched (primary)

- `crates/sail/compiler/src/emit/element.rs`
- `crates/sail/compiler/src/lib.rs` (`compile_element`)
- `crates/sail/compiler/src/parse/mod.rs`
- `crates/sail/compiler/src/bin/sail-compiler.rs`
- `crates/sail/compiler/stdlib/chip.sail`
- `crates/sail/compiler/src/action_manifest.rs` (chip summary)
- `crates/sail/compiler/tests/element_mode.rs`
- `packages/sail/runtime/src/element-controllers/{chip.ts,index.ts}`
- `packages/sail/runtime/src/actions/chip.ts`
- `packages/sail/runtime/src/{mount,component,index}.ts`
- `packages/sail/runtime/test/chip-controller.test.ts`
- `packages/sail/l1-conformance/specs/chip.mjs` (comments)
- `docs/ARCHITECTURE.md`
