// The browser half of the acceptance proof.
//
// `test/timeline-acceptance.test.ts` proves `timeline()` agrees with an
// independent oracle. This proves it agrees with the ENGINE: the same
// captured keyframes are handed to real WAAPI in headless Chromium, and at
// every sample point the browser's own computed transform for the WAAPI
// element is compared to the computed transform of an element whose
// transform is composed from the custom properties `timeline()` drives.
//
// Both sides are read with `getComputedStyle().transform`, so the comparison
// is between two 2D matrices Chromium resolved itself. Nothing in this file
// interpolates anything — if `timeline()` were subtly wrong about an offset,
// a hold, or a bezier, the matrices would diverge and no shared bug could
// hide it.
//
//   bunx playwright test        (from packages/sail/runtime)

import { test, expect } from '@playwright/test'
import { execFileSync } from 'node:child_process'
import { readFileSync, mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))

const CAPTURE = JSON.parse(
  readFileSync(
    resolve(HERE, '../test/fixtures/motion-capture/waapi-animations.json'),
    'utf8',
  ),
) as Array<{
  target: string
  duration: number
  keyframes: Array<{ offset: number; easing: string; transform: string }>
}>

const TILE = CAPTURE.find((a) => a.target === 'span.home-diag-tile')!

/** Build the real runtime for the browser. Fails loudly rather than mocking. */
function buildBundle(): string {
  const out = join(mkdtempSync(join(tmpdir(), 'sail-timeline-e2e-')), 'bundle.js')
  execFileSync(
    'bun',
    ['build', resolve(HERE, 'browser-entry.ts'), '--target=browser', '--outfile', out],
    { stdio: 'pipe' },
  )
  return readFileSync(out, 'utf8')
}

const BUNDLE = buildBundle()

const PAGE = `
<style>
  body { margin: 0 }
  span { display: block; width: 40px; height: 40px; background: #333 }
  /* The sail element's transform is composed ENTIRELY from custom
     properties. No JS touches its style attribute. */
  #sail {
    transform: translate(var(--tile-x)) rotate(var(--tile-rotate)) scale(var(--tile-scale));
  }
</style>
<span id="truth"></span>
<span id="sail"></span>
`

/** Sample points: the captured stops, plus a dense sweep between them. */
const SAMPLES = [
  ...TILE.keyframes.map((k) => k.offset),
  ...Array.from({ length: 121 }, (_, i) => i / 120),
].sort((a, b) => a - b)

test.describe('timeline() vs real WAAPI — motion.dev tile', () => {
  test('computed transforms agree at every sample point', async ({ page }) => {
    await page.setContent(PAGE)
    await page.addScriptTag({ content: BUNDLE })

    const result = await page.evaluate(
      ({ tile, samples }) => {
        const truth = document.getElementById('truth')!
        const sailEl = document.getElementById('sail')!

        // Left side: the capture, replayed through the browser's own
        // animation engine, paused and seeked by hand.
        const anim = truth.animate(
          tile.keyframes.map((k) => ({
            offset: k.offset,
            easing: k.easing,
            transform: k.transform,
          })),
          { duration: tile.duration, fill: 'both' },
        )
        anim.pause()

        // Right side: timeline(), reflected to custom properties exactly the
        // way the compiler emits it for `sx="translate-x-(--tile-x)"`.
        const { signal, effect, timeline } = window.sail
        const parse = (t: string, fn: string) =>
          Number(new RegExp(`${fn}\\(([-\\d.]+)`).exec(t)![1])

        const stops = tile.keyframes.map((k) => ({
          offset: k.offset,
          x: parse(k.transform, 'translate'),
          rotate: parse(k.transform, 'rotate'),
          scale: parse(k.transform, 'scale'),
        }))

        const progress = signal(0)
        const t = timeline(
          progress,
          {
            tileX: stops.map((s) => [s.offset, `${s.x}px`] as const),
            tileRotate: stops.map((s) => [s.offset, `${s.rotate}deg`] as const),
            tileScale: stops.map((s) => [s.offset, s.scale] as const),
          },
          { easing: [0.6, 0, 0.2, 1] },
        )
        effect(() => sailEl.style.setProperty('--tile-x', String(t.tileX())))
        effect(() => sailEl.style.setProperty('--tile-rotate', String(t.tileRotate())))
        effect(() => sailEl.style.setProperty('--tile-scale', String(t.tileScale())))

        const readMatrix = (el: Element): number[] => {
          const s = getComputedStyle(el).transform
          if (s === 'none') return [1, 0, 0, 1, 0, 0]
          return /matrix\(([^)]+)\)/.exec(s)![1]!.split(',').map((n) => Number(n.trim()))
        }

        const rows: Array<{ p: number; truth: number[]; sail: number[] }> = []
        for (const p of samples) {
          anim.currentTime = p * tile.duration
          progress.set(p)
          rows.push({ p, truth: readMatrix(truth), sail: readMatrix(sailEl) })
        }
        return rows
      },
      { tile: TILE, samples: SAMPLES },
    )

    expect(result.length).toBe(SAMPLES.length)

    let worst = 0
    for (const row of result) {
      for (let i = 0; i < 6; i++) {
        // Matrix entries a–d are unitless (scale/rotate), e–f are pixels.
        // The bound is set by the translate channel: two bezier solvers
        // disagreeing by 3.66e-4 of eased progress move a 120px translate by
        // 0.044px, and Chromium serialises computed transforms at limited
        // precision on top of that. Observed worst case here is 0.042; 0.1
        // leaves room for solver drift across Chromium versions while still
        // being orders of magnitude tighter than any real interpolation bug
        // (a missed offset moves the tile by whole pixels).
        const delta = Math.abs(row.truth[i]! - row.sail[i]!)
        worst = Math.max(worst, delta)
        expect(
          delta,
          `matrix[${i}] at progress ${row.p}: WAAPI ${row.truth[i]} vs timeline ${row.sail[i]}`,
        ).toBeLessThan(0.1)
      }
    }
    // Surfaced so a slow drift shows up in the log before it fails.
    console.log(`worst matrix component delta across ${result.length} samples: ${worst}`)
  })

  test('the tile actually moves in the browser — not a static element', async ({ page }) => {
    await page.setContent(PAGE)
    await page.addScriptTag({ content: BUNDLE })

    const spread = await page.evaluate(
      ({ tile }) => {
        const el = document.getElementById('sail')!
        const { signal, effect, timeline } = window.sail
        const parse = (t: string, fn: string) =>
          Number(new RegExp(`${fn}\\(([-\\d.]+)`).exec(t)![1])
        const stops = tile.keyframes.map((k) => ({
          offset: k.offset,
          x: parse(k.transform, 'translate'),
          rotate: parse(k.transform, 'rotate'),
          scale: parse(k.transform, 'scale'),
        }))
        const progress = signal(0)
        const t = timeline(
          progress,
          {
            tileX: stops.map((s) => [s.offset, `${s.x}px`] as const),
            tileRotate: stops.map((s) => [s.offset, `${s.rotate}deg`] as const),
            tileScale: stops.map((s) => [s.offset, s.scale] as const),
          },
          { easing: [0.6, 0, 0.2, 1] },
        )
        effect(() => el.style.setProperty('--tile-x', String(t.tileX())))
        effect(() => el.style.setProperty('--tile-rotate', String(t.tileRotate())))
        effect(() => el.style.setProperty('--tile-scale', String(t.tileScale())))

        // Read the element's real, laid-out bounding box — the thing a user
        // would actually see move. An animation that "runs" without moving
        // anything produces one box, forever.
        const boxes = new Set<string>()
        for (let i = 0; i <= 400; i++) {
          progress.set(i / 400)
          const r = el.getBoundingClientRect()
          boxes.add(`${r.left.toFixed(2)}/${r.width.toFixed(2)}`)
        }
        const lefts: number[] = []
        for (let i = 0; i <= 400; i++) {
          progress.set(i / 400)
          lefts.push(el.getBoundingClientRect().left)
        }
        return { distinctBoxes: boxes.size, minLeft: Math.min(...lefts), maxLeft: Math.max(...lefts) }
      },
      { tile: TILE },
    )

    expect(spread.distinctBoxes).toBeGreaterThan(100)
    // 120px of travel, plus the scale pop widening the box either side.
    expect(spread.maxLeft - spread.minLeft).toBeGreaterThan(100)
  })

  test('a clock()-driven tile advances on its own at ~60fps', async ({ page }) => {
    await page.setContent(PAGE)
    await page.addScriptTag({ content: BUNDLE })

    const perf = await page.evaluate(
      ({ tile }) => {
        const el = document.getElementById('sail')!
        const { effect, timeline, clock } = window.sail
        const parse = (t: string, fn: string) =>
          Number(new RegExp(`${fn}\\(([-\\d.]+)`).exec(t)![1])
        const stops = tile.keyframes.map((k) => ({
          offset: k.offset,
          x: parse(k.transform, 'translate'),
          rotate: parse(k.transform, 'rotate'),
          scale: parse(k.transform, 'scale'),
        }))

        // The acceptance shape verbatim: 3600ms, driven by clock().
        const p = clock(tile.duration, { reducedMotion: 'play' })
        const t = timeline(
          p,
          {
            tileX: stops.map((s) => [s.offset, `${s.x}px`] as const),
            tileRotate: stops.map((s) => [s.offset, `${s.rotate}deg`] as const),
            tileScale: stops.map((s) => [s.offset, s.scale] as const),
          },
          { easing: [0.6, 0, 0.2, 1] },
        )
        effect(() => el.style.setProperty('--tile-x', String(t.tileX())))
        effect(() => el.style.setProperty('--tile-rotate', String(t.tileRotate())))
        effect(() => el.style.setProperty('--tile-scale', String(t.tileScale())))

        return new Promise<{ frames: number; ms: number; progress: number; moved: boolean }>(
          (done) => {
            const t0 = performance.now()
            const start = el.getBoundingClientRect().left
            let frames = 0
            let moved = false
            const step = () => {
              frames++
              if (Math.abs(el.getBoundingClientRect().left - start) > 1) moved = true
              if (performance.now() - t0 < 1000) requestAnimationFrame(step)
              else done({ frames, ms: performance.now() - t0, progress: p(), moved })
            }
            requestAnimationFrame(step)
          },
        )
      },
      { tile: TILE },
    )

    const fps = (perf.frames / perf.ms) * 1000
    console.log(
      `clock()-driven tile: ${perf.frames} frames in ${perf.ms.toFixed(0)}ms ` +
        `(${fps.toFixed(1)}fps), progress reached ${perf.progress.toFixed(3)}`,
    )

    // The clock advanced roughly a second's worth of a 3600ms run…
    expect(perf.progress).toBeGreaterThan(0.15)
    expect(perf.progress).toBeLessThan(0.45)
    // …the element actually moved while it did…
    expect(perf.moved).toBe(true)
    // …and per-frame custom-property writes did not cost us the frame rate.
    expect(fps).toBeGreaterThan(50)
  })
})

// ── Perf: the two-lane guidance, measured ────────────────────────────────────
//
// `timeline()` writes custom properties from JS every frame, which means
// main-thread style recalc — the cost `fx:` avoids by handing WAAPI
// animations to the compositor. That trade is the whole reason the two lanes
// exist, so the number that matters is: how many choreographed elements
// before the main thread stops keeping up? These measure it rather than
// asserting it from theory.

test.describe('timeline() perf — the choreography lane on the main thread', () => {
  for (const count of [1, 12, 60]) {
    test(`${count} element(s) × 3 channels holds 60fps`, async ({ page }) => {
      await page.setContent(`
        <style>
          body { margin: 0 }
          span {
            display: block; width: 24px; height: 24px; background: #333;
            transform: translate(var(--x)) rotate(var(--r)) scale(var(--s));
          }
        </style>
      `)
      await page.addScriptTag({ content: BUNDLE })

      const perf = await page.evaluate(
        ({ tile, count }) => {
          const { effect, timeline, clock } = window.sail
          const parse = (t: string, fn: string) =>
            Number(new RegExp(`${fn}\\(([-\\d.]+)`).exec(t)![1])
          const stops = tile.keyframes.map((k) => ({
            offset: k.offset,
            x: parse(k.transform, 'translate'),
            rotate: parse(k.transform, 'rotate'),
            scale: parse(k.transform, 'scale'),
          }))

          // ONE clock, many elements — the real shape of choreography.
          const p = clock(tile.duration, { loop: true, reducedMotion: 'play' })
          for (let i = 0; i < count; i++) {
            const el = document.createElement('span')
            document.body.appendChild(el)
            const t = timeline(
              p,
              {
                x: stops.map((s) => [s.offset, `${s.x}px`] as const),
                r: stops.map((s) => [s.offset, `${s.rotate}deg`] as const),
                s: stops.map((s) => [s.offset, s.scale] as const),
              },
              { easing: [0.6, 0, 0.2, 1] },
            )
            effect(() => el.style.setProperty('--x', String(t.x())))
            effect(() => el.style.setProperty('--r', String(t.r())))
            effect(() => el.style.setProperty('--s', String(t.s())))
          }

          return new Promise<{ frames: number; ms: number; worstGap: number }>((done) => {
            const t0 = performance.now()
            let last = t0
            let frames = 0
            let worstGap = 0
            const step = () => {
              const now = performance.now()
              if (frames > 2) worstGap = Math.max(worstGap, now - last)
              last = now
              frames++
              if (now - t0 < 1500) requestAnimationFrame(step)
              else done({ frames, ms: now - t0, worstGap })
            }
            requestAnimationFrame(step)
          })
        },
        { tile: TILE, count },
      )

      const fps = (perf.frames / perf.ms) * 1000
      console.log(
        `PERF ${String(count).padStart(2)} elements (${count * 3} channels): ` +
          `${fps.toFixed(1)}fps, worst frame gap ${perf.worstGap.toFixed(1)}ms`,
      )
      expect(fps).toBeGreaterThan(50)
      // Worst frame gap is REPORTED, not gated. Under headless Chromium a
      // single GC pause or compositor hiccup lands a 20–45ms frame at ANY
      // element count (we see it at 1 element as readily as at 60), so
      // asserting on it would gate the suite on the harness's noise rather
      // than on `timeline()`. Sustained average fps is the stable signal and
      // is what's asserted; the gap is logged so a real regression — a worst
      // gap that climbs WITH element count across runs — is still visible.
    })
  }
})

// ── The other two gauntlet cases ─────────────────────────────────────────────

test.describe('the ball and the spin — visible, in a real browser', () => {
  test('springValue drops the ball with real physics and it settles', async ({ page }) => {
    await page.setContent(`
      <style>
        body { margin: 0; height: 600px }
        #ball {
          position: absolute; top: 0; left: 20px;
          width: 32px; height: 32px; border-radius: 50%; background: #333;
          transform: translateY(var(--ball-y));
        }
      </style>
      <div id="ball"></div>
    `)
    await page.addScriptTag({ content: BUNDLE })

    const drop = await page.evaluate(() => {
      const el = document.getElementById('ball')!
      const { signal, effect, springValue } = window.sail
      const target = signal(0)
      // A bouncy drop: low damping so it overshoots the floor and comes back.
      const y = springValue(target, { stiffness: 260, damping: 12 })
      effect(() => el.style.setProperty('--ball-y', `${y()}px`))

      return new Promise<{ peak: number; settled: number; samples: number; distinct: number }>(
        (done) => {
          target.set(400)
          const seen = new Set<number>()
          let peak = 0
          const t0 = performance.now()
          const step = () => {
            const top = el.getBoundingClientRect().top
            seen.add(Math.round(top * 10))
            peak = Math.max(peak, top)
            if (performance.now() - t0 < 2500) requestAnimationFrame(step)
            else
              done({
                peak,
                settled: el.getBoundingClientRect().top,
                samples: seen.size,
                distinct: seen.size,
              })
          }
          requestAnimationFrame(step)
        },
      )
    })

    console.log(
      `ball: peak ${drop.peak.toFixed(1)}px, settled ${drop.settled.toFixed(1)}px, ` +
        `${drop.distinct} distinct positions`,
    )
    // It got there…
    expect(drop.settled).toBeGreaterThan(395)
    expect(drop.settled).toBeLessThan(405)
    // …it OVERSHOT on the way (a tween cannot do this)…
    expect(drop.peak).toBeGreaterThan(410)
    // …and it was visible doing it, not a single jump.
    expect(drop.distinct).toBeGreaterThan(30)
  })

  test('a spin runs at a visible rate — not a stuck or invisible loop', async ({ page }) => {
    await page.setContent(`
      <style>
        body { margin: 0 }
        #spin {
          width: 40px; height: 40px; background: #333;
          transform: rotate(var(--spin));
        }
      </style>
      <div id="spin"></div>
    `)
    await page.addScriptTag({ content: BUNDLE })

    const spin = await page.evaluate(() => {
      const el = document.getElementById('spin')!
      const { effect, timeline, clock } = window.sail
      // One full turn per second, forever.
      const p = clock(1000, { loop: true, reducedMotion: 'play' })
      const { spin } = timeline(p, { spin: [[0, '0deg'], [1, '360deg']] })
      effect(() => el.style.setProperty('--spin', String(spin())))

      return new Promise<{ angles: number[]; distinct: number }>((done) => {
        const angles: number[] = []
        const t0 = performance.now()
        const step = () => {
          const m = getComputedStyle(el).transform
          const parts = /matrix\(([^)]+)\)/.exec(m)
          if (parts) {
            const [a, b] = parts[1]!.split(',').map(Number)
            angles.push((Math.atan2(b!, a!) * 180) / Math.PI)
          }
          if (performance.now() - t0 < 1000) requestAnimationFrame(step)
          else done({ angles, distinct: new Set(angles.map((x) => Math.round(x))).size })
        }
        requestAnimationFrame(step)
      })
    })

    console.log(`spin: ${spin.angles.length} frames, ${spin.distinct} distinct whole degrees`)
    // A full turn in a second sampled at ~60fps gives ~60 frames, and a
    // genuinely spinning element lands on a DIFFERENT whole degree on
    // essentially every one of them (360° / 60 frames = 6° per frame). A
    // "running but invisible" loop collapses to one angle; a stuttering one
    // repeats angles. Both fail here.
    expect(spin.angles.length).toBeGreaterThan(50)
    expect(spin.distinct).toBeGreaterThan(spin.angles.length * 0.9)
    expect(Math.max(...spin.angles)).toBeGreaterThan(150)
    expect(Math.min(...spin.angles)).toBeLessThan(-150)
  })
})
