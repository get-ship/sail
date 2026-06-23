import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { signal } from '../src/signal'
import { bindIf } from '../src/if'
import { bindFor } from '../src/for'
import {
  registerTransition,
  runIn,
  runOut,
  fade,
  slide,
  scale,
  type Transition,
} from '../src/transition'

// jsdom 25 doesn't ship Element.animate, so each test installs a tiny
// shim that returns a thenable Animation-like object. The shim records
// the keyframes + timing it was called with so assertions can inspect
// what the transition actually requested.

interface ShimAnim {
  keyframes: Keyframe[]
  timing: KeyframeAnimationOptions
  finished: Promise<void>
  cancel(): void
  resolve(): void
  cancelled: boolean
}

let anims: ShimAnim[] = []
const origAnimate = (Element.prototype as { animate?: unknown }).animate

function installAnimateShim() {
  anims = []
  ;(Element.prototype as unknown as {
    animate: (kf: Keyframe[], t: KeyframeAnimationOptions) => Animation
  }).animate = function (kf: Keyframe[], t: KeyframeAnimationOptions) {
    let resolveFn: () => void = () => {}
    const rec: ShimAnim = {
      keyframes: kf,
      timing: t,
      cancelled: false,
      cancel() {
        if (this.cancelled) return
        this.cancelled = true
        resolveFn()
      },
      resolve() { resolveFn() },
      finished: new Promise<void>(r => { resolveFn = r }),
    }
    anims.push(rec)
    return rec as unknown as Animation
  }
}

function uninstallAnimateShim() {
  if (origAnimate === undefined) {
    delete (Element.prototype as { animate?: unknown }).animate
  } else {
    ;(Element.prototype as { animate?: unknown }).animate = origAnimate
  }
}

function host() {
  const h = document.createElement('div')
  document.body.appendChild(h)
  return h
}

function ifAnchor(host: HTMLElement): Comment {
  const a = document.createComment('if')
  host.appendChild(a)
  return a
}

beforeEach(installAnimateShim)
afterEach(() => {
  uninstallAnimateShim()
  document.body.innerHTML = ''
})

describe('transition factories', () => {
  it('fade.in emits opacity 0→1 over the requested duration', () => {
    const el = document.createElement('div')
    const anim = fade.in!(el, { duration: 300 })
    expect(anim).toBeDefined()
    expect(anims).toHaveLength(1)
    expect(anims[0]!.keyframes).toEqual([{ opacity: 0 }, { opacity: 1 }])
    expect(anims[0]!.timing.duration).toBe(300)
  })

  it('fade.out emits opacity 1→0', () => {
    fade.out!(document.createElement('div'), {})
    expect(anims[0]!.keyframes).toEqual([{ opacity: 1 }, { opacity: 0 }])
    expect(anims[0]!.timing.duration).toBe(200) // default
  })

  it('slide honors the axis option', () => {
    slide.in!(document.createElement('div'), { axis: 'x' })
    const kf = anims[0]!.keyframes
    const first = kf[0] as { transform: string }
    expect(first.transform).toContain('translateX')
  })

  it('scale honors the `from` option', () => {
    scale.out!(document.createElement('div'), { from: 0.5 })
    const last = anims[0]!.keyframes[1] as { transform: string }
    expect(last.transform).toContain('scale(0.5)')
  })
})

describe('runIn / runOut', () => {
  it('does nothing when the element has no registration', () => {
    const el = document.createElement('div')
    expect(runIn(el)).toBeUndefined()
    expect(runOut(el)).toBeUndefined()
  })

  it('runs the in-factory when registered as "both"', () => {
    const el = document.createElement('div')
    registerTransition(el, fade, { duration: 50 })
    runIn(el)
    expect(anims).toHaveLength(1)
    expect(anims[0]!.timing.duration).toBe(50)
  })

  it('cancels any in-flight animation before starting the next', () => {
    const el = document.createElement('div')
    registerTransition(el, fade, { duration: 100 })
    runIn(el)
    const first = anims[0]!
    runOut(el)
    expect(first.cancelled).toBe(true)
  })
})

describe('bindIf with transitions', () => {
  it('runs the in-transition when the branch mounts', () => {
    const h = host()
    const a = ifAnchor(h)
    const visible = signal(true)
    bindIf({
      anchor: a,
      cond: () => visible(),
      then_: () => {
        const el = document.createElement('span')
        el.textContent = 'X'
        registerTransition(el, fade, { duration: 100 })
        return { nodes: [el] }
      },
    })
    expect(anims).toHaveLength(1)
    expect(anims[0]!.keyframes).toEqual([{ opacity: 0 }, { opacity: 1 }])
  })

  it('defers DOM removal until the out-transition finishes', async () => {
    const h = host()
    const a = ifAnchor(h)
    const visible = signal(true)
    bindIf({
      anchor: a,
      cond: () => visible(),
      then_: () => {
        const el = document.createElement('span')
        el.textContent = 'X'
        registerTransition(el, fade, { duration: 100 })
        return { nodes: [el] }
      },
    })
    expect(h.textContent).toBe('X')
    visible.set(false)
    // Out-transition started but not resolved yet — node still in DOM.
    expect(h.textContent).toBe('X')
    expect(anims).toHaveLength(2)
    anims[1]!.resolve()
    await new Promise(r => setTimeout(r, 0))
    expect(h.textContent).toBe('')
  })

  it('rapid re-mount rescues the departing branch — no double remove, no stall', async () => {
    const h = host()
    const a = ifAnchor(h)
    const visible = signal(true)
    let renderCount = 0
    const cleanup = vi.fn()
    bindIf({
      anchor: a,
      cond: () => visible(),
      then_: () => {
        renderCount++
        const el = document.createElement('span')
        el.textContent = 'X'
        registerTransition(el, fade, { duration: 100 })
        return { nodes: [el], cleanup }
      },
    })
    expect(renderCount).toBe(1)
    visible.set(false) // start out-transition
    visible.set(true)  // rescue
    // No re-render: the departing branch was promoted back to active.
    expect(renderCount).toBe(1)
    expect(cleanup).not.toHaveBeenCalled()
    expect(h.textContent).toBe('X')
    // Resolve the (now-cancelled) animation — must NOT remove the
    // node, because the rescue handed it back to `active`.
    anims[1]!.resolve()
    await new Promise(r => setTimeout(r, 0))
    expect(h.textContent).toBe('X')
    expect(cleanup).not.toHaveBeenCalled()
  })

  it('asymmetric in: + out: register both phases on one element', () => {
    const h = host()
    const a = ifAnchor(h)
    const visible = signal(true)
    bindIf({
      anchor: a,
      cond: () => visible(),
      then_: () => {
        const el = document.createElement('span')
        el.textContent = 'X'
        registerTransition(el, fade, {}, 'in')
        registerTransition(el, slide, {}, 'out')
        return { nodes: [el] }
      },
    })
    expect(anims).toHaveLength(1)
    expect(anims[0]!.keyframes).toEqual([{ opacity: 0 }, { opacity: 1 }])
    visible.set(false)
    expect(anims).toHaveLength(2)
    // out used slide → keyframes carry a transform.
    const outFirst = anims[1]!.keyframes[0] as { transform: string }
    expect(outFirst.transform).toBeDefined()
  })

  it('hard dispose flushes departing nodes immediately', () => {
    const h = host()
    const a = ifAnchor(h)
    const visible = signal(true)
    const cleanup = vi.fn()
    const stop = bindIf({
      anchor: a,
      cond: () => visible(),
      then_: () => {
        const el = document.createElement('span')
        el.textContent = 'X'
        registerTransition(el, fade, { duration: 100 })
        return { nodes: [el], cleanup }
      },
    })
    visible.set(false) // out-transition in flight; node still in DOM
    expect(h.textContent).toBe('X')
    stop()
    expect(h.textContent).toBe('')
    expect(cleanup).toHaveBeenCalledTimes(1)
  })
})

describe('bindFor with transitions', () => {
  it('runs in-transition for newly inserted rows', () => {
    const h = host()
    const a = ifAnchor(h)
    const rows = signal<{ id: number }[]>([])
    bindFor({
      anchor: a,
      items: () => rows(),
      key: r => r.id,
      render: r => {
        const li = document.createElement('li')
        li.textContent = String(r.id)
        registerTransition(li, fade, { duration: 50 })
        return { nodes: [li] }
      },
    })
    rows.set([{ id: 1 }, { id: 2 }])
    expect(anims).toHaveLength(2)
  })

  it('defers removal of departed rows until out-transition completes', async () => {
    const h = host()
    const a = ifAnchor(h)
    const rows = signal<{ id: number }[]>([{ id: 1 }, { id: 2 }])
    bindFor({
      anchor: a,
      items: () => rows(),
      key: r => r.id,
      render: r => {
        const li = document.createElement('li')
        li.textContent = String(r.id)
        registerTransition(li, fade, { duration: 50 })
        return { nodes: [li] }
      },
    })
    expect(h.textContent).toBe('12')
    anims.length = 0 // ignore enter animations
    rows.set([{ id: 1 }])
    expect(anims).toHaveLength(1)
    // Row 2 is mid-out — still in DOM.
    expect(h.textContent).toBe('12')
    anims[0]!.resolve()
    await new Promise(r => setTimeout(r, 0))
    expect(h.textContent).toBe('1')
  })
})

describe('reduced motion', () => {
  it('collapses duration to 0 when prefers-reduced-motion: reduce', () => {
    const orig = window.matchMedia
    window.matchMedia = (q: string) => ({
      matches: q === '(prefers-reduced-motion: reduce)',
      media: q,
      addEventListener() {},
      removeEventListener() {},
      addListener() {},
      removeListener() {},
      onchange: null,
      dispatchEvent() { return false },
    }) as unknown as MediaQueryList
    try {
      const el = document.createElement('div')
      registerTransition(el, fade, { duration: 5000 })
      runIn(el)
      expect(anims[0]!.timing.duration).toBe(0)
    } finally {
      window.matchMedia = orig
    }
  })
})

describe('custom transitions', () => {
  it('an author-defined Transition is invoked like a built-in', () => {
    const calls: string[] = []
    const custom: Transition = {
      in(node, _o) {
        calls.push('in:' + (node as HTMLElement).id)
        return undefined
      },
      out(node, _o) {
        calls.push('out:' + (node as HTMLElement).id)
        return undefined
      },
    }
    const h = host()
    const a = ifAnchor(h)
    const visible = signal(true)
    bindIf({
      anchor: a,
      cond: () => visible(),
      then_: () => {
        const el = document.createElement('span')
        el.id = 'x'
        registerTransition(el, custom)
        return { nodes: [el] }
      },
    })
    visible.set(false)
    expect(calls).toEqual(['in:x', 'out:x'])
  })
})
