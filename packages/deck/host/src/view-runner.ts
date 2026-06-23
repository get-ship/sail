//! High-level Deck view confinement: assemble the endowment surface + confine a
//! whole compiled view into the `{ default: { setup } }` shape `mount()` accepts
//! as `opts.module`. The host calls these; `mount.ts` (SUI) is unchanged — it
//! already takes a pre-resolved module, so Deck drives confinement without a
//! SUI→Deck dependency or a live-path edit. Default (unconfined) mount is
//! untouched; a host opts in per view.

import { confineController } from './confine'
import { lowerModuleToExpression } from './lower-imports'

/** The minimal node-factory surface the compiled view's renderer needs. */
interface NodeFactories {
  createElement: (tag: string) => unknown
  createElementNS?: (ns: string, tag: string) => unknown
  createTextNode: (text: string) => unknown
  createComment: (text: string) => unknown
  createDocumentFragment: () => unknown
}

/** Capability-narrow a real `document` to node factories only — node creation
 *  returns DETACHED nodes the host attaches. `querySelector`, `cookie`,
 *  `innerHTML`, write access etc. are NOT carried over. */
function narrowDocument(doc: Document): NodeFactories {
  return {
    createElement: doc.createElement.bind(doc),
    createElementNS: doc.createElementNS.bind(doc),
    createTextNode: doc.createTextNode.bind(doc),
    createComment: doc.createComment.bind(doc),
    createDocumentFragment: doc.createDocumentFragment.bind(doc),
  }
}

export interface ViewEndowmentInput {
  /** `@ship/sui-runtime` namespace — signal/derived/effect/controller/… The
   *  host injects it (keeps deck-host decoupled from a hard runtime dep). */
  runtime: Record<string, unknown>
  /** `@ship/sui-dom` namespace — bindText/bindAttr/… (run host-side). */
  dom?: Record<string, unknown>
  /** Policy-gated op surface (op-registry `ship`, NOT the full SDK). */
  ship: unknown
  route?: unknown
  navigate?: (path: string) => void
  /** Defaults to the realm's `document`; the view sees only its node factories. */
  doc?: Document
}

/**
 * Build the hardened endowment surface a confined view evaluates against: the
 * runtime + dom binding surfaces, a node-factory-only `document`, and the
 * policy-gated ops/route. `harden`-ed so the view can't mutate shared host
 * state through them. Requires {@link lockdownRealm} to have run.
 */
export function buildViewEndowments(input: ViewEndowmentInput): Record<string, unknown> {
  const doc = input.doc ?? (globalThis as { document?: Document }).document
  const surface: Record<string, unknown> = {
    ...input.runtime,
    ...(input.dom ?? {}),
    document: doc ? narrowDocument(doc) : undefined,
    ship: input.ship,
  }
  if (input.route !== undefined) surface.route = input.route
  if (input.navigate) surface.navigate = input.navigate
  return harden(surface) as Record<string, unknown>
}

/** The pre-resolved module shape `mount()` accepts as `opts.module`. */
export interface CompiledViewModule {
  default: { setup: (ctx: unknown) => Record<string, unknown> }
}

/**
 * Confine a whole compiled view module `source` against `endowments` and return
 * it in the `{ default: { setup } }` shape `mount()` consumes. The view's glue +
 * bundled controller run inside one Compartment; ambient authority is denied.
 */
export function confineViewModule(
  source: string,
  endowments: Record<string, unknown>,
): CompiledViewModule {
  const def = confineController<{ setup: (ctx: unknown) => Record<string, unknown> }>(
    lowerModuleToExpression(source),
    endowments,
  )
  return { default: def }
}

// `harden` is a realm global installed by `import 'ses'` (via ./confine.ts).
declare function harden<T>(value: T): T
