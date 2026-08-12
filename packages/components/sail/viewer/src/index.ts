// @ship/sail-components/viewers — the component-viewing surface every host composes.
//
//   <sail-component-viewer>  the shared viewer: a live stage holding the REAL
//                           component (slotted), variant chips that drive a real
//                           attribute on it, the generated prop table, usage.
//   <sail-render>            the mount engine: compile/runtime error branches,
//                           breakpoint-framed viewport.
//   ensureWasm/makeCompile  the browser compile seam.

import './sail-component-viewer/sail-component-viewer.compiled.js'
import './sail-render/sail-render.compiled.js'

export {
  ensureWasm,
  isWasmReady,
  makeCompile,
  type CompileFn,
  type WasmCompileOpts,
  type WasmCompileResult,
} from './wasm-bootstrap'
