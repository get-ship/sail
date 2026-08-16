/**
 * Highlight a `.sail` buffer with tree-sitter-sail + highlights.scm.
 * Same query the Rust crate runs (`tree_sitter_sail::highlight`).
 */

import { Parser, Language, Query } from 'web-tree-sitter'
import { isSailScope, type SailScope } from './scopes'
import { toTokHtml } from './tok'

export type { SailScope }
export { SCOPE_TO_CM_CLASS, SCOPE_TO_LSP, SCOPE_TO_TOK, SCOPES, isSailScope } from './scopes'
export { toTokHtml } from './tok'

export interface HighlightSpan {
  start: number
  end: number
  scope: SailScope
}

export interface HighlightAssets {
  /** URL or filesystem path to `tree-sitter-sail.wasm`. */
  wasm: string | Uint8Array
  /** Body of `queries/highlights.scm`. */
  highlightsScm: string
  /**
   * web-tree-sitter runtime wasm. Path (`locateFile`) or bytes (`wasmBinary`).
   * Optional — omitted, the runtime finds `web-tree-sitter.wasm` itself.
   */
  runtimeWasm?: string | Uint8Array
}

let ready: Promise<void> | null = null
let language: Language | null = null
let query: Query | null = null

function initParser(runtimeWasm?: string | Uint8Array): Promise<void> {
  if (runtimeWasm instanceof Uint8Array) {
    return Parser.init({ wasmBinary: runtimeWasm })
  }
  if (typeof runtimeWasm === 'string') {
    return Parser.init({ locateFile: () => fileUrlToPath(runtimeWasm) })
  }
  return Parser.init()
}

/** `Language.load` in Node uses `fs.readFile`; it wants a path, not `file://`. */
function fileUrlToPath(href: string): string {
  if (!href.startsWith('file:')) return href
  try {
    return decodeURIComponent(new URL(href).pathname)
  } catch {
    return href
  }
}

function wasmInput(wasm: string | Uint8Array): string | Uint8Array {
  return typeof wasm === 'string' ? fileUrlToPath(wasm) : wasm
}

export async function initHighlight(assets: HighlightAssets): Promise<void> {
  if (!ready) {
    ready = (async () => {
      await initParser(assets.runtimeWasm)
      language = await Language.load(wasmInput(assets.wasm))
      // web-tree-sitter 0.26 dropped Language.query — Query is constructed.
      query = new Query(language, assets.highlightsScm)
    })()
    try {
      await ready
    } catch (err) {
      ready = null
      language = null
      query = null
      throw err
    }
    return
  }
  await ready
}

/** Default wasm + query next to this package. Safe to call more than once. */
export async function ensureHighlight(): Promise<void> {
  if (ready) {
    await ready
    return
  }
  const wasmHref = new URL('../tree-sitter-sail.wasm', import.meta.url).href
  const scmHref = new URL('../../grammar/queries/highlights.scm', import.meta.url).href
  const highlightsScm = await readAssetText(scmHref)
  await initHighlight({ wasm: wasmHref, highlightsScm })
}

async function readAssetText(href: string): Promise<string> {
  const res = await fetch(href)
  if (!res.ok) {
    throw new Error(`sail-highlight: failed to load query (${res.status})`)
  }
  return res.text()
}

/** `tok-*` HTML for `<code-block>`. Requires init / ensure first. */
export function sailTokHtml(src: string): string {
  return toTokHtml(src, highlight(src))
}

export function highlight(src: string): HighlightSpan[] {
  if (!language || !query) {
    throw new Error('initHighlight() must resolve before highlight()')
  }
  if (!src) return []
  const parser = new Parser()
  parser.setLanguage(language)
  const tree = parser.parse(src)
  if (!tree) return []
  const captures = query.captures(tree.rootNode)
  const out: HighlightSpan[] = []
  for (const cap of captures) {
    const name = cap.name
    if (name.startsWith('_') || !isSailScope(name)) continue
    out.push({
      start: cap.node.startIndex,
      end: cap.node.endIndex,
      scope: name,
    })
  }
  out.sort((a, b) => a.start - b.start || a.end - b.end)
  parser.delete()
  tree.delete()
  return out
}
