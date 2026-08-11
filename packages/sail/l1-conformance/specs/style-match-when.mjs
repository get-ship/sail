// SPEC-TESTING-1 — style `match:` / `when:` (+ structural `<when>`) rendered proof
// (spike S6.2). SCOUT-S6 File B / job-spec-spike-s6-2-match-when.
//
// Doctrine (l1-conformance model): compile a real .sail fixture with live
// sail-compiler → mount lowered markup + emitted stylesheet under the theme
// grid → assert getComputedStyle / exclusive layout. Screenshots are audit
// trail only. Emit-string pins of CSS/JS are NOT the gate.
//
// ── CONTRACT (implementer must honor; do not weaken this file) ─────────────
// Fixture path (implementer-owned; missing → RED):
//   packages/sail/l1-conformance/fixtures/style-match-when.sail
//
// The fixture MUST exercise all three axes (see RECOMMENDED_FIXTURE below):
//   1. style match on `variant` — solid vs soft with distinct paint
//      (bg primary / muted + matching fg tokens)
//   2. style when hover — opacity changes under :hover on [data-t=hover-host]
//   3. structural <when {tablet}> — exclusive arms:
//        then → [data-t=wide]    else → [data-t=narrow]
//
// Required markers after compile + mount:
//   [data-t="match-solid"][variant="solid"]
//   [data-t="match-soft"][variant="soft"]
//   [data-t="hover-host"]
//   [data-t="wide"] | [data-t="narrow"]  (exactly one mounted per MQ state)
//
// Assertions (executed paint/layout — not emit text):
//   A. solid backgroundColor !== soft backgroundColor; neither transparent
//   B. Playwright hover on hover-host → computed opacity differs from rest
//   C. matchMedia tablet false → only narrow in document; true → only wide
//      (real bindWhen branch swap — not dual-hidden CSS)
//
// Forbidden:
//   - Snapshot of emitted JS/CSS as the sole gate
//   - mockQueryState-style substrate injection
//   - Hand-typed utility CSS that fakes match arms without compiling Style
//
// Until fixtures/style-match-when.sail lands, this gate fails RED.
//
//   node run.mjs style-match-when   (from packages/sail/l1-conformance)
//   # requires target/debug/sail-compiler when the fixture is present

import { execFileSync } from 'node:child_process'
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const ROOT = resolve(HERE, '../../../..')
const FIXTURE = resolve(HERE, '../fixtures/style-match-when.sail')
const COMPILER = resolve(ROOT, 'target/debug/sail-compiler')
const TMP = resolve(HERE, '../.style-match-when-fixture')

/**
 * Documented fixture body for the implementer. Copy into
 * `fixtures/style-match-when.sail` (or an equivalent that preserves markers +
 * Style match/when + structural when). Not auto-written — absence keeps RED.
 */
export const RECOMMENDED_FIXTURE = `<div data-t="match-solid" variant="solid" style({
  match: {
    variant: {
      solid: { paint: bg(primary).fg(primary-foreground) },
      soft:  { paint: bg(muted).fg(muted-foreground) },
    }
  },
  when: { hover: { paint: opacity(50) } }
})>solid</div>
<div data-t="match-soft" variant="soft" style({
  match: {
    variant: {
      solid: { paint: bg(primary).fg(primary-foreground) },
      soft:  { paint: bg(muted).fg(muted-foreground) },
    }
  },
  when: { hover: { paint: opacity(50) } }
})>soft</div>
<div data-t="hover-host" style({
  paint: bg(primary).opacity(100),
  when: { hover: { paint: opacity(50) } }
})>hover</div>
<div data-t="when-wrap">
  <when {tablet}>
    <div data-t="wide" style({ box: w(full) })>wide</div>
    {else => <div data-t="narrow" style({ box: w(full) })>narrow</div>}
  </when>
</div>
`

/** @type {{ ok:true, css:string, rows:string[], query:string, wideHtml:string, narrowHtml:string } | { ok:false, why:string, detail?:string }} */
function compileSurface() {
  if (!existsSync(FIXTURE)) {
    return {
      ok: false,
      why: 'missing-fixture',
      detail: `Implementer must add ${FIXTURE} (see RECOMMENDED_FIXTURE / CONTRACT in style-match-when.mjs)`,
    }
  }
  if (!existsSync(COMPILER)) {
    return {
      ok: false,
      why: 'missing-compiler',
      detail: `Build sail-compiler → ${COMPILER}`,
    }
  }

  const src = readFileSync(FIXTURE, 'utf8')
  if (!src.includes('match:') || !/when\s*:/.test(src) || !src.includes('<when')) {
    return {
      ok: false,
      why: 'fixture-incomplete',
      detail: 'Fixture must include style match:, style when:, and structural <when>',
    }
  }
  for (const marker of ['match-solid', 'match-soft', 'hover-host', 'wide', 'narrow']) {
    if (!src.includes(`data-t="${marker}"`) && !src.includes(`data-t='${marker}'`)) {
      return {
        ok: false,
        why: 'fixture-missing-marker',
        detail: `Fixture must mark data-t="${marker}"`,
      }
    }
  }

  mkdirSync(TMP, { recursive: true })
  // Compile the implementer fixture as-is (route mode — no <component> wrapper).
  const sailPath = resolve(TMP, 'fixture.sail')
  writeFileSync(sailPath, src)
  try {
    execFileSync(COMPILER, ['build', sailPath, '--out', TMP], {
      cwd: ROOT,
      stdio: 'pipe',
    })
  } catch (err) {
    const stderr = err?.stderr?.toString?.() || String(err).split('\n')[0]
    return { ok: false, why: 'compile-failed', detail: stderr.slice(0, 400) }
  }

  const jsPath = resolve(TMP, 'fixture.js')
  if (!existsSync(jsPath)) {
    return { ok: false, why: 'compile-no-js', detail: jsPath }
  }
  const js = readFileSync(jsPath, 'utf8')

  // Extract stylesheet the compiler embeds for Style (inside __mount or top-level).
  const styleM = js.match(/const __suiStyles = ("(?:[^"\\]|\\.)*")/)
  if (!styleM) {
    return {
      ok: false,
      why: 'no-sui-styles',
      detail: 'compiled fixture has no __suiStyles — Style emit path missing',
    }
  }
  const css = JSON.parse(styleM[1])

  const tpls = [...js.matchAll(/__tpl\("((?:[^"\\]|\\.)*)"\)/g)].map((m) =>
    JSON.parse(`"${m[1]}"`),
  )
  const pick = (t) => tpls.find((h) => h.includes(`data-t="${t}"`))
  const solid = pick('match-solid')
  const soft = pick('match-soft')
  const hover = pick('hover-host')
  const wideHtml = pick('wide')
  const narrowHtml = pick('narrow')
  if (!solid || !soft || !hover || !wideHtml || !narrowHtml) {
    return {
      ok: false,
      why: 'tpl-missing-markers',
      detail: `tpls=${tpls.length}; need match-solid/soft, hover-host, wide, narrow`,
    }
  }

  // Structural when query must come from the compiler emit (tablet → min-width).
  if (!js.includes('bindWhen')) {
    return {
      ok: false,
      why: 'no-bindWhen-emit',
      detail: 'structural <when> must lower to bindWhen',
    }
  }
  const queryM = js.match(/query:\s*("(?:[^"\\]|\\.)*")/)
  if (!queryM) {
    return { ok: false, why: 'no-when-query', detail: 'bindWhen query string missing from emit' }
  }
  const query = JSON.parse(queryM[1])

  return {
    ok: true,
    css,
    rows: [solid, soft, hover],
    query,
    wideHtml,
    narrowHtml,
  }
}

const SURFACE = compileSurface()

/** Bundle real bindWhen for structural-when behavior (not a hand-rolled swap). */
function whenRuntimeBundle() {
  const dir = mkdtempSync(join(tmpdir(), 'l1-style-when-'))
  const entry = join(dir, 'when-entry.ts')
  const out = join(dir, 'when.bundle.js')
  const whenTs = resolve(ROOT, 'packages/sail/runtime/src/when.ts').replace(/\\/g, '/')
  writeFileSync(
    entry,
    `import { bindWhen, __resetMediaQueryCacheForTests } from ${JSON.stringify(whenTs)}
;(globalThis as unknown as { SailWhen: unknown }).SailWhen = {
  bindWhen,
  __resetMediaQueryCacheForTests,
}
`,
  )
  execFileSync('bun', ['build', entry, '--outfile', out, '--format=iife', '--target=browser'], {
    stdio: 'pipe',
  })
  return readFileSync(out, 'utf8')
}

export default {
  tag: 'style-match-when',
  width: 760,
  css: SURFACE.ok ? SURFACE.css : '',
  rows() {
    if (!SURFACE.ok) {
      return [
        `<div data-t="gate-missing" data-why="${SURFACE.why}">${SURFACE.detail || SURFACE.why}</div>`,
      ]
    }
    // Lowered hosts from the compiler (__tpl) — not hand-typed class lists.
    return SURFACE.rows
  },
  probe() {
    const bad = []
    const missing = document.querySelector('[data-t="gate-missing"]')
    if (missing) {
      return {
        count: 0,
        bad: [
          {
            why: missing.getAttribute('data-why') || 'gate-missing',
            detail: missing.textContent || '',
          },
        ],
      }
    }

    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    // One theme is enough for language proof; ship is the default catalog light.
    const scope = "[theme='ship'] "
    const q = (t) => document.querySelector(`${scope}[data-t='${t}']`)

    const solid = q('match-solid')
    const soft = q('match-soft')
    if (!solid || !soft) {
      bad.push({ why: 'match-hosts-missing', solid: !!solid, soft: !!soft })
      return { count: 0, bad }
    }

    const solidBg = getComputedStyle(solid).backgroundColor
    const softBg = getComputedStyle(soft).backgroundColor
    const solidFg = getComputedStyle(solid).color
    const softFg = getComputedStyle(soft).color

    // A. Active match arms must paint differently (attribute-gated Style match).
    if (clear(solidBg)) bad.push({ why: 'solid-arm-transparent-bg', solidBg })
    if (clear(softBg)) bad.push({ why: 'soft-arm-transparent-bg', softBg })
    if (solidBg === softBg) {
      bad.push({ why: 'match-arms-same-paint', solidBg, softBg })
    }
    // Soft must not accidentally take solid's primary fill (wrong selector).
    if (!clear(solidBg) && !clear(softBg) && solidBg === softBg) {
      /* already pushed */
    }
    if (clear(solidFg) || clear(softFg)) {
      bad.push({ why: 'match-arm-no-fg', solidFg, softFg })
    }

    // Hover host must exist for behavior mode (opacity channel lives there).
    if (!q('hover-host')) bad.push({ why: 'hover-host-missing' })

    return {
      count: document.querySelectorAll(`${scope}[data-t]`).length,
      bad: bad.slice(0, 12),
    }
  },
  // Behavior only when the fixture compiled — keeps the RED (missing-fixture)
  // path free of the actions/when bundle so the probe alone is the alarm.
  ...(SURFACE.ok
    ? {
        behavior: {
          rows() {
            // Static Style hosts + an anchor host for real bindWhen (structural when).
            return [
              ...SURFACE.rows,
              `<div data-t="when-host" style="min-height:24px"><!--when--></div>`,
            ]
          },
          attach: [],
          async run(page) {
            const bad = []

            // ── B. style when: hover → opacity channel changes ─────────────
            const hover = page.locator('[data-t="hover-host"]').first()
            if ((await hover.count()) === 0) {
              bad.push({ why: 'hover-host-missing-behavior' })
            } else {
              const rest = await hover.evaluate((el) => getComputedStyle(el).opacity)
              await hover.hover()
              await page.waitForTimeout(40)
              const over = await hover.evaluate((el) => getComputedStyle(el).opacity)
              if (rest === over) {
                bad.push({ why: 'hover-when-no-opacity-change', rest, over })
              }
              // Move away so later probes are not stuck in :hover.
              await page.mouse.move(0, 0)
              await page.waitForTimeout(20)
            }

            // ── C. structural <when {tablet}> via real bindWhen + MQ ───────
            // Query string + branch markup come from the compiled fixture; the
            // runtime is the real bindWhen (bundled), not a dual-hidden CSS cheat.
            let whenJs
            try {
              whenJs = whenRuntimeBundle()
            } catch (err) {
              bad.push({
                why: 'when-runtime-bundle-failed',
                error: String(err).split('\n')[0],
              })
              return bad
            }
            await page.addScriptTag({ content: whenJs })

            const { query, wideHtml, narrowHtml } = SURFACE
            const whenIssues = await page.evaluate(
              ({ query, wideHtml, narrowHtml }) => {
                const issues = []
                const SailWhen = globalThis.SailWhen
                if (!SailWhen || typeof SailWhen.bindWhen !== 'function') {
                  return [{ why: 'SailWhen.bindWhen-missing' }]
                }
                if (typeof SailWhen.__resetMediaQueryCacheForTests === 'function') {
                  SailWhen.__resetMediaQueryCacheForTests()
                }

                const host = document.querySelector('[data-t="when-host"]')
                if (!host) return [{ why: 'when-host-missing' }]

                // Controllable matchMedia: drive tablet true/false without viewport hacks.
                /** @type {Set<() => void>} */
                const listeners = new Set()
                let matches = false
                const mql = {
                  get matches() {
                    return matches
                  },
                  media: query,
                  onchange: null,
                  addEventListener(_t, cb) {
                    listeners.add(typeof cb === 'function' ? cb : cb.handleEvent)
                  },
                  removeEventListener(_t, cb) {
                    listeners.delete(typeof cb === 'function' ? cb : cb.handleEvent)
                  },
                  addListener() {},
                  removeListener() {},
                  dispatchEvent() {
                    return false
                  },
                }
                const setMatches = (v) => {
                  matches = v
                  for (const l of listeners) l()
                }
                window.matchMedia = (q) => {
                  // Only the fixture query is under test; other callers get no-match.
                  if (q === query) return mql
                  return {
                    matches: false,
                    media: q,
                    onchange: null,
                    addEventListener() {},
                    removeEventListener() {},
                    addListener() {},
                    removeListener() {},
                    dispatchEvent() {
                      return false
                    },
                  }
                }

                const parse = (html) => {
                  const t = document.createElement('template')
                  t.innerHTML = html
                  return t.content.firstElementChild
                }

                const anchor = document.createComment('when')
                host.replaceChildren(anchor)
                SailWhen.bindWhen({
                  anchor,
                  query,
                  then_: () => {
                    const el = parse(wideHtml)
                    return { nodes: el ? [el] : [] }
                  },
                  else_: () => {
                    const el = parse(narrowHtml)
                    return { nodes: el ? [el] : [] }
                  },
                })

                const snap = () => ({
                  wide: !!host.querySelector('[data-t="wide"]'),
                  narrow: !!host.querySelector('[data-t="narrow"]'),
                  text: host.textContent?.replace(/\s+/g, ' ').trim() || '',
                })

                // Default SSR/mobile-first: matches=false → else / narrow only.
                let s = snap()
                if (!(s.narrow && !s.wide)) {
                  issues.push({ why: 'when-else-not-exclusive', ...s, matches: false })
                }

                setMatches(true)
                s = snap()
                if (!(s.wide && !s.narrow)) {
                  issues.push({ why: 'when-then-not-exclusive', ...s, matches: true })
                }

                setMatches(false)
                s = snap()
                if (!(s.narrow && !s.wide)) {
                  issues.push({ why: 'when-reswap-else-failed', ...s, matches: false })
                }

                return issues
              },
              { query, wideHtml, narrowHtml },
            )
            bad.push(...whenIssues)

            return bad
          },
        },
      }
    : {}),
}
