// The exit-hold contract: `<if>`/`<match>` branch removal with a
// registered `fx:exit` must hold the node in the DOM until the exit
// animation finishes, must never wedge removal forever if the animation
// gets stuck, and an interrupted exit (branch flips back mid-exit) must
// leave exactly one copy of the node in the DOM with no orphaned
// animation still running behind it.
//
// Root cause this pins: `if.ts`'s swap engine already deferred removal
// correctly (`runLeave` → pending promise → removal on resolve). The break
// was in `motion.ts`'s lifecycle runner: the "exit" animation was created
// and played at MOUNT time (via `createAnimations` in the attach path),
// and `runLeave` just cancelled those controls and snapped to the final
// pose — never actually starting an animation, always resolving nothing,
// so `runLeave` returned `null` and `if.ts` removed the node immediately.
// The fix parks the exit config at attach and only creates/plays the
// animation inside `runLeave`, racing it against a bounded timeout.

import { afterEach, describe, expect, it, vi } from 'vitest'

const { animateCalls, controlsByEl } = vi.hoisted(() => ({
  animateCalls: [] as Array<{ el: unknown; keyframes: unknown; transition: unknown }>,
  controlsByEl: new Map<unknown, { cancelled: boolean; resolve: () => void }>(),
}))

vi.mock('motion/mini', () => ({
  animate: (el: unknown, keyframes: unknown, transition: unknown) => {
    animateCalls.push({ el, keyframes, transition })
    let resolveFinished!: () => void
    const finished = new Promise<void>((resolve) => { resolveFinished = resolve })
    const entry = { cancelled: false, resolve: resolveFinished }
    controlsByEl.set(el, entry)
    return {
      stop: () => {},
      cancel: () => { entry.cancelled = true },
      finished,
    }
  },
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  configurable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  }),
})

import { bindIf } from '../src/if'
import { signal } from '../src/signal'
import { __sui_motion_attach, __EXIT_TIMEOUT_MS_FOR_TEST } from '../src/motion'
import type { MotionAttachConfig } from '../src/motion'

const fadeOut: MotionAttachConfig = {
  trigger: 'exit',
  motions: [{ property: 'opacity', preset: 'fade-out', kind: 'tween', from: 1, to: 0, duration: 0.12 }],
}

function setup() {
  const host = document.createElement('div')
  document.body.appendChild(host)
  const anchor = document.createComment('if')
  host.appendChild(anchor)
  return { host, anchor }
}

function renderCardWithExit() {
  const el = document.createElement('div')
  el.textContent = 'card'
  const disposeMotion = __sui_motion_attach(el, fadeOut)
  return { nodes: [el], cleanup: disposeMotion }
}

// `finished.catch()` → `Promise.all` → `Promise.race` → `runLeave`'s
// `.then` → `if.ts`'s own `.then` is several microtask hops deep; flush
// them all rather than guessing a hop count.
const flush = async (): Promise<void> => {
  for (let i = 0; i < 10; i++) await Promise.resolve()
}

afterEach(() => {
  vi.restoreAllMocks()
  vi.useRealTimers()
  animateCalls.length = 0
  controlsByEl.clear()
  document.body.innerHTML = ''
})

describe('motion exit-hold — defers removal until the animation finishes', () => {
  it('keeps the node in the DOM while the exit animation is in flight, then removes it on completion', async () => {
    const { host, anchor } = setup()
    const test = signal(0)
    bindIf({ anchor, test: () => test(), branches: [renderCardWithExit] })
    expect(host.textContent).toBe('card')

    test.set(-1) // branch removed — exit should start, not remove immediately
    expect(host.textContent).toBe('card') // still held
    expect(animateCalls.length).toBe(1)

    const el = animateCalls[0]!.el
    const control = controlsByEl.get(el)!
    control.resolve() // animation finishes on its own
    await flush()
    await flush()

    expect(host.textContent).toBe('')
  })
})

describe('motion exit-hold — timeout guard', () => {
  it('never wedges removal forever: a stuck animation is force-removed after the bound', async () => {
    vi.useFakeTimers()
    const { host, anchor } = setup()
    const test = signal(0)
    bindIf({ anchor, test: () => test(), branches: [renderCardWithExit] })

    test.set(-1)
    expect(host.textContent).toBe('card')

    const el = animateCalls[0]!.el
    const control = controlsByEl.get(el)!
    // The animation's `finished` promise never resolves (a stuck/broken
    // animation) — only the timeout guard can unstick removal.
    await vi.advanceTimersByTimeAsync(__EXIT_TIMEOUT_MS_FOR_TEST)

    expect(control.cancelled).toBe(true) // guard cancels the stuck animation
    expect(host.textContent).toBe('')
  })
})

describe('motion exit-hold — interrupted exit', () => {
  it('a branch flip back mid-exit rescues the node: no leak, no double-mount', async () => {
    const { host, anchor } = setup()
    const test = signal(0)
    bindIf({ anchor, test: () => test(), branches: [renderCardWithExit] })

    test.set(-1) // start exiting
    expect(animateCalls.length).toBe(1)
    const el = animateCalls[0]!.el
    const control = controlsByEl.get(el)!

    test.set(0) // flip back before the exit animation finished
    expect(host.textContent).toBe('card')
    expect(host.querySelectorAll('div').length).toBe(1) // exactly one copy, no double-mount
    expect(control.cancelled).toBe(true) // the in-flight exit was cancelled, not left running

    control.resolve() // even if the stale animation's promise settles later...
    await flush()
    await flush()
    expect(host.textContent).toBe('card') // ...the rescued node is unaffected

    test.set(-1) // and a fresh exit still works after a rescue
    expect(host.textContent).toBe('card')
    const secondCall = animateCalls[animateCalls.length - 1]!
    controlsByEl.get(secondCall.el)!.resolve()
    await flush()
    await flush()
    expect(host.textContent).toBe('')
  })
})
