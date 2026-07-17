// Direct-emit hydrate helpers for `{#while-loading}` / `{#while-error}`.
//
// The compiler-emitted view's `hydrate(root, ctx)` calls these for each
// SSR'd `<sail-while-loading key="bN">` / `<sail-while-error key="bN">`
// element. On signal transitions, the render callback (also emitted by
// the compiler, captured in the setup closure) builds fresh CSR DOM
// with inline reactive bindings — no manifest walk, no `wireSubtree`.
//
// SSR'd active content stays put on first effect-run (`firstRun`) so
// initial paint matches server output. Leaf bindings inside the SSR'd
// body are not wired — by definition the signal must transition before
// they would change, and any transition replaces the entire body.

import { effect } from '@ship/sail-runtime'

type WhileResult = { nodes: Node[]; cleanup: () => void }
type WhileRender = () => WhileResult
type WhileErrorRender = (err: unknown) => WhileResult

export function hydrateWhileLoading(
  el: Element,
  key: string,
  signalGetter: () => unknown,
  render: WhileRender,
): () => void {
  let current: Element = el
  let branchDispose: (() => void) | null = null
  let firstRun = true

  const condDispose = effect(() => {
    let isLoading = true
    try {
      const v = signalGetter() as { kind?: string } | undefined
      isLoading = !v || v.kind === 'loading'
    } catch {
      /* signal not ready; default to loading */
    }
    if (firstRun) { firstRun = false; return }
    applyTransition(isLoading)
  })

  function applyTransition(show: boolean): void {
    if (branchDispose) { try { branchDispose() } catch {} ; branchDispose = null }
    const parent = current.parentNode
    if (!parent) return
    const owner = current.ownerDocument ?? document
    const next = owner.createElement('sail-while-loading') as HTMLElement
    next.setAttribute('key', key)
    if (show) {
      next.style.display = 'contents'
      const result = render()
      for (const node of result.nodes) next.appendChild(node)
      branchDispose = () => { try { result.cleanup() } catch {} }
    } else {
      next.setAttribute('hidden', '')
    }
    parent.replaceChild(next, current)
    current = next
  }

  return () => {
    try { condDispose() } catch {}
    if (branchDispose) { try { branchDispose() } catch {} ; branchDispose = null }
  }
}

export function hydrateWhileError(
  el: Element,
  key: string,
  signalGetter: () => unknown,
  render: WhileErrorRender,
): () => void {
  let current: Element = el
  let branchDispose: (() => void) | null = null
  let firstRun = true

  const condDispose = effect(() => {
    let isError = false
    let errorVal: unknown
    try {
      const v = signalGetter() as { kind?: string; error?: unknown } | undefined
      isError = !!v && v.kind === 'error'
      errorVal = v?.error
    } catch {}
    if (firstRun) { firstRun = false; return }
    applyTransition(isError, errorVal)
  })

  function applyTransition(show: boolean, errorVal: unknown): void {
    if (branchDispose) { try { branchDispose() } catch {} ; branchDispose = null }
    const parent = current.parentNode
    if (!parent) return
    const owner = current.ownerDocument ?? document
    const next = owner.createElement('sail-while-error') as HTMLElement
    next.setAttribute('key', key)
    if (show) {
      next.style.display = 'contents'
      const result = render(errorVal)
      for (const node of result.nodes) next.appendChild(node)
      branchDispose = () => { try { result.cleanup() } catch {} }
    } else {
      next.setAttribute('hidden', '')
    }
    parent.replaceChild(next, current)
    current = next
  }

  return () => {
    try { condDispose() } catch {}
    if (branchDispose) { try { branchDispose() } catch {} ; branchDispose = null }
  }
}
