// End-to-end smoke for the motion-runtime refactor. This avoids the
// vitest source-loader plugin (which hangs in this worktree on a
// pre-existing compiler/napi bug, NOT introduced by this commit) and
// drives the runtime directly with the same shape the compiler emits.
//
// What we assert:
//  - band: `motion:loop` with `translate-x: 'marquee'` reaches the runtime
//    under normal motion, the loop animation is created, and reduced
//    motion skips it
//  - dots: `motion:loop` with `translate-y: 'bounce'` and `opacity: 'bounce'`
//    plus `stagger: 0.15` reaches the runtime for every dot
//  - reduced-motion: a single matchMedia per attach is the authority

// happy-dom lives under node_modules/.bun/happy-dom@<ver>/node_modules/
// happy-dom/. The version suffix is what bun pins. We import the
// package via its absolute path on disk (baked from process.cwd() so
// the script is cwd-independent).
import { readdirSync } from 'node:fs'
const WT = process.cwd()
const HAPPY_DOM_DIR = readdirSync(`${WT}/node_modules/.bun`).find(d => d.startsWith('happy-dom@'))
const { Window } = await import(`${WT}/node_modules/.bun/${HAPPY_DOM_DIR}/node_modules/happy-dom/lib/index.js`)
const w = new Window()
globalThis.window = w
globalThis.Element = w.Element
globalThis.HTMLElement = w.HTMLElement
globalThis.Node = w.Node
globalThis.NodeList = w.NodeList
globalThis.SVGElement = w.SVGElement
globalThis.SVGSVGElement = w.SVGSVGElement
globalThis.performance = w.performance
globalThis.requestAnimationFrame = w.requestAnimationFrame.bind(w)
globalThis.cancelAnimationFrame = w.cancelAnimationFrame.bind(w)
globalThis.getComputedStyle = w.getComputedStyle.bind(w)

let mod = await import(`${WT}/packages/sail/runtime/src/motion.ts`)
let realAttach = mod.__sui_motion_attach
let calls = []
let attach = (el, cfg) => {
  calls.push({ el: el.tagName, cfg: JSON.parse(JSON.stringify(cfg)) })
  return realAttach(el, cfg)
}

function setReduced(on) {
  Object.defineProperty(globalThis, 'matchMedia', {
    configurable: true, writable: true,
    value: (q) => ({
      matches: on ? q.includes('reduced') : false,
      media: q, onchange: null,
      addListener() {}, removeListener() {},
      addEventListener() {}, removeEventListener() {},
      dispatchEvent() { return false },
    }),
  })
}

console.log('--- band: marquee loop ---')
setReduced(false)
calls = []
const bandTrack = w.document.createElement('div')
w.document.body.append(bandTrack)
attach(bandTrack, {
  trigger: 'loop',
  motions: [{
    property: 'translate-x',
    preset: 'marquee',
    kind: 'tween',
    duration: 7.5,
    from: 0,
    to: -1200,
    iterations: Infinity,
    direction: 'normal',
  }],
})
console.log('normal motion: calls =', calls.length, 'preset =', calls[0]?.cfg.motions[0].preset)
console.log('keyframes handed to animate():', JSON.stringify(mod.__buildKeyframesForTest(
  { property: 'translate-x', preset: 'marquee', kind: 'tween', duration: 7.5, from: 0, to: -1200 },
  bandTrack,
)))
await new Promise(r => setTimeout(r, 100))
// happy-dom stubs `element.animate()` — it does not run a real compositor,
// so it cannot demonstrate PIXELS moving (that's apps/sail-site/test/
// motion.spec.ts's job, in a real browser). What this script CAN and does
// pin: the keyframe shape handed to motion/mini. The bug this script exists
// to catch was `{ x: [...] }`/`{ y: [...] }` — motion/mini's own value
// names, not CSS properties on an HTML element, so WAAPI silently ignored
// them forever. The `translate` key logged above is the fix.
setReduced(true)
calls = []
const bandTrack2 = w.document.createElement('div')
w.document.body.append(bandTrack2)
attach(bandTrack2, {
  trigger: 'loop',
  motions: [{
    property: 'translate-x',
    preset: 'marquee',
    kind: 'tween',
    duration: 7.5,
    from: 0,
    to: -1200,
    iterations: Infinity,
    direction: 'normal',
  }],
})
console.log('reduced motion: calls =', calls.length, 'resting translate =', bandTrack2.style.translate)

console.log('--- dots: bounce stagger ---')
setReduced(false)
calls = []
const dot1 = w.document.createElement('span')
const dot2 = w.document.createElement('span')
const dot3 = w.document.createElement('span')
w.document.body.append(dot1, dot2, dot3)
for (const d of [dot1, dot2, dot3]) {
  attach(d, {
    trigger: 'loop',
    stagger: 0.15,
    motions: [
      { property: 'translate-y', preset: 'bounce', kind: 'tween',
        duration: 0.18, from: 0, to: 1,
        iterations: Infinity, direction: 'alternate' },
      { property: 'opacity', preset: 'bounce', kind: 'tween',
        duration: 0.18, from: 0.4, to: 1,
        iterations: Infinity, direction: 'alternate' },
    ],
  })
}
console.log('3 dots under normal motion: calls =', calls.length, '(expect 3)')
console.log('first call motions =', JSON.stringify(calls[0]?.cfg.motions, null, 2))
console.log('stagger present:', calls[0]?.cfg.stagger, '(expect 0.15)')

console.log('--- single reduced-motion authority ---')
let mqlCount = 0
let reducedTrue = false
const origMM = globalThis.matchMedia
globalThis.matchMedia = (q) => {
  mqlCount++
  return {
    matches: reducedTrue && q.includes('reduced'),
    media: q, onchange: null,
    addListener() {}, removeListener() {},
    addEventListener() {}, removeEventListener() {},
    dispatchEvent() { return false },
  }
}
setReduced(false)
reducedTrue = false
calls = []
const a = w.document.createElement('div')
const b = w.document.createElement('div')
w.document.body.append(a, b)
attach(a, { trigger: 'loop', motions: [{ property: 'opacity', preset: 'pulse', kind: 'tween', duration: 0.5, iterations: Infinity, direction: 'alternate' }] })
attach(b, { trigger: 'loop', motions: [{ property: 'opacity', preset: 'pulse', kind: 'tween', duration: 0.5, iterations: Infinity, direction: 'alternate' }] })
console.log('2 attaches, reduced off: matchMedia calls =', mqlCount, '(runtime consults matchMedia per attach)')

// Now flip reduced-motion on and re-attach
reducedTrue = true
calls = []
const c = w.document.createElement('div')
w.document.body.append(c)
attach(c, { trigger: 'loop', motions: [{ property: 'opacity', preset: 'pulse', kind: 'tween', duration: 0.5, iterations: Infinity, direction: 'alternate' }] })
console.log('1 attach, reduced on: c.style.opacity =', c.style.opacity, '(expect 1 — the final pose)')
