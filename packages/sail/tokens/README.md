# @ship/tokens

Design tokens (DTCG JSON under `model/`) and generated CSS / registry.

```sh
cd packages/sail/tokens
bun run generate   # cargo run -p sail-compiler -- tokens
bun test
```

Edit sources under `model/`; do not hand-edit generated CSS.
