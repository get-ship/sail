// @ship/sail-viewer — the component-viewing surface every host composes.
//
//   <sui-component-viewer>  the shared viewer: a live stage holding the REAL
//                           component (slotted), variant chips that drive a real
//                           attribute on it, the generated prop table, usage.
//   <sui-render>            the mount engine: compile/runtime error branches,
//                           breakpoint-framed viewport.
//   ensureWasm/makeCompile  the browser compile seam.

import './sui-component-viewer/sui-component-viewer.compiled.js'
import './sui-render/sui-render.compiled.js'

export {
  ensureWasm,
  isWasmReady,
  makeCompile,
  type CompileFn,
  type WasmCompileOpts,
  type WasmCompileResult,
} from './wasm-bootstrap'
