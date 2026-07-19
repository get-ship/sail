// Module evaluator for compiled `.sail` output — the render-engine seam's
// "runtime evaluator" (build-spec §Mount by raw .sail source, option B).
//
// The compiler emits an ES module whose only imports are the framework
// specifiers (`@ship/sail-runtime`, `@ship/sail-dom`, `@ship/sail-runtime/route`,
// the side-effect `@ship/sail-runtime/template-hooks`) plus an optional
// `import setup from "<marker>"` when a setup is injected. Those specifiers
// resolve to the SAME singleton modules this engine is statically linked
// against — so the evaluator rewrites the bare imports to reference the
// injected namespaces and runs the body. Static linkage (not a fetch, not a
// daemon) keeps the single-runtime guarantee: the specimen shares the host
// page's `@ship/sail-runtime` instance, so signals/effects/boundary events
// interoperate across the seam.
//
// NOT a test shortcut: the framework namespaces are real static `import`s the
// bundler keeps, so this is the production evaluator a standalone SPA ships
// (the deck importmap externalizes the same specifiers to the same chunks).

import * as sailRuntime from '@ship/sail-runtime'
import * as sailDom from '@ship/sail-dom'
import * as sailRoute from '@ship/sail-runtime/route'
import '@ship/sail-runtime/template-hooks'
import type { MountModule } from '@ship/sail-runtime'

/** A user setup the host injects for a view-mode specimen (the throwing-setup
 *  diagnostic path, or a future ctx-bearing view). */
export type InjectedSetup =
  | ((ctx: unknown) => Record<string, unknown> | void)
  | { setup: (ctx: unknown) => unknown }

function rewriteImports(js: string): string {
  return (
    js
      // import { …names… } from "@ship/sail-runtime/internal"
      .replace(
        /^\s*import\s*\{\s*([^}]+?)\s*\}\s*from\s*["']@ship\/sail-runtime\/internal["']\s*;?\s*$/gm,
        'const { $1 } = __runtime;',
      )
      // import { …names… } from "@ship/sail-runtime"
      .replace(
        /^\s*import\s*\{\s*([^}]+?)\s*\}\s*from\s*["']@ship\/sail-runtime["']\s*;?\s*$/gm,
        'const { $1 } = __runtime;',
      )
      // import { …names… } from "@ship/sail-dom"
      .replace(
        /^\s*import\s*\{\s*([^}]+?)\s*\}\s*from\s*["']@ship\/sail-dom["']\s*;?\s*$/gm,
        'const { $1 } = __dom;',
      )
      // import { route as __routeFactory } from "@ship/sail-runtime/route"
      .replace(
        /^\s*import\s*\{\s*route\s+as\s+__routeFactory\s*\}\s*from\s*["']@ship\/deck\/route["']\s*;?\s*$/gm,
        'const __routeFactory = __route.route;',
      )
      // side-effect: import "@ship/sail-runtime"
      .replace(/^\s*import\s+["']@ship\/sail-runtime["']\s*;?\s*$/gm, '')
      // side-effect: import "@ship/sail-runtime/template-hooks"
      .replace(/^\s*import\s+["']@ship\/sail-runtime\/template-hooks["']\s*;?\s*$/gm, '')
      // import setup from "<marker>" — the injected controller/setup
      .replace(/^\s*import\s+setup\s+from\s+["'][^"']+["']\s*;?\s*$/gm, 'const setup = __setup;')
  )
}

function normalizeSetup(setup: InjectedSetup | undefined): { setup: (ctx: unknown) => unknown } | null {
  if (!setup) return null
  if (typeof setup === 'function') return { setup: (ctx) => setup(ctx) ?? {} }
  return setup
}

/**
 * Evaluate a compiled mount-form module and return its `{ setup }` default —
 * the same `MountModule` the compiler's `component((ctx) => …)` wrapper produces.
 */
export function evalViewModule(js: string, setup?: InjectedSetup): MountModule {
  const body = rewriteImports(js).replace(/\bexport\s+default\s+/g, 'return ')
  // eslint-disable-next-line @typescript-eslint/no-implied-eval
  const fn = new Function('__runtime', '__dom', '__route', '__setup', body) as (
    r: typeof sailRuntime,
    d: typeof sailDom,
    rt: typeof sailRoute,
    s: { setup: (ctx: unknown) => unknown } | null,
  ) => MountModule
  return fn(sailRuntime, sailDom, sailRoute, normalizeSetup(setup))
}

/**
 * Evaluate a compiled COMPONENT-mode module. The body is a side-effecting
 * `component(tag, def)` call that runs `customElements.define(tag, …)`. There
 * is no value to return — registration is the effect. The caller guards
 * `customElements.get(tag)` before calling so a redefine never throws
 * `NotSupportedError`.
 */
export function evalComponentModule(js: string): void {
  const body = rewriteImports(js).replace(/\bexport\s+default\s+/g, '')
  // eslint-disable-next-line @typescript-eslint/no-implied-eval
  const fn = new Function('__runtime', '__dom', '__route', '__setup', body) as (
    r: typeof sailRuntime,
    d: typeof sailDom,
    rt: typeof sailRoute,
    s: null,
  ) => void
  fn(sailRuntime, sailDom, sailRoute, null)
}
