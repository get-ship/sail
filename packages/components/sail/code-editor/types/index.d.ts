/**
 * Public types for `@ship/sail-components/code`.
 *
 * Checked in (not tsc-emit): the side-effect entry imports a generated
 * `.controller.ts` companion that is gitignored, so declaration emit cannot
 * follow `src/index.ts`. Keep this surface aligned with:
 * - `src/sail-code-editor/sail-code-editor.sail` (`SailDiagnostic` / API)
 * - `src/sail-code-editor/sail-code-editor.ops.ts`
 *
 * Side-effect: `import '@ship/sail-components/code'` registers `<sail-code-editor>`.
 */

export interface SailDiagnostic {
  line: number
  column?: number
  endLine?: number
  endColumn?: number
  severity: 'error' | 'warning' | 'info'
  message: string
  source?: string
}

export interface SailCompletionSource {
  trigger: string
  items: Array<{
    label: string
    detail?: string
    info?: string
    insert?: string
  }>
}

/** Imperative host API on the upgraded custom element. */
export interface SailCodeEditorApi {
  value: string
  setDiagnostics(diagnostics: SailDiagnostic[]): void
  setCompletions(sources: SailCompletionSource[]): void
  setCursor?(line: number, col?: number): void
  focus(): void
}

export interface OpsHoverResult {
  contents: string
  range?: {
    line?: number
    column?: number
    endLine?: number
    endColumn?: number
  }
}

export interface OpsCompletionItem {
  label: string
  detail?: string
  info?: string
  insert?: string
  kind?: string
}

export interface OpsDiagnostic {
  line: number
  column?: number
  endLine?: number
  endColumn?: number
  severity: 'error' | 'warning' | 'info'
  message: string
  source?: string
}

export interface OpsPosArgs {
  source?: string
  path?: string
  line: number
  column: number
  signal?: AbortSignal
}

export interface OpsDiagArgs {
  source?: string
  path?: string
  signal?: AbortSignal
}

export interface OpsAuthoringClient {
  hover(args: OpsPosArgs): Promise<OpsHoverResult | null>
  complete(args: OpsPosArgs): Promise<OpsCompletionItem[]>
  diagnostics(args: OpsDiagArgs): Promise<OpsDiagnostic[]>
  supportsHover(): boolean | null
  supportsComplete(): boolean | null
  supportsDiagnostics(): boolean | null
}

export declare function createOpsAuthoringClient(opts: {
  baseUrl: string
  fetch?: typeof fetch
}): OpsAuthoringClient

export declare function normalizeOpsBase(
  url: string | null | undefined,
): string | null

export declare function offsetToLineCol(
  source: string,
  offset: number,
): { line: number; column: number }

export declare function parseOpsDiagnostics(raw: unknown): OpsDiagnostic[]
