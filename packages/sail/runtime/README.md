# @ship/sail-runtime

Browser runtime for compiled `.sail` modules: signals, custom elements, patch apply.

```sh
cd packages/sail/runtime
bun test
```

## API tiers

| Tier | What | Where |
|------|------|--------|
| **A — Template / ambient** | Tags, control flow, Style, `on:`/`use:`, Style motion (`.enter`/`.exit`/`.loop`/…), ambient controller vocab | Compiler tables → sail-site language reference + skill |
| **B — Author runtime** | Intentional `@ship/sail-runtime` imports humans may type | [`author-surface.json`](./author-surface.json) + root barrel subset |
| **C — Internal / host** | Emit-only (`/internal`, binders, …) and host subpaths (`server`, `framework`, `dev`, …) | Never on language reference pages |

Docs gen for tier B:

```sh
node apps/sail-site/scripts/gen-runtime-surface.mjs
# or: (cd apps/sail-site && bun run gen)
```

Public value exports humans should learn: `author-surface.json`. Internals and host entry points may change without notice.
