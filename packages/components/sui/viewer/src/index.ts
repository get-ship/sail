// @ship/sui-viewer — the wasm compile seam plus the mount engine.
//
// The pack publishes what a host needs to render a live SUI specimen:
// `ensureWasm`/`makeCompile` (browser compile) and `<sui-render>` (the mount
// stage: compile/runtime error branches, breakpoint-framed viewport).
//
// The knob/matrix/source/viewer chrome is NOT published — the docs site owns
// its own detail view, which reads the generated prop registry and mounts real
// components with real interactions. Those four remain in-tree as the
// pack-internal composition behind `<sui-component-viewer>`.

import './sui-render/sui-render.compiled.js'

export {
  ensureWasm,
  isWasmReady,
  makeCompile,
  type CompileFn,
  type WasmCompileOpts,
  type WasmCompileResult,
} from './wasm-bootstrap'
