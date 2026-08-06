# SAIL syntax freeze annexes (A–E)

Machine-readable inventories for **SAIL-SYNTAX-FREEZE** ratification.

| Annex | File | Source of truth |
|-------|------|-----------------|
| A L1 elements | `a-l1-elements.json` | `language-surface.gen.ts` ELEMENTS (compiler stdlib) |
| B Tokens | `b-tokens.json` | `packages/sail/tokens/src/token-registry.json` |
| C Public actions | `c-public-actions.json` | `action_manifest::PUBLIC_ACTION_NAMES` |
| D Runtime exports | `d-runtime-exports.json` | `packages/sail/runtime/author-surface.json` |
| E Channel vocabulary | `e-channel-vocabulary.json` | style IR `channel_for_setter` + conditions + MOTION |

Human tables: [`ANNEXES.md`](./ANNEXES.md). Index: [`index.json`](./index.json).

```bash
# regenerate
node scripts/gen-freeze-annexes.mjs

# drift check (CI / local)
node scripts/gen-freeze-annexes.mjs --check
```

Do not hand-edit generated JSON or `ANNEXES.md`.
