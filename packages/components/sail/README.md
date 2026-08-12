# `@ship/sail-components`

Single tree-shaken SAIL components package (primitives, charts, media, chat,
canvas, code, viewers, mermaid).

## Consumption model (dual audience)

| Audience | Import | What resolves |
|----------|--------|----------------|
| **Sail apps** | Domain barrels | Source `.ts` (registers tags via side-effect imports) |
| **Non-sail / single-tag** | Domain-prefixed tag | Precompiled `.compiled.js` (CE registration) |

Adjacent `.sail` sources ship for debug and for the deck/sail compiler, which
recompiles from source when bundling. Do not mix random entry styles — one path
per surface.

### Domain barrels (sail-source)

```ts
import '@ship/sail-components/primitives'
import '@ship/sail-components/charts'
import '@ship/sail-components/code'
import '@ship/sail-components/viewers'
import '@ship/sail-components/media'
import '@ship/sail-components/chat'
import '@ship/sail-components/canvas'
import '@ship/sail-components/mermaid'
```

### Per-component (tree-shake / non-sail)

```ts
import '@ship/sail-components/charts/sail-bar-chart'
import '@ship/sail-components/primitives/sail-calendar'
```

### Utilities (one canonical path each)

```ts
import { … } from '@ship/sail-components/media/engine'
import { ensureWasm, makeCompile } from '@ship/sail-components/viewers/wasm-bootstrap'
import { … } from '@ship/sail-components/viewers/same-doc-mount'
```

There are **no** short aliases (`./engine`, `./sail-calendar`, bare
`./wasm-bootstrap`). Prefer domain-prefixed paths only.

## Layout

Domain folders under this package are **source trees**, not separate npm
packages. The only publishable package name is `@ship/sail-components`.
