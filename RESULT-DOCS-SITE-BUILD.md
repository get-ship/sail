# RESULT — docs-site-build

**Branch:** `job/docs-site-build`  
**Base:** `job/channels-1` @ e6e5f2c5 (docs-freeze-align + for-as + freeze-annexes)  
**Status:** READY TO MERGE

## Goal

`sail dev --project apps/sail-site` stays up and serves docs pages.

## Root cause

HTML text runs in the Sail parser track `'` / `"` / `` ` `` as string delimiters so `{…}` arm expressions can contain tagged markup. **ASCII apostrophes in ordinary prose** (possessives / contractions such as `template's`, `doesn't`, `yesterday's`) therefore open a “string” and swallow following close tags until the next ASCII `'`, producing:

- `unterminated <p>`
- `unexpected </p> while inside <strong>` / `<div>` / `<li>`
- `arm body parse error: unexpected </sail-message> while inside <span>`

Parser was **not** weakened; docs markup was fixed.

## Fix

Replaced plain-text ASCII apostrophes with typographic U+2019 (`’`) across failing sail-site route pages. Left controller JS, attribute values, and `{…}` expression bodies untouched (where pairing is intentional).

### Files (22)

- `apps/sail-site/routes/ai/{authoring,eval,tooling}/page.sail`
- `apps/sail-site/routes/components/page.sail`, `components/[tag]/page.sail`
- `apps/sail-site/routes/deck/{page,data,dynamic-routes,hmr,layouts,rendering}/page.sail`
- `apps/sail-site/routes/sail/{why,overview}/page.sail`
- `apps/sail-site/routes/sail/language/{actions,animation,capabilities,data,errors,layout,sail-file,styling}/page.sail`
- `apps/sail-site/routes/sail/guide/making-a-theme/page.sail`

## Proof

```text
cargo build -p deck-cli --bin sail   # ok
./target/debug/sail dev --project apps/sail-site --addr 127.0.0.1:7890
# → sail dev: serving … on http://127.0.0.1:7890  (stays up)

curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:7890/                        → 200
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:7890/sail/language/styling  → 200
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:7890/sail/reference/regions → 200
```

`sail check apps/sail-site/routes`: product pages green. Residual (non-blocking under `sail dev` host registration):

- `deck/examples/greet/[name]/page.sail` — `route` ambient only when registered as a deck route by file location; `sail check` path-scan does not inject it. Live under dev at `/deck/examples/greet/:name`.

## Ops notes (local only, not committed)

- Fresh worktree needed `bun install` and a `packages/sail/compiler-wasm/pkg` (gitignored; copied or `bun run wasm`) so automatic `build-assets` could resolve `@ship/sail-compiler-wasm` for LSP bundles.
- Framework bake under `.deck/` is gitignored; regenerated on first `sail dev`.

## No push

Commit only. Do not push.
