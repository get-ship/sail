// Module evaluator for compiled `.sui` output — the render-engine seam's
// "runtime evaluator" (build-spec §Mount by raw .sui source, option B).
//
// The compiler emits an ES module whose only imports are the framework
// specifiers (`@ship/sui-runtime`, `@ship/sui-dom`, `@ship/sui-runtime/route`,
// the side-effect `@ship/sui-runtime/template-hooks`) plus an optional
// `import setup from "<marker>"` when a setup is injected. Those specifiers
// resolve to the SAME singleton modules this engine is statically linked
// against — so the evaluator rewrites the bare imports to reference the
// injected namespaces and runs the body. Static linkage (not a fetch, not a
// daemon) keeps the single-runtime guarantee: the specimen shares the host
// page's `@ship/sui-runtime` instance, so signals/effects/boundary events
// interoperate across the seam.
//
// NOT a test shortcut: the framework namespaces are real static `import`s the
// bundler keeps, so this is the production evaluator a standalone SPA ships
// (the deck importmap externalizes the same specifiers to the same chunks).

import * as suiRuntime from '@ship/sui-runtime'
import * as suiDom from '@ship/sui-dom'
import * as suiRoute from '@ship/deck-router'
import '@ship/sui-runtime/template-hooks'
import type { ViewDef } from '@ship/sui-runtime'

/** A user setup the host injects for a view-mode specimen (the throwing-setup
 *  diagnostic path, or a future ctx-bearing view). */
export type InjectedSetup =
  | ((ctx: unknown) => Record<string, unknown> | void)
  | { setup: (ctx: unknown) => unknown }

function rewriteImports(js: string): string {
  return (
    js
      // import { …names… } from "@ship/sui-runtime"
      .replace(
        /^\s*import\s*\{\s*([^}]+?)\s*\}\s*from\s*["']@ship\/sui-runtime["']\s*;?\s*$/gm,
        'const { $1 } = __runtime;',
      )
      // import { …names… } from "@ship/sui-dom"
      .replace(
        /^\s*import\s*\{\s*([^}]+?)\s*\}\s*from\s*["']@ship\/sui-dom["']\s*;?\s*$/gm,
        'const { $1 } = __dom;',
      )
      // import { route as __routeFactory } from "@ship/sui-runtime/route"
      .replace(
        /^\s*import\s*\{\s*route\s+as\s+__routeFactory\s*\}\s*from\s*["']@ship\/deck\/route["']\s*;?\s*$/gm,
        'const __routeFactory = __route.route;',
      )
      // side-effect: import "@ship/sui-runtime"
      .replace(/^\s*import\s+["']@ship\/sui-runtime["']\s*;?\s*$/gm, '')
      // side-effect: import "@ship/sui-runtime/template-hooks"
      .replace(/^\s*import\s+["']@ship\/sui-runtime\/template-hooks["']\s*;?\s*$/gm, '')
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
 * Evaluate a compiled view-mode module and return its `{ setup }` default —
 * the same `ViewDef` the compiler's `view((ctx) => …)` wrapper produces.
 */
export function evalViewModule(js: string, setup?: InjectedSetup): ViewDef {
  const body = rewriteImports(js).replace(/\bexport\s+default\s+/g, 'return ')
  // eslint-disable-next-line @typescript-eslint/no-implied-eval
  const fn = new Function('__runtime', '__dom', '__route', '__setup', body) as (
    r: typeof suiRuntime,
    d: typeof suiDom,
    rt: typeof suiRoute,
    s: { setup: (ctx: unknown) => unknown } | null,
  ) => ViewDef
  return fn(suiRuntime, suiDom, suiRoute, normalizeSetup(setup))
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
    r: typeof suiRuntime,
    d: typeof suiDom,
    rt: typeof suiRoute,
    s: null,
  ) => void
  fn(suiRuntime, suiDom, suiRoute, null)
}
