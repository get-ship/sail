// L1 visual-conformance harness — the ONE reusable render+probe engine.
//
// Every L1 element gets a spec (states + probe). The harness renders the state
// matrix across ALL 7 themes, screenshots it to a TRACKED dir (the audit trail
// — committed, diffable over time), and runs the spec's probe in-page to ASSERT
// invariants (the gate: non-zero `bad` fails CI). Replaces the throwaway
// per-component `*-proof.mjs` scripts with one home that leaves a trail.
//
// Render method (v1): inject the real framework CSS (tokens/themes/contracts/
// element-skins) and render each element's lowered-output shape across the theme
// grid — faithful for mechanism-1 (compiles-away) and mechanism-2 (native+skin)
// L1, whose lowered output IS static HTML + that CSS. Mechanism-3 composites
// (light-DOM components + controllers) need real mounting — a second render
// mode tracked as follow-up, not a second framework.

import { chromium } from 'playwright'
import { readFileSync, mkdirSync } from 'node:fs'
import { execFileSync } from 'node:child_process'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const TOK = resolve(HERE, '../tokens/src')
const FRAMEWORK_CSS = ['tokens.css', 'themes.css', 'sail-base.css', 'element-skins.css']
  .map((f) => readFileSync(resolve(TOK, f), 'utf8'))
  .join('\n')

// The 7 canonical themes. `.dark` is toggled for the three dark themes so the
// `.dark`-scoped token overrides resolve (the fidelity bug earlier harnesses hit).
export const THEMES = ['ship-light', 'ship-dark', 'paper', 'halo', 'console', 'contrast', 'glass']
const DARK = new Set(['ship-dark', 'console', 'contrast'])

const SHELL_CSS = `
html,body{margin:0}body{font-family:system-ui}
section{padding:26px 40px;border-bottom:1px solid color-mix(in srgb,var(--foreground) 12%,transparent);background:var(--background);color:var(--foreground)}
.l1-theme{font:600 11px ui-monospace,monospace;letter-spacing:.08em;text-transform:uppercase;opacity:.55;margin-bottom:18px}
.l1-row{display:flex;gap:10px;align-items:center;flex-wrap:wrap;margin-bottom:10px}
`

/**
 * Render + probe one spec across all themes.
 * @param spec {{ tag:string, width?:number, rows:()=>string[], probe:()=>{count:number,bad:any[]} }}
 * @param outDir absolute dir for the tracked screenshot
 * @returns {{ count:number, bad:any[] }}
 */
// ONE browser for the whole run. Launching per spec (2× per spec: render +
// behavior) meant ~76 launches on a full suite — under load a launch times out
// and takes specs down for reasons that have nothing to do with the component.
let BROWSER = null
async function browser() {
  if (!BROWSER) BROWSER = await chromium.launch()
  return BROWSER
}
export async function closeBrowser() {
  if (BROWSER) {
    await BROWSER.close()
    BROWSER = null
  }
}

export async function renderSpec(spec, outDir) {
  const rows = spec.rows().map((r) => `<div class="l1-row">${r}</div>`).join('')
  const section = (t) =>
    `<section data-theme-preview="${t}" class="${DARK.has(t) ? 'dark' : ''}"><div class="l1-theme">${t}</div>${rows}</section>`
  const extra = spec.css ? `\n${spec.css}` : ''
  const html = `<!doctype html><html><head><meta charset="utf8"><style>${FRAMEWORK_CSS}\n${SHELL_CSS}${extra}</style></head><body>${THEMES.map(section).join('')}</body></html>`

  const b = await browser()
  const ctx = await b.newContext({ viewport: { width: spec.width ?? 760, height: 900 }, deviceScaleFactor: 2 })
  try {
    const p = await ctx.newPage()
    await p.setContent(html, { waitUntil: 'load' })
    // Optional in-page setup before probe + screenshot — e.g. a native [popover]
    // is display:none until shown, so overlay specs open theirs here.
    if (spec.setup) {
      await p.evaluate(spec.setup)
      await p.waitForTimeout(60)
    }
    await p.waitForTimeout(120)
    const result = await p.evaluate(spec.probe)
    mkdirSync(outDir, { recursive: true })
    await p.screenshot({ path: resolve(outDir, `${spec.tag}.png`), fullPage: true })
    return result
  } finally {
    await ctx.close()
  }
}

// ── BEHAVIOR mode ────────────────────────────────────────────────────────────
// The probe above asserts RENDER + computed styles + ARIA state across themes.
// This asserts the `use:` action's live BEHAVIOR: bundle the real actions, attach
// them to the lowered markup, and drive genuine interaction (click / keyboard)
// through Playwright. One theme only — selectors must be unambiguous.

let ACTIONS_JS = null
function actionsBundle() {
  if (ACTIONS_JS) return ACTIONS_JS
  const out = resolve(HERE, '.actions.bundle.js')
  execFileSync(
    'bun',
    ['build', resolve(HERE, 'behavior-entry.ts'), '--outfile', out, '--format=iife', '--target=browser'],
    { stdio: 'pipe' },
  )
  ACTIONS_JS = readFileSync(out, 'utf8')
  return ACTIONS_JS
}

/**
 * Drive a spec's `use:` actions with real interaction.
 * @param spec {{ tag:string, width?:number, rows:()=>string[],
 *                behavior?:{ attach:[string,string][], run:(page)=>Promise<any[]> } }}
 * @returns {Promise<any[]>} issues (empty = pass); [] when the spec has no behavior
 */
export async function behaveSpec(spec) {
  if (!spec.behavior) return []
  // Overlays render their SURFACE in the theme grid (a native [popover] is
  // top-layer + display:none, so it can't live in a 7-up grid) but need the real
  // popover + trigger to be DRIVEN — so behavior may supply its own markup.
  const rowSrc = spec.behavior.rows ?? spec.rows
  const rows = rowSrc().map((r) => `<div class="l1-row">${r}</div>`).join('')
  const extra = spec.css ? `\n${spec.css}` : ''
  const html = `<!doctype html><html><head><meta charset="utf8"><style>${FRAMEWORK_CSS}\n${SHELL_CSS}${extra}</style></head><body><section>${rows}</section></body></html>`

  const b = await browser()
  const ctx = await b.newContext({ viewport: { width: spec.width ?? 760, height: 900 } })
  try {
    const p = await ctx.newPage()
    await p.setContent(html, { waitUntil: 'load' })
    await p.addScriptTag({ content: actionsBundle() })
    await p.evaluate((attach) => {
      for (const [sel, name] of attach) {
        const fn = globalThis.SuiActions?.[name]
        if (typeof fn !== 'function') throw new Error(`no such use: action → ${name}`)
        for (const el of document.querySelectorAll(sel)) fn(el)
      }
    }, spec.behavior.attach)
    await p.waitForTimeout(60)
    // A throwing behavior test is a FAILED spec, not a dead run — report it.
    try {
      return await spec.behavior.run(p)
    } catch (err) {
      return [{ why: 'behavior-threw', error: String(err).split('\n')[0] }]
    }
  } finally {
    await ctx.close()
  }
}
