// Behavioral scenario runner. Executes a behavior spec (see behavior-spec.md)
// against a mounted route and reports pass/fail per scenario. Queries are by
// what a USER perceives — visible text, role+name, field label — never CSS or
// signal names, so two faithful-but-different builds of the same behavior both
// pass.
//
// Substrate note: this runs on `renderRoute` (jsdom) — correct for the EVAL,
// which grades in Ship's own infra. The in-app author path is the same scenario
// format mounted in the Tauri webview; that's a separate runner, not this one.

import { renderRoute, type ControllerInput } from './render'

export type Step =
  | { see: string }
  | { notSee: string }
  | { click: string }
  | { fill: { field: string; value: string } }
  | { choose: { option: string; in: string } }

export interface Scenario {
  name: string
  given?: { url?: string; ship?: Record<string, unknown> }
  steps: Step[]
}

export interface BehaviorSpec {
  view: string
  scenarios: Scenario[]
}

export interface ScenarioResult {
  name: string
  passed: boolean
  /** 0-based index of the step that failed, if any. */
  failedStep?: number
  error?: string
}

/* --------------------------- accessible queries --------------------------- */
// Debt: weaker parallel a11y helpers (CSS role selectors + label-only name).
// Prefer @ship/sail-test getByRole for app behavioral suites. Re-home this
// runner onto sail-test when next touching scenario eval (not mass-rewritten
// on SPEC-TEST-1 FIX).

const ROLE_SELECTORS: Record<string, string> = {
  button: 'button, [role="button"]',
  link: 'a[href], [role="link"]',
  heading: 'h1, h2, h3, h4, h5, h6, [role="heading"]',
  textbox: 'input:not([type="checkbox"]):not([type="radio"]), textarea, [role="textbox"]',
}

/** aria-label, else trimmed visible text. */
function accessibleName(el: Element): string {
  const aria = el.getAttribute('aria-label')
  if (aria) return aria.trim()
  return (el.textContent ?? '').trim()
}

interface Target {
  role?: string
  name: string
}

function parseTarget(s: string): Target {
  const m = s.match(/^role:(\S+)\s+name:(.+)$/)
  if (m) return { role: m[1]!, name: m[2]!.trim() }
  return { name: s }
}

function byRole(root: HTMLElement, role: string, name: string): Element[] {
  const sel = ROLE_SELECTORS[role] ?? `[role="${role}"]`
  return Array.from(root.querySelectorAll(sel)).filter(
    (el) => accessibleName(el).toLowerCase() === name.toLowerCase(),
  )
}

/** Visible text present anywhere under root. */
function textPresent(root: HTMLElement, text: string): boolean {
  return (root.textContent ?? '').toLowerCase().includes(text.toLowerCase())
}

function findClickable(root: HTMLElement, t: Target): Element | null {
  if (t.role) return byRole(root, t.role, t.name)[0] ?? null
  // Bare label: prefer an actual control whose name matches, else any element
  // whose own text equals the label.
  const controls = Array.from(root.querySelectorAll('button, a[href], [role="button"], [role="link"]'))
  const exact = controls.find((el) => accessibleName(el).toLowerCase() === t.name.toLowerCase())
  if (exact) return exact
  const all = Array.from(root.querySelectorAll('*'))
  return all.find((el) => (el.textContent ?? '').trim().toLowerCase() === t.name.toLowerCase()) ?? null
}

function findField(root: HTMLElement, field: string): HTMLInputElement | HTMLTextAreaElement | null {
  const f = field.toLowerCase()
  // 1. <label> text → associated control
  for (const label of Array.from(root.querySelectorAll('label'))) {
    if (!(label.textContent ?? '').toLowerCase().includes(f)) continue
    const forId = label.getAttribute('for')
    if (forId) {
      const byId = root.ownerDocument?.getElementById(forId)
      if (byId && root.contains(byId)) return byId as HTMLInputElement
    }
    const nested = label.querySelector('input, textarea')
    if (nested) return nested as HTMLInputElement
  }
  // 2. name / aria-label / placeholder
  return (root.querySelector(
    `input[name="${field}"], textarea[name="${field}"], ` +
      `input[aria-label="${field}"], input[placeholder="${field}"]`,
  ) as HTMLInputElement | null)
}

/* ------------------------------- step exec ------------------------------- */

function runStep(root: HTMLElement, step: Step): void {
  if ('see' in step) {
    const t = parseTarget(step.see)
    const ok = t.role ? byRole(root, t.role, t.name).length > 0 : textPresent(root, t.name)
    if (!ok) throw new Error(`expected to see "${step.see}"`)
    return
  }
  if ('notSee' in step) {
    const t = parseTarget(step.notSee)
    const present = t.role ? byRole(root, t.role, t.name).length > 0 : textPresent(root, t.name)
    if (present) throw new Error(`expected NOT to see "${step.notSee}"`)
    return
  }
  if ('click' in step) {
    const el = findClickable(root, parseTarget(step.click))
    if (!el) throw new Error(`no clickable element named "${step.click}"`)
    el.dispatchEvent(new Event('click', { bubbles: true }))
    return
  }
  if ('fill' in step) {
    const input = findField(root, step.fill.field)
    if (!input) throw new Error(`no input for field "${step.fill.field}"`)
    input.value = step.fill.value
    input.dispatchEvent(new Event('input', { bubbles: true }))
    input.dispatchEvent(new Event('change', { bubbles: true }))
    return
  }
  if ('choose' in step) {
    const groupText = step.choose.in.toLowerCase()
    // Find the smallest container whose heading/label/legend mentions the group.
    const labels = Array.from(root.querySelectorAll('h1,h2,h3,h4,h5,h6,legend,label,[aria-label]'))
    const groupLabel = labels.find((el) =>
      (accessibleName(el) || '').toLowerCase().includes(groupText),
    )
    const scope = (groupLabel?.closest('fieldset, section, div, ul') as HTMLElement) ?? root
    const option = Array.from(scope.querySelectorAll('*')).find(
      (el) => (el.textContent ?? '').trim().toLowerCase() === step.choose.option.toLowerCase(),
    )
    if (!option) throw new Error(`no option "${step.choose.option}" in group "${step.choose.in}"`)
    option.dispatchEvent(new Event('click', { bubbles: true }))
    return
  }
}

/* ------------------------------- fake ship ------------------------------- */

/** Build a fake `ship` SDK from `given.ship` ("domain.op" → value). Each op
 *  returns a live source that immediately resolves `ok`. */
function buildShip(given: Record<string, unknown> | undefined): unknown {
  const ship: Record<string, Record<string, unknown>> = {}
  for (const [key, value] of Object.entries(given ?? {})) {
    const [domain, op] = key.split('.')
    if (!domain || !op) continue
    ship[domain] ??= {}
    ship[domain]![op] = () => ({
      subscribe: (sig: { set: (s: unknown) => void }) => {
        sig.set({ kind: 'ok', value })
        return () => {}
      },
    })
  }
  return ship
}

/* --------------------------------- run --------------------------------- */

/**
 * Run every scenario in a spec against the given route source + controller.
 * Each scenario mounts a fresh route, so they're isolated.
 */
export async function runScenarios(
  source: string,
  controller: ControllerInput | undefined,
  spec: BehaviorSpec,
): Promise<ScenarioResult[]> {
  const results: ScenarioResult[] = []
  for (const scenario of spec.scenarios) {
    // Build the RenderOpts object conditionally so undefined
    // controller / url / providers don't trip exactOptionalPropertyTypes.
    const opts: {
      controller?: ControllerInput
      providers?: Record<string, unknown>
      url?: string
    } = {}
    if (controller !== undefined) opts.controller = controller
    // Scenario specs may still key a fake under `given.ship` — treat it as
    // a flat provider named by the scenario (or a no-op empty bag).
    const fake = buildShip(scenario.given?.ship)
    if (fake !== undefined) opts.providers = { ship: fake }
    if (scenario.given?.url !== undefined) opts.url = scenario.given.url
    const v = renderRoute(source, opts)
    let result: ScenarioResult = { name: scenario.name, passed: true }
    let i = 0
    try {
      for (; i < scenario.steps.length; i++) {
        runStep(v.root, scenario.steps[i]!)
        await v.flush()
      }
    } catch (err) {
      result = {
        name: scenario.name,
        passed: false,
        failedStep: i,
        error: err instanceof Error ? err.message : String(err),
      }
    } finally {
      v.unmount()
    }
    results.push(result)
  }
  return results
}
