import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { initSync } from '@ship/sail-compiler-wasm'

const wasmPath = fileURLToPath(new URL('../../../sail/compiler-wasm/pkg/sail_compiler_bg.wasm', import.meta.url))
initSync({ module: readFileSync(wasmPath) })
