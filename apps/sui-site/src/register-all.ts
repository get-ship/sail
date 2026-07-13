// Nothing. The docs are built from L1 — the elements are in the language.
//
// The INTERACTIVE tutorial (the two-pane playground: <sui-code-editor> tabs,
// wasm compile via @ship/sui-viewer, sandboxed preview, console — recovered
// at src/tutorial-boot.ts + public/tut-host.html + the ba48c1f~1 pages) is
// HARD-BLOCKED here: importing @ship/sui-code-editor kills the build because
// the ship-sui plugin rejects the pack's styles under the floor. The moment
// the L2 pack-styling ruling lands, the tutorial route imports the pack and
// the playground comes back exactly as it was.
export {}
