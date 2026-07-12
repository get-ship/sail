// L1 conformance spec: the `motion:` directive runtime. Motion cannot be
// asserted from stills or from animation bookkeeping — playState/play()
// accounting can all read "success" while every pixel sits frozen at the
// first keyframe (a zombie animation is "running", just on a timeline that
// never advances). The only trustworthy observable is COMPUTED VALUES OVER
// TIME, so every behavior gate here samples getComputedStyle per frame.
//
// Construction order matters and is part of the contract under test: the
// compiler bakes `el.animate(...)` on a <template> clone BEFORE insertion,
// while the node is still owned by the template's inert contents document.
// `Element.animate()` binds the Animation to the node document's timeline at
// call time, and an inert document's timeline never advances — an animation
// minted there holds its target at the FIRST keyframe forever (beating inline
// styles) unless the runtime re-homes it onto the live document timeline.
// Every gate below builds its element through that exact path.

// One helper string, injected into the page: clone-from-template + bake +
// attach + insert, exactly as the emit does it.
const BUILD = `
  globalThis.buildMotionEl = function buildMotionEl(id, keyframeSets, config) {
    const t = document.createElement('template')
    t.innerHTML = '<div id="' + id + '" style="width:120px;height:44px;border-radius:8px;background:rgb(30,120,220);color:#fff;font:12px system-ui;display:flex;align-items:center;justify-content:center">' + id + '</div>'
    const el = t.content.firstElementChild.cloneNode(true)
    // Bake BEFORE insertion, on the inert-document clone — the emit's order.
    const anims = keyframeSets.map(([kf, opts]) => el.animate(kf, opts))
    globalThis.SuiMotion.attach(el, { trigger: config.trigger, enter: anims, leave: anims, ...(config.stagger ? { stagger: config.stagger } : {}) })
    document.getElementById('m-host').append(el)
    return el
  }
  globalThis.sampleSeries = function sampleSeries(el, frames) {
    const out = []
    return new Promise((done) => {
      const tick = () => {
        const cs = getComputedStyle(el)
        out.push({ t: performance.now(), opacity: cs.opacity, transform: cs.transform })
        if (out.length >= frames) return done(out)
        requestAnimationFrame(tick)
      }
      requestAnimationFrame(tick)
    })
  }
`

export default {
  tag: 'motion',
  width: 720,
  // RENDER mode has no runtime — motion's visual contract lives entirely in
  // the behavior gate. The grid documents the at-rest pose: a motion-decorated
  // surface is authored-pose at rest (opacity 1, identity transform).
  rows() {
    return [
      `<div class="l1-mark" style="width:200px;padding:14px;border:1px solid var(--border);border-radius:10px;background:var(--card);color:var(--foreground);font-size:13px">motion surface — at rest</div>`,
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.l1-mark').forEach((el, i) => {
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'not-rendered' })
      if (getComputedStyle(el).opacity !== '1') bad.push({ i, why: 'rest-pose-not-authored', opacity: getComputedStyle(el).opacity })
    })
    return { count: document.querySelectorAll('.l1-mark').length, bad }
  },

  behavior: {
    rows() {
      return [`<div id="m-host" style="display:flex;gap:12px;align-items:flex-start;min-height:120px"></div>`]
    },
    attach: [],
    async run(page) {
      const bad = []
      await page.addScriptTag({ content: BUILD })

      // ── ENTER: template-baked animation must visibly RUN after insertion ──
      // Mid-transition shortly after mount (0 < opacity < 1), arrived pose
      // (opacity 1) within 2s. A run frozen at the first keyframe — the
      // inert-timeline regression — fails both.
      const enter = await page.evaluate(async () => {
        const el = buildMotionEl('m-enter', [
          [[{ offset: 0, opacity: '0' }, { offset: 1, opacity: '1' }], { duration: 400, easing: 'linear', fill: 'both' }],
          [[{ offset: 0, transform: 'translateY(16px)' }, { offset: 1, transform: 'translateY(0px)' }], { duration: 400, easing: 'linear', fill: 'both' }],
        ], { trigger: 'enter' })
        const series = await sampleSeries(el, 45)
        return { series, final: getComputedStyle(el).opacity }
      })
      const ops = enter.series.map((s) => +s.opacity)
      if (!(ops[0] < 0.5)) bad.push({ why: 'enter-did-not-start-near-departed-pose', first: ops[0] })
      if (!ops.some((o) => o > 0.05 && o < 0.95)) bad.push({ why: 'enter-never-mid-transition', ops: ops.slice(0, 10) })
      await page.waitForTimeout(1600) // total < 2s since mount
      const enterFinal = await page.evaluate(() => getComputedStyle(document.getElementById('m-enter')).opacity)
      if (enterFinal !== '1') bad.push({ why: 'enter-not-arrived-within-2s', opacity: enterFinal })

      // ── STAGGER: same-wave siblings cascade in arrival order ──
      const stagger = await page.evaluate(async () => {
        const kf = () => [[[{ offset: 0, opacity: '0' }, { offset: 1, opacity: '1' }], { duration: 250, easing: 'linear', fill: 'both' }]]
        // Both attach in ONE synchronous run — one wave, offsets 0 and 1.
        const a = buildMotionEl('m-stag-a', kf(), { trigger: 'enter', stagger: 150 })
        const b = buildMotionEl('m-stag-b', kf(), { trigger: 'enter', stagger: 150 })
        const [sa, sb] = await Promise.all([sampleSeries(a, 40), sampleSeries(b, 40)])
        const arrived = (s) => s.findIndex((x) => +x.opacity >= 1)
        return { a: arrived(sa), b: arrived(sb) }
      })
      if (stagger.a < 0 || stagger.b < 0 || !(stagger.b > stagger.a))
        bad.push({ why: 'stagger-no-cascade', arrivedFrameA: stagger.a, arrivedFrameB: stagger.b })

      // ── HOVER: computed transform leaves identity while the spring runs ──
      await page.evaluate(() => {
        buildMotionEl('m-hover', [
          [[{ offset: 0, transform: 'scale(0.9)' }, { offset: 1, transform: 'scale(1)' }], { duration: 400, easing: 'linear', fill: 'both' }],
        ], { trigger: 'hover' })
      })
      const restTransform = await page.evaluate(() => getComputedStyle(document.getElementById('m-hover')).transform)
      if (restTransform !== 'none') bad.push({ why: 'hover-rest-pose-not-authored', restTransform })
      await page.hover('#m-hover')
      const hoverSeen = await page.evaluate(async () => {
        const series = await sampleSeries(document.getElementById('m-hover'), 20)
        return series.map((s) => s.transform)
      })
      if (!hoverSeen.some((t) => t !== 'none' && t !== 'matrix(1, 0, 0, 1, 0, 0)'))
        bad.push({ why: 'hover-transform-never-changed', seen: hoverSeen.slice(0, 5) })

      // ── EXIT: departing node is HELD in the DOM with declining opacity ──
      const exit = await page.evaluate(async () => {
        const el = buildMotionEl('m-exit', [
          [[{ offset: 0, opacity: '1' }, { offset: 1, opacity: '0' }], { duration: 400, easing: 'linear', fill: 'both' }],
        ], { trigger: 'exit' })
        await new Promise((r) => requestAnimationFrame(r))
        const rest = getComputedStyle(el).opacity // parked exit must not alter the pose
        const leaving = globalThis.SuiMotion.runLeave([el])
        if (leaving === null) return { rest, why: 'runLeave-returned-null' }
        const series = await sampleSeries(el, 15)
        const midConnected = el.isConnected
        // A zombie animation's `finished` never settles — race it, or a broken
        // runtime turns this gate into a CI hang instead of a red spec.
        const settled = await Promise.race([
          leaving.then(() => true),
          new Promise((r) => setTimeout(() => r(false), 3000)),
        ])
        if (!settled) return { rest, series, midConnected, why: 'exit-never-settled' }
        el.remove() // if.ts removes after the promise; the spec plays that role
        return { rest, series, midConnected, removed: !el.isConnected }
      })
      if (exit.why) bad.push({ why: exit.why })
      else {
        if (exit.rest !== '1') bad.push({ why: 'exit-rest-pose-not-authored', rest: exit.rest })
        if (!exit.midConnected) bad.push({ why: 'exit-node-not-held-in-dom' })
        const eo = exit.series.map((s) => +s.opacity)
        if (!eo.some((o) => o > 0.05 && o < 0.95)) bad.push({ why: 'exit-never-mid-transition', eo })
        if (!(eo[eo.length - 1] < eo[0])) bad.push({ why: 'exit-opacity-not-declining', first: eo[0], last: eo[eo.length - 1] })
        if (!exit.removed) bad.push({ why: 'exit-node-not-removed-after-settle' })
      }

      return bad
    },
  },
}
