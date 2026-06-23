// Local declaration shim used only for type-checking the eval package
// in isolation. Runtime resolution still goes through the workspace
// alias (vitest config) or the published package main field.
//
// Mirrors the public surface of @ship/sui-dom we depend on. Keep in
// sync with packages/ship-set-dom/src/index.ts.

declare module '@ship/sui-dom' {
  export interface HydrateOptions {
    container: HTMLElement
    route?: Record<string, string>
    ship?: unknown
    navigate?: (path: string) => void
  }

  export interface HydrateHandle {
    dispose: () => void
  }

  export function hydrate(
    template: string,
    setup?: (() => unknown) | undefined,
    opts?: HydrateOptions,
  ): HydrateHandle

  export interface ParsedSource {
    data: unknown[]
    state: unknown[]
    component: unknown | null
    templateNodes: Node[]
    doc: Document
  }

  export function parseSetSource(source: string): ParsedSource
}
