// Web-worker entry. Runs the structural analyser off the main thread
// so heavy parses don't stall the editor. Stateless for the cheap
// endpoints (diagnose / complete / hover); the lazy oxc-parser
// bundle is loaded on first `preload` or first hover that needs
// identifier resolution.

import { diagnose, type SailDiagnostic } from './diagnostics'
import { completionsAt, allSources, type SailCompletionSource } from './completions'
import { hoverAt, type SailHoverInfo } from './hover'

interface DiagnoseRequest   { id: number; kind: 'diagnose';   text: string }
interface CompleteAtRequest { id: number; kind: 'completeAt'; text: string; offset: number }
interface AllSourcesRequest { id: number; kind: 'allSources' }
interface HoverAtRequest    { id: number; kind: 'hoverAt';    text: string; offset: number }
interface PreloadRequest    { id: number; kind: 'preload' }
type Request =
  | DiagnoseRequest | CompleteAtRequest | AllSourcesRequest
  | HoverAtRequest | PreloadRequest

interface OkResponse<T> { id: number; ok: true; data: T }
interface ErrResponse    { id: number; ok: false; error: string }
type Response<T> = OkResponse<T> | ErrResponse

function reply<T>(id: number, data: T): void {
  postMessage({ id, ok: true, data } satisfies Response<T>)
}
function fail(id: number, err: unknown): void {
  postMessage({ id, ok: false, error: err instanceof Error ? err.message : String(err) } satisfies Response<unknown>)
}

// Lazy oxc-parser handle. v2a (dict-only hover) ships without
// touching oxc-parser at all — the preload endpoint is a stub that
// resolves true immediately. v2b will switch to a real dynamic
// import once identifier-aware hover is wired into the editor UI;
// the bundle hit is ~500 KB gzipped over the wire.
//
// Why not load it yet: the docs build (Vite/Rollup) can't resolve
// oxc-parser's optional native binding (@oxc-parser/binding-wasm32-wasi)
// and refuses to bundle a worker that imports it. The static-dict
// hover already covers ~80% of useful queries; the deferred bundle
// only pays off once we wire identifier resolution through it.
let oxcLoad: Promise<unknown> | null = null
function preloadOxc(): Promise<unknown> {
  if (!oxcLoad) {
    // Stub. Resolves on the next microtask so the client's `preload()`
    // pattern works end-to-end without actually fetching anything.
    oxcLoad = Promise.resolve(null)
  }
  return oxcLoad
}

addEventListener('message', (event: MessageEvent<Request>) => {
  const req = event.data
  try {
    switch (req.kind) {
      case 'diagnose': {
        // WASM-backed now → async (lazy one-time module init in diagnose()).
        diagnose(req.text).then(
          (diags) => reply<SailDiagnostic[]>(req.id, diags),
          (e) => fail(req.id, e),
        )
        break
      }
      case 'completeAt': {
        const src = completionsAt(req.text, req.offset)
        reply<SailCompletionSource | null>(req.id, src)
        break
      }
      case 'allSources': {
        reply<SailCompletionSource[]>(req.id, allSources())
        break
      }
      case 'hoverAt': {
        // v2a: static dictionary (directives + modifiers + blocks).
        // v2b adds identifier resolution via oxc once preload lands;
        // for now we return whatever the static dict knows.
        const info = hoverAt(req.text, req.offset)
        reply<SailHoverInfo | null>(req.id, info)
        break
      }
      case 'preload': {
        // Fire-and-forget warm-up. Resolves once oxc is in memory;
        // any failure is logged and the dict-only path keeps working.
        preloadOxc().then(
          () => reply<boolean>(req.id, true),
          () => reply<boolean>(req.id, false),
        )
        break
      }
    }
  } catch (e) {
    fail(req.id, e)
  }
})
