# keyed/sail

Owned [js-framework-benchmark](https://github.com/krausest/js-framework-benchmark)
`keyed/sail` sources. Legal Sail only — no `class=`, no `classList`, no
raw `div`/`span`. Driver chrome (Bootstrap classes, `tr.danger`, the
delete `a>span`) is stamped onto emitted JS in the benchmark checkout
(`~/dev/js-framework-benchmark/sail-driver/`). Never copy that patch here.

```sh
bun tests/perf/scripts/sync-keyed-sail.ts
# then, in the benchmark checkout:
#   cd frameworks/keyed/sail && bun install && bun run build-prod
#   bun ../../../../sail-driver/patch-emit.ts
```
