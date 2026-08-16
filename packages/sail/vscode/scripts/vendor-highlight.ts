/**
 * Copy tree-sitter-sail.wasm + highlights.scm (+ web-tree-sitter.wasm)
 * into vendor/ so a VSIX / Install from Location works offline.
 *
 * Invoked from `bun run compile`. Drift gate: test/package.test.ts.
 */
import { copyFileSync, existsSync, mkdirSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
export const vscodeRoot = resolve(here, '..')
export const vendorDir = resolve(vscodeRoot, 'vendor')
export const grammarHighlightsScm = resolve(
  vscodeRoot,
  '../grammar/queries/highlights.scm',
)
export const languageWasmSrc = resolve(
  vscodeRoot,
  '../highlight/tree-sitter-sail.wasm',
)
export const vendorHighlightsScm = resolve(vendorDir, 'highlights.scm')
export const vendorLanguageWasm = resolve(vendorDir, 'tree-sitter-sail.wasm')
export const vendorRuntimeWasm = resolve(vendorDir, 'web-tree-sitter.wasm')

function findRuntimeWasm(): string {
  const candidates = [
    resolve(vscodeRoot, 'node_modules/web-tree-sitter/web-tree-sitter.wasm'),
    resolve(vscodeRoot, '../highlight/node_modules/web-tree-sitter/web-tree-sitter.wasm'),
    resolve(vscodeRoot, '../../../node_modules/web-tree-sitter/web-tree-sitter.wasm'),
  ]
  for (const c of candidates) {
    if (existsSync(c)) return c
  }
  throw new Error(
    'web-tree-sitter.wasm not found — bun install @ship/sail-highlight / web-tree-sitter',
  )
}

export function vendorHighlightAssets(): void {
  if (!existsSync(grammarHighlightsScm)) {
    throw new Error(`missing ${grammarHighlightsScm}`)
  }
  if (!existsSync(languageWasmSrc)) {
    throw new Error(
      `missing ${languageWasmSrc} — bun run build-wasm in packages/sail/highlight`,
    )
  }
  mkdirSync(vendorDir, { recursive: true })
  copyFileSync(grammarHighlightsScm, vendorHighlightsScm)
  copyFileSync(languageWasmSrc, vendorLanguageWasm)
  copyFileSync(findRuntimeWasm(), vendorRuntimeWasm)
}

if (import.meta.main) {
  vendorHighlightAssets()
  console.log(`vendored highlight assets → ${vendorDir}`)
}
