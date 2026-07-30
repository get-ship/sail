/**
 * Behavior grader — scenario format borrowed as an *idea* from ship-eval;
 * implementation is self-contained (no shared imports).
 *
 * Cheap unit path: pure DOM-less step validation against a static HTML string
 * (or skipped when no browser / no HTML).
 * Frontier path: optional playwright against a live base URL (deck serve).
 */

import type { BehaviorGrade, BehaviorSpec, Scenario, Step } from '../types.ts'

export interface BehaviorGradeOpts {
  spec: BehaviorSpec | null
  /**
   * HTML document body to query (static). When provided, runs the pure
   * scenario runner without a browser — good for fixtures/tests.
   */
  html?: string
  /**
   * Live base URL for playwright (e.g. http://127.0.0.1:7878).
   * Requires playwright optional dependency.
   */
  baseUrl?: string
  /** When neither html nor baseUrl, skip rather than fail. */
  allowSkip?: boolean
}

export async function gradeBehavior(opts: BehaviorGradeOpts): Promise<BehaviorGrade> {
  const allowSkip = opts.allowSkip ?? false
  if (!opts.spec || !opts.spec.scenarios?.length) {
    return {
      ok: true,
      passed: 0,
      failed: 0,
      results: [],
      skipped: true,
      detail: 'no behavior spec',
    }
  }

  if (opts.html != null) {
    const results = runScenariosOnHtml(opts.html, opts.spec.scenarios)
    const failed = results.filter((r) => !r.passed).length
    const passed = results.length - failed
    return { ok: failed === 0, passed, failed, results }
  }

  if (opts.baseUrl) {
    try {
      const results = await runScenariosPlaywright(opts.baseUrl, opts.spec.scenarios)
      const failed = results.filter((r) => !r.passed).length
      const passed = results.length - failed
      return { ok: failed === 0, passed, failed, results }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      if (allowSkip && /Cannot find package|Cannot find module|playwright/i.test(msg)) {
        return {
          ok: true,
          passed: 0,
          failed: 0,
          results: [],
          skipped: true,
          detail: `playwright unavailable: ${msg}`,
        }
      }
      return {
        ok: false,
        passed: 0,
        failed: opts.spec.scenarios.length,
        results: opts.spec.scenarios.map((s) => ({
          name: s.name,
          passed: false,
          error: msg,
        })),
        detail: msg,
      }
    }
  }

  if (allowSkip) {
    return {
      ok: true,
      passed: 0,
      failed: 0,
      results: [],
      skipped: true,
      detail: 'behavior leg skipped (no html fixture or baseUrl)',
    }
  }
  return {
    ok: false,
    passed: 0,
    failed: opts.spec.scenarios.length,
    results: [],
    detail: 'behavior leg requires html or baseUrl',
  }
}

/* ── pure HTML scenario runner (happy-dom-free, regex/text based) ── */

interface ScenarioResult {
  name: string
  passed: boolean
  failedStep?: number
  error?: string
}

export function runScenariosOnHtml(html: string, scenarios: Scenario[]): ScenarioResult[] {
  return scenarios.map((s) => runOne(html, s))
}

function runOne(html: string, scenario: Scenario): ScenarioResult {
  // Mutable document text for fill/click approximations.
  let doc = html
  for (let i = 0; i < scenario.steps.length; i++) {
    try {
      doc = applyStep(doc, scenario.steps[i]!)
    } catch (e) {
      return {
        name: scenario.name,
        passed: false,
        failedStep: i,
        error: e instanceof Error ? e.message : String(e),
      }
    }
  }
  return { name: scenario.name, passed: true }
}

function applyStep(html: string, step: Step): string {
  if ('see' in step) {
    if (!visibleMatch(html, step.see)) {
      throw new Error(`expected to see "${step.see}"`)
    }
    return html
  }
  if ('notSee' in step) {
    if (visibleMatch(html, step.notSee)) {
      throw new Error(`expected NOT to see "${step.notSee}"`)
    }
    return html
  }
  if ('click' in step) {
    if (!visibleMatch(html, step.click) && !roleNameMatch(html, step.click)) {
      throw new Error(`no clickable element named "${step.click}"`)
    }
    return html
  }
  if ('fill' in step) {
    if (!fieldPresent(html, step.fill.field)) {
      throw new Error(`no input for field "${step.fill.field}"`)
    }
    // Annotate filled value so later `see` can observe it.
    return html + `<!--filled:${step.fill.field}=${step.fill.value}-->`
  }
  if ('choose' in step) {
    if (!visibleMatch(html, step.choose.option) || !visibleMatch(html, step.choose.in)) {
      throw new Error(`no option "${step.choose.option}" in group "${step.choose.in}"`)
    }
    return html + `<!--chose:${step.choose.option}-->`
  }
  return html
}

function stripTags(html: string): string {
  return html
    .replace(/<script[\s\S]*?<\/script>/gi, ' ')
    .replace(/<style[\s\S]*?<\/style>/gi, ' ')
    .replace(/<!--[\s\S]*?-->/g, ' ')
    .replace(/<[^>]+>/g, ' ')
    .replace(/\s+/g, ' ')
}

function visibleMatch(html: string, query: string): boolean {
  if (query.startsWith('role:')) {
    return roleNameMatch(html, query)
  }
  const text = stripTags(html) + extractFilledValues(html)
  return text.toLowerCase().includes(query.toLowerCase())
}

function extractFilledValues(html: string): string {
  const vals: string[] = []
  for (const m of html.matchAll(/<!--filled:[^=]*=([^>]+)-->/g)) {
    vals.push(m[1] ?? '')
  }
  for (const m of html.matchAll(/<!--chose:([^>]+)-->/g)) {
    vals.push(m[1] ?? '')
  }
  return ' ' + vals.join(' ')
}

function roleNameMatch(html: string, query: string): boolean {
  const m = query.match(/^role:(\S+)\s+name:(.+)$/i)
  if (!m) {
    // bare click label
    return stripTags(html).toLowerCase().includes(query.toLowerCase())
  }
  const role = m[1]!.toLowerCase()
  const name = m[2]!.trim().toLowerCase()
  const roleTags: Record<string, RegExp> = {
    button: /<button\b[^>]*>[\s\S]*?<\/button>|<[^>]+role=["']button["'][^>]*>/gi,
    link: /<a\b[^>]*>[\s\S]*?<\/a>|<[^>]+role=["']link["'][^>]*>/gi,
    heading: /<h[1-6]\b[^>]*>[\s\S]*?<\/h[1-6]>/gi,
    textbox: /<input\b[^>]*>|<textarea\b[^>]*>[\s\S]*?<\/textarea>/gi,
  }
  const re = roleTags[role]
  if (!re) {
    return stripTags(html).toLowerCase().includes(name)
  }
  const chunks = html.match(re) ?? []
  return chunks.some((c) => stripTags(c).toLowerCase().includes(name) || c.toLowerCase().includes(`aria-label="${name}"`))
}

function fieldPresent(html: string, field: string): boolean {
  const f = field.toLowerCase()
  // label text
  if (new RegExp(`<label\\b[^>]*>[^<]*${escapeRe(field)}`, 'i').test(html)) return true
  if (new RegExp(`aria-label=["'][^"']*${escapeRe(field)}`, 'i').test(html)) return true
  if (new RegExp(`placeholder=["'][^"']*${escapeRe(field)}`, 'i').test(html)) return true
  if (new RegExp(`name=["']${escapeRe(field)}["']`, 'i').test(html)) return true
  // sail <label>Email</label><input ...> patterns — check label then input nearby
  const text = stripTags(html).toLowerCase()
  return text.includes(f) && /<input\b|<textarea\b/i.test(html)
}

function escapeRe(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

/* ── playwright (optional) ── */

/**
 * Run scenarios against a live base URL via playwright.
 * Exported for tests that inject a fake browser module via dependency injection
 * is impractical; unit tests cover the control-flow helper instead.
 */
export async function runScenariosPlaywright(
  baseUrl: string,
  scenarios: Scenario[],
): Promise<ScenarioResult[]> {
  // Dynamic import so unit tests don't need playwright installed.
  const pw = await import('playwright')
  const browser = await pw.chromium.launch({ headless: true })
  const results: ScenarioResult[] = []
  try {
    for (const scenario of scenarios) {
      const page = await browser.newPage()
      try {
        const url = scenario.given?.url
          ? new URL(scenario.given.url, baseUrl).toString()
          : baseUrl
        await page.goto(url, { waitUntil: 'networkidle', timeout: 30_000 })
        const stepResult = await runPlaywrightScenarioSteps(page, scenario)
        results.push(stepResult)
      } catch (e) {
        results.push({
          name: scenario.name,
          passed: false,
          error: e instanceof Error ? e.message : String(e),
        })
      } finally {
        await page.close().catch(() => {})
      }
    }
  } finally {
    await browser.close()
  }
  return results
}

/**
 * Execute steps for one scenario. On first step failure returns a single fail
 * result (never also a pass). Exported for unit tests without a real browser.
 */
export async function runPlaywrightScenarioSteps(
  page: Parameters<typeof applyPlaywrightStep>[0],
  scenario: Scenario,
): Promise<ScenarioResult> {
  for (let i = 0; i < scenario.steps.length; i++) {
    try {
      await applyPlaywrightStep(page, scenario.steps[i]!)
    } catch (e) {
      return {
        name: scenario.name,
        passed: false,
        failedStep: i,
        error: e instanceof Error ? e.message : String(e),
      }
    }
  }
  return { name: scenario.name, passed: true }
}

async function applyPlaywrightStep(
  page: {
    getByText: (t: string | RegExp, o?: { exact?: boolean }) => { first: () => { isVisible: (o?: { timeout?: number }) => Promise<boolean>; click: () => Promise<void> } }
    getByRole: (role: string, o?: { name?: string | RegExp }) => { first: () => { isVisible: (o?: { timeout?: number }) => Promise<boolean>; click: () => Promise<void>; fill: (v: string) => Promise<void> } }
    getByLabel: (t: string | RegExp) => { first: () => { fill: (v: string) => Promise<void>; isVisible: (o?: { timeout?: number }) => Promise<boolean> } }
  },
  step: Step,
): Promise<void> {
  if ('see' in step) {
    const t = parseTarget(step.see)
    if (t.role) {
      const ok = await page.getByRole(t.role, { name: t.name }).first().isVisible({ timeout: 5000 }).catch(() => false)
      if (!ok) throw new Error(`expected to see "${step.see}"`)
    } else {
      const ok = await page.getByText(t.name, { exact: false }).first().isVisible({ timeout: 5000 }).catch(() => false)
      if (!ok) throw new Error(`expected to see "${step.see}"`)
    }
    return
  }
  if ('notSee' in step) {
    const t = parseTarget(step.notSee)
    const visible = t.role
      ? await page.getByRole(t.role, { name: t.name }).first().isVisible({ timeout: 1000 }).catch(() => false)
      : await page.getByText(t.name, { exact: false }).first().isVisible({ timeout: 1000 }).catch(() => false)
    if (visible) throw new Error(`expected NOT to see "${step.notSee}"`)
    return
  }
  if ('click' in step) {
    const t = parseTarget(step.click)
    if (t.role) {
      await page.getByRole(t.role, { name: t.name }).first().click()
    } else {
      await page.getByRole('button', { name: t.name }).first().click().catch(async () => {
        await page.getByText(t.name, { exact: false }).first().click()
      })
    }
    return
  }
  if ('fill' in step) {
    await page.getByLabel(step.fill.field).first().fill(step.fill.value)
    return
  }
  if ('choose' in step) {
    await page.getByText(step.choose.option, { exact: true }).first().click()
    return
  }
}

function parseTarget(s: string): { role?: string; name: string } {
  const m = s.match(/^role:(\S+)\s+name:(.+)$/i)
  if (m) return { role: m[1], name: m[2]!.trim() }
  return { name: s }
}
