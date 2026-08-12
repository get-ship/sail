/**
 * Lazy LSP loaders — kept outside the controller so dynamic `import()` is not
 * ambient-authority in the confined controller region (SPEC-L2-PACKAGE-1 O4).
 * Package-local modules may use dynamic import; controllers must not.
 */

export type LspDiagnoseFn = (
  text: string,
  wasmUrl?: string,
) => Promise<
  Array<{
    line: number
    column?: number
    endLine?: number
    endColumn?: number
    severity: 'error' | 'warning' | 'info'
    message: string
    source?: string
  }>
>

export type LspAllSourcesFn = () => Array<{
  trigger: string
  items: Array<{
    label: string
    detail?: string
    info?: string
    insert?: string
    kind?: string
  }>
}>

export type LspHoverFn = (
  source: string,
  offset: number,
) => { contents: string } | null

export async function loadLspModules(): Promise<{
  diagnose: LspDiagnoseFn | null
  allSources: LspAllSourcesFn | null
  hoverAt: LspHoverFn | null
}> {
  let diagnose: LspDiagnoseFn | null = null
  let allSources: LspAllSourcesFn | null = null
  let hoverAt: LspHoverFn | null = null
  try {
    const [diag, comp] = await Promise.all([
      import('@ship/sail-lsp-browser/diagnostics') as Promise<{ diagnose: LspDiagnoseFn }>,
      import('@ship/sail-lsp-browser/completions') as Promise<{ allSources: LspAllSourcesFn }>,
    ])
    diagnose = diag.diagnose
    allSources = comp.allSources
  } catch (err) {
    console.warn('[sail-code-editor] LSP init failed; diagnostics unavailable', err)
  }
  try {
    const hoverMod = (await import(
      /* @vite-ignore */ '@ship/sail-lsp-browser/hover'
    )) as { hoverAt: LspHoverFn }
    if (typeof hoverMod.hoverAt === 'function') hoverAt = hoverMod.hoverAt
  } catch {
    hoverAt = null
  }
  return { diagnose, allSources, hoverAt }
}
