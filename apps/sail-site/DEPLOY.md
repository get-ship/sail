# Deploy sail-site (static Cloudflare Pages)

The docs site is a CSR SPA. `sail build --bundle-framework --base assets`
emits one `index.html` (importmap + `#root`) plus `/assets`, `/_deck`,
public files, and `/sail_compiler_bg.wasm`. There is **no HTML per route**
and **no SSR**. The host must SPA-fallback extensionless paths to
`index.html`.

## One-command local prove

From `apps/sail-site`:

```bash
bun run build:static          # → dist/
bun run crawl:static          # headless full nav-graph crawl
```

`crawl:static` serves `dist/` with the same MIME map Pages needs
(`application/wasm`, `font/woff2`, …) and fails if any crawled route
is not 200 or if a same-origin asset 404s.

## One-command Pages deploy

```bash
bun run deploy:pages
```

Needs `CLOUDFLARE_API_TOKEN` + `CLOUDFLARE_ACCOUNT_ID` (or
`npx wrangler login`). First time, create the project:

```bash
npx wrangler pages project create sail-site
bun run deploy:pages
```

Config SSOT:

| Knob | Value |
| --- | --- |
| Wrangler | `apps/sail-site/wrangler.toml` (`pages_build_output_dir = "./dist"`) |
| Build | `bun run build:static` |
| Output | `apps/sail-site/dist` |
| SPA | `public/_redirects` → `/* /index.html 200` |
| MIME | `public/_headers` (wasm / woff2 / icons / `_deck/*.js`) |

Dashboard (git-connected): root directory `apps/sail-site`, same build
command + output dir. `build:static` wraps `sail build` — put `sail` on
PATH or set `SAIL_BIN`. Alternative: build locally and
`wrangler pages deploy dist` the artifact.

## What a dumb host must do

1. Serve `dist/` at `/`.
2. Extensionless paths → `index.html` (200), not 404.
3. `/sail_compiler_bg.wasm` → `application/wasm` (playground / tutorial).
4. `*.woff2` → `font/woff2`.
5. `/_deck/icons/*.svg` → `image/svg+xml` (named glyphs, no CDN).

## Playground wasm

`/sail_compiler_bg.wasm` must be the **same compile** as `_deck/compiler-wasm.js`.
`--bundle-framework` builds the JS from current source; the `.wasm` is copied
from `packages/sail/compiler-wasm/pkg`. If `pkg` is stale, compile panics
(`std::time` unsupported / ABI drift) and the badge stays on `compiling…`.

```bash
bun run wasm          # from apps/sail-site (or repo root)
bun run build:static
```
