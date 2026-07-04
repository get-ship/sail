#!/usr/bin/env bun
// primitive-adherence.ts — does a produced `.sui` COMPOSE, or hand-roll?
//
// The SUI contract (.claude/skills/sui/SKILL.md) is: compose primitives for
// layout (`<sui-grid>` / `<column>` / `<row>`) and UI (`<sui-*>`),
// spend design tokens via `var(--token)`, and NEVER hand-roll layout (ad-hoc
// `display:grid` / `display:flex`) or literal values (hex / px / raw font
// stacks) that a token already owns. A view that obeys this reads as one
// system; one that hand-rolls drifts. `sui-layout-primitive`,
// `sui-css-token`, `sui-css-literal`, and `sui-font-literal` police the same
// line at compile time — this grader turns it into a scalar so the eval can
// score the *degree* of adherence, not just pass/fail.
// ── The score ───────────────────────────────────────────────────────────────
//
//   rewards   = W_PRIM  * primitiveUses    // <sui-*> composition in the body
//             + W_TOKEN * tokenRefs         // var(--token) spends anywhere
//
//   penalties = W_LAYOUT * adHocLayout      // display:grid/flex OFF :host
//             + W_LIT    * literalValues    // hex / px / rgb|hsl / raw font
//
//   score = (rewards + penalties === 0)
//     ? 1                                   // nothing to grade → vacuously clean
//     : clamp01( 0.5 + 0.5 * (rewards - penalties) / (rewards + penalties) )
//
// The signed-balance form is symmetric and interpretable: all reward + no
// penalty → 1.0, all penalty + no reward → 0.0, an even split → 0.5. The
// weights bias the two cardinal sins — ad-hoc layout is the worst (SKILL.md:
// "If no primitive fits, the primitive is *missing*") so it costs double a
// stray literal — without any special-case branching.
//
// Detection is a text/CSS heuristic, not a full parser — same spirit as
// template-purity.ts ("not worth a full parser in a measurement script"). The
// literal-value regexes mirror grade.ts's `FORBIDDEN_IN_CSS`, extended with
// rgb/hsl and a raw `font-family` check (the `sui-font-literal` idea). The
// `:host` exemption for ad-hoc layout mirrors `sui-layout-primitive`, which
// leaves the host element's own root layout alone.

// --- weights (tunable, documented above) -----------------------------------
const W_PRIM = 1
const W_TOKEN = 1
const W_LAYOUT = 2
const W_LIT = 1

// --- detectors --------------------------------------------------------------

// A `<sui-*>` opening tag. `</sui-…>` closers can't match (`<` then `/`), so
// this counts compositions, not markup volume.
const PRIMITIVE_TAG = /<sui-[a-z][\w-]*/g
// A design-token spend. `var(--foreground)`, `var(--space-4)`, … Fallback
// forms still count as a spend here (their own smell is a separate rule).
const TOKEN_REF = /var\(\s*--[\w-]+/g

// Literal-value smells inside a `<style>` block. Mirrors grade.ts
// FORBIDDEN_IN_CSS (`#hex | rgb( | Npx`) plus hsl and standalone rgb/hsl.
const HEX = /#[0-9a-fA-F]{3,8}\b/g
const PX = /\b\d+(?:\.\d+)?px\b/g
const COLOR_FN = /\b(?:rgba?|hsla?)\s*\(/g
// Ad-hoc layout: `display: flex | grid | inline-flex | inline-grid`.
const AD_HOC_LAYOUT = /display\s*:\s*(?:inline-)?(?:flex|grid)\b/g
// A `font-family:` declaration and its value (up to `;` or block end).
const FONT_FAMILY_DECL = /font-family\s*:\s*([^;}]+)/gi

export interface AdherenceBreakdown {
  primitiveUses: number
  tokenRefs: number
  adHocLayout: number
  literalValues: number
  rewards: number
  penalties: number
  score: number
}

function clamp01(n: number): number {
  return Math.max(0, Math.min(1, n))
}

function countMatches(text: string, re: RegExp): number {
  return (text.match(re) ?? []).length
}

// Drop HTML comments so a `<!-- … -->` mention of a `<sui-*>` tag or a
// `var(--token)` never inflates (or deflates) the score.
function stripHtmlComments(sui: string): string {
  return sui.replace(/<!--[\s\S]*?-->/g, '')
}

// Pull each `<style>…</style>` block's inner CSS, comments stripped. HTML
// comments are removed first so a commented-out `<style>` doesn't count.
function styleBlocks(sui: string): string[] {
  const out: string[] = []
  const re = /<style\b[^>]*>([\s\S]*?)<\/style>/gi
  let m: RegExpExecArray | null
  while ((m = re.exec(stripHtmlComments(sui)))) {
    out.push(m[1]!.replace(/\/\*[\s\S]*?\*\//g, ''))
  }
  return out
}

// Non-nested `selector { body }` rules. Deep @media nesting isn't handled —
// same measurement-script tradeoff template-purity.ts documents; component
// `<style>` blocks are overwhelmingly flat.
function ruleBlocks(css: string): { selector: string; body: string }[] {
  const out: { selector: string; body: string }[] = []
  const re = /([^{}]*)\{([^{}]*)\}/g
  let m: RegExpExecArray | null
  while ((m = re.exec(css))) out.push({ selector: m[1]!.trim(), body: m[2]! })
  return out
}

// Ad-hoc `display:flex|grid`, excluding the component's own `:host` root
// layout (there's no `<column>` to swap the host itself with). Mirrors the
// `:host` exemption in `sui-layout-primitive`.
function adHocLayoutCount(css: string): number {
  let n = 0
  for (const { selector, body } of ruleBlocks(css)) {
    if (/:host\b/.test(selector)) continue
    n += countMatches(body, AD_HOC_LAYOUT)
  }
  return n
}

// Literal values a token should own: hex, px, rgb()/hsl(), and any
// `font-family` whose value doesn't defer to a `var(--…)` role token.
function literalValueCount(css: string): number {
  let n = countMatches(css, HEX) + countMatches(css, PX) + countMatches(css, COLOR_FN)
  let m: RegExpExecArray | null
  FONT_FAMILY_DECL.lastIndex = 0
  while ((m = FONT_FAMILY_DECL.exec(css))) {
    if (!/var\(\s*--/.test(m[1]!)) n += 1
  }
  return n
}

// Score one `.sui` source. Pure text-in — the template-purity.ts contract.
export function scorePrimitiveAdherence(suiText: string): AdherenceBreakdown {
  const scannable = stripHtmlComments(suiText)
  const primitiveUses = countMatches(scannable, PRIMITIVE_TAG)
  const tokenRefs = countMatches(scannable, TOKEN_REF)

  const css = styleBlocks(suiText).join('\n')
  const adHocLayout = adHocLayoutCount(css)
  const literalValues = literalValueCount(css)

  const rewards = W_PRIM * primitiveUses + W_TOKEN * tokenRefs
  const penalties = W_LAYOUT * adHocLayout + W_LIT * literalValues
  const total = rewards + penalties
  const score = total === 0 ? 1 : clamp01(0.5 + 0.5 * (rewards - penalties) / total)

  return { primitiveUses, tokenRefs, adHocLayout, literalValues, rewards, penalties, score }
}

export interface PerFileAdherence extends AdherenceBreakdown {
  file: string
}

export interface AdherenceReport {
  perFile: PerFileAdherence[]
  overall: number
}

// Grade a set of `.sui` sources. `overall` is the mean of the per-file
// scores (each file is one authoring unit; volume shouldn't let one huge file
// dominate a set of small ones). An empty set scores 1 — nothing to fault.
export function gradePrimitiveAdherence(files: { file: string; text: string }[]): AdherenceReport {
  const perFile = files.map(({ file, text }) => ({ file, ...scorePrimitiveAdherence(text) }))
  const overall =
    perFile.length === 0 ? 1 : perFile.reduce((s, f) => s + f.score, 0) / perFile.length
  return { perFile, overall }
}

// --- CLI (measurement, exit 0) ---------------------------------------------
// `bun eval/scripts/primitive-adherence.ts <file.sui | dir> [...]`
// Prints a per-file breakdown + an overall score, matching how the other
// eval scripts report. A directory is walked for `.sui` files.
if (import.meta.main) {
  const { readdirSync, statSync, readFileSync, existsSync } = await import('node:fs')
  const { join } = await import('node:path')

  const args = process.argv.slice(2)
  if (args.length === 0) {
    console.error('usage: primitive-adherence.ts <file.sui | dir> [...]')
    process.exit(2)
  }

  const collect = (p: string): string[] => {
    if (!existsSync(p)) return []
    if (statSync(p).isDirectory()) {
      return readdirSync(p).flatMap((e) => collect(join(p, e)))
    }
    return p.endsWith('.sui') ? [p] : []
  }

  const files = args.flatMap(collect)
  const report = gradePrimitiveAdherence(
    files.map((file) => ({ file, text: readFileSync(file, 'utf8') })),
  )

  for (const f of report.perFile) {
    console.log(
      `  ${f.score.toFixed(2)}  ${f.file}  ` +
        `(+prim=${f.primitiveUses} +tok=${f.tokenRefs} ` +
        `-layout=${f.adHocLayout} -literal=${f.literalValues})`,
    )
  }
  console.log(`overall primitive-adherence: ${report.overall.toFixed(2)} over ${report.perFile.length} file(s)`)
  process.exit(0)
}
