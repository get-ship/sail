# bake-off: wasm controllers (branch job/bakeoff-wasm-controllers)

Pinned to unified head 1991ce2b4. This branch carries the in-tree conformance
harness; per-language controller sources live on sibling branches:
- job/bakeoff-grain         (grain F1/F2 controllers + notes)
- job/bakeoff-rust          (rust/wasm-bindgen F1/F2)
- job/bakeoff-compile-away  (tree-shake measurement work)

Rules/invariants: CONTRACT.md at ~/dev/sail-bakeoff (mirrored here when the
first branch commit lands). Nothing here touches product code until C1-C6
pass — this branch is harness-only.
