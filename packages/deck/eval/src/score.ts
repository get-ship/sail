// Scoring for one model output. Three sub-scores, each independent:
//   - compiled:    does the .sui parse, and (if there's a setup .ts)
//                  does it type-check?
//   - boundedScope: are there any escape-hatch patterns in the output?
//   - behavioral:  do the task's behavioral tests pass against the
//                  hydrated DOM?
// A pass requires all three.

import { typeCheckSetup } from './typecheck.ts'
import { createMockShip } from './mock-ship.ts'

// Lazy because @ship/sui-dom/test-utils boots a wasm compiler that
// only matters when we actually score a template. CLI --help and
// dispatch-only paths don't need it loaded.
async function loadDom(): Promise<{
  compileSui: (source: string) => { js: string }
  compileAndMount: (
    source: string,
    setup: undefined,
    opts: { container: HTMLElement; ship?: unknown },
  ) => { dispose: () => void }
}> {
  const mod = (await import('@ship/sui-dom/test-utils')) as {
    compileSui: (source: string, opts?: unknown) => { js: string }
    compileAndMount: (
      source: string,
      setup: undefined,
      opts: { container: HTMLElement; ship?: unknown },
    ) => { dispose: () => void }
  }
  return { compileSui: mod.compileSui, compileAndMount: mod.compileAndMount }
}

export interface ScoreInput {
  template: string
  setup?: string
  behavioralTests: string
  /** Override the default mock ship; tests run with createMockShip() otherwise. */
  ship?: unknown
}

export interface SubScores {
  compiled: boolean
  boundedScope: boolean
  behavioral: boolean
}

export interface ScoreReport extends SubScores {
  diagnostics: {
    compile?: string
    boundedScope?: string[]
    behavioral?: string
  }
}

/** Score a single model output. */
export async function scoreOutput(input: ScoreInput): Promise<ScoreReport> {
  const diagnostics: ScoreReport['diagnostics'] = {}

  const compileRes = await checkCompile(input.template, input.setup)
  if (!compileRes.ok && compileRes.error) {
    diagnostics.compile = compileRes.error
  }

  const sniff = sniffEscapeHatches(input.template, input.setup)
  if (sniff.length > 0) diagnostics.boundedScope = sniff

  let behavioral = false
  if (!compileRes.ok) {
    diagnostics.behavioral = 'skipped — compile failed'
  } else if (sniff.length > 0) {
    diagnostics.behavioral = 'skipped — bounded-scope check failed'
  } else {
    const behavRes = await runBehavioral(input)
    behavioral = behavRes.ok
    if (!behavRes.ok && behavRes.error) {
      diagnostics.behavioral = behavRes.error
    }
  }

  return {
    compiled: compileRes.ok,
    boundedScope: sniff.length === 0,
    behavioral,
    diagnostics,
  }
}

interface Outcome {
  ok: boolean
  error?: string
}

async function checkCompile(template: string, setup?: string): Promise<Outcome> {
  try {
    const { compileSui } = await loadDom()
    compileSui(template)
  } catch (err) {
    return { ok: false, error: `parse: ${describe(err)}` }
  }
  if (!setup) return { ok: true }
  return typeCheckSetup(setup)
}

const ESCAPE_HATCH_PATTERNS: Array<{ name: string; re: RegExp }> = [
  { name: 'inline <script>', re: /<script(?:\s[^>]*)?>[\s\S]*?<\/script>/i },
  { name: 'eval()', re: /\beval\s*\(/ },
  { name: 'new Function()', re: /\bnew\s+Function\s*\(/ },
  { name: 'document.write', re: /\bdocument\s*\.\s*write\b/ },
  { name: 'customElements.define', re: /\bcustomElements\s*\.\s*define\s*\(/ },
  { name: 'Function constructor', re: /\bFunction\s*\(\s*['"`]/ },
]

/** Static escape-hatch sniff over the template and (if present) setup. */
export function sniffEscapeHatches(template: string, setup?: string): string[] {
  const hits = new Set<string>()
  for (const p of ESCAPE_HATCH_PATTERNS) {
    if (p.re.test(template)) hits.add(`${p.name} in template`)
    if (setup && p.re.test(setup)) hits.add(`${p.name} in setup`)
  }
  return [...hits]
}

const AsyncFunctionCtor = Object.getPrototypeOf(
  async function () {},
).constructor as new (
  ...args: string[]
) => (ctx: BehavioralCtx) => Promise<void>

interface BehavioralCtx {
  container: HTMLElement
  expect: (cond: unknown, msg?: string) => void
  tick: () => Promise<void>
  ship: unknown
}

async function runBehavioral(input: ScoreInput): Promise<Outcome> {
  if (typeof document === 'undefined') {
    return { ok: false, error: 'behavioral: no document — run inside jsdom' }
  }
  const container = document.createElement('div')
  document.body.appendChild(container)
  const ship = input.ship ?? createMockShip()
  let handle: { dispose: () => void }
  try {
    const { compileAndMount } = await loadDom()
    handle = compileAndMount(input.template, undefined, {
      container,
      ship,
    })
  } catch (err) {
    container.remove()
    return { ok: false, error: `hydrate: ${describe(err)}` }
  }
  try {
    const fn = new AsyncFunctionCtor(
      'ctx',
      `const { container, expect, tick, ship } = ctx;\n${input.behavioralTests}`,
    )
    const ctx: BehavioralCtx = {
      container,
      expect: (cond, msg) => {
        if (!cond) throw new Error(msg ?? 'expect failed')
      },
      tick: () => new Promise((r) => setTimeout(r, 0)),
      ship,
    }
    await fn(ctx)
    return { ok: true }
  } catch (err) {
    return { ok: false, error: `behavioral: ${describe(err)}` }
  } finally {
    handle.dispose()
    container.remove()
  }
}

function describe(err: unknown): string {
  if (err instanceof Error) return err.message
  return String(err)
}
