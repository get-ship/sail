// G6 / LANG-1 item 1 — late-mounted dynamic default slot must not wedge the
// event loop. Root cause was `use:recoverySurface` (L1 <error-state>): its
// MutationObserver rewrote toggle/code textContent unconditionally, which
// re-fired the observer forever under childList+characterData observation
// (slot projection + dynamic default-slot text).

import { afterEach, describe, expect, it } from 'vitest'
import { component, signal } from '../src/index'
import { bindIf } from '../src/if'
import { bindText } from '../src/bind'
import type { MountModule } from '../src/component-mount'
import { recoverySurface } from '../src/actions/recovery-surface'
import '../src/contracts'

let n = 0
const freshTag = (): string => `recovery-slot-${++n}`
const flush = (): Promise<void> => new Promise((r) => setTimeout(r, 0))
const drainMicrotasks = async (rounds = 40): Promise<void> => {
  for (let i = 0; i < rounds; i++) {
    await new Promise<void>((r) => queueMicrotask(() => r()))
  }
}

afterEach(() => {
  document.body.innerHTML = ''
})

/** Count MutationObserver deliveries; throw if a storm develops. */
function watchMO(limit = 120): { count: () => number; restore: () => void } {
  let hits = 0
  const Orig = globalThis.MutationObserver
  globalThis.MutationObserver = class extends Orig {
    constructor(cb: MutationCallback) {
      super((records, obs) => {
        hits += 1
        if (hits > limit) {
          throw new Error(`MutationObserver storm: ${hits} deliveries (limit ${limit})`)
        }
        return cb(records, obs)
      })
    }
  } as typeof MutationObserver
  return {
    count: () => hits,
    restore: () => {
      globalThis.MutationObserver = Orig
    },
  }
}

/** error-state-shaped anatomy recoverySurface indexes into. */
const anatomy = `
  <div class="sail-root" role="alert">
    <div class="icon"></div>
    <div class="copy">
      <div class="title"><slot name="title"></slot></div>
      <div class="desc"><slot></slot></div>
      <div class="code"></div>
      <button type="button">Show details</button>
      <div class="details"><slot name="details"></slot></div>
    </div>
    <div class="actions"></div>
  </div>
`

function registerErrorLike(tag: string): void {
  component(
    tag,
    {},
    {
      setup: () => ({
        __mount: (host: HTMLElement) => {
          const tmpl = document.createElement('template')
          tmpl.innerHTML = anatomy
          host.append(...Array.from(tmpl.content.childNodes))
          const root = host.querySelector('.sail-root') as HTMLElement
          const handle = recoverySurface(root)
          return () => handle?.destroy?.()
        },
      }),
    } as unknown as MountModule,
  )
}

describe('recoverySurface + late dynamic default slot (G6)', () => {
  it('stamps role=alert + aria-live=assertive on the recovery host', () => {
    const root = document.createElement('div')
    root.innerHTML = `
      <div class="icon"></div>
      <div class="copy">
        <div class="title">Broken</div>
        <div class="desc">try again</div>
        <div class="code"></div>
        <button type="button">Show details</button>
        <div class="details"></div>
      </div>
      <div class="actions"></div>
    `
    document.body.appendChild(root)
    const handle = recoverySurface(root)
    expect(root.getAttribute('role')).toBe('alert')
    expect(root.getAttribute('aria-live')).toBe('assertive')
    handle?.destroy?.()
    root.remove()
  })

  it('mounts with dynamic default-slot body after async state flip without MO storm', async () => {
    const mo = watchMO(80)
    try {
      const tag = freshTag()
      registerErrorLike(tag)

      const parent = document.createElement('div')
      document.body.appendChild(parent)
      const err = signal<string | null>(null)
      const anchor = document.createComment('if')
      parent.appendChild(anchor)

      const dispose = bindIf({
        anchor,
        cond: () => !!err(),
        then_: () => {
          const host = document.createElement(tag)
          host.setAttribute('variant', 'inline')

          const title = document.createElement('span')
          title.setAttribute('slot', 'title')
          title.textContent = 'Launch failed'
          host.appendChild(title)

          // Dynamic default-slot body — the eval-ui /launch wedge path.
          const t = document.createTextNode('')
          host.appendChild(t)
          const stop = bindText(t, () => err() ?? '')

          return { nodes: [host], cleanup: () => stop() }
        },
      })

      // Idle first (async work), then flip — late mount.
      await flush()
      expect(parent.querySelector(tag)).toBeNull()

      err.set('StartBatch failed: leaf down')
      await drainMicrotasks(50)
      await flush()

      expect(mo.count()).toBeLessThan(80)
      expect(parent.textContent).toContain('Launch failed')
      expect(parent.textContent).toContain('StartBatch failed: leaf down')

      const host = parent.querySelector(tag)!
      const dflt = host.querySelector('slot:not([name])')
      expect(dflt?.textContent).toContain('StartBatch failed: leaf down')

      // Subsequent signal updates must stay finite too.
      err.set('StartBatch failed: retry exhausted')
      await drainMicrotasks(50)
      await flush()
      expect(mo.count()).toBeLessThan(80)
      expect(dflt?.textContent).toContain('retry exhausted')

      dispose()
    } finally {
      mo.restore()
    }
  })

  it('code attribute + dynamic default slot stays finite (syncCode is idempotent)', async () => {
    const mo = watchMO(80)
    try {
      const tag = freshTag()
      registerErrorLike(tag)

      const msg = signal('rpc 5xx')
      const host = document.createElement(tag)

      const title = document.createElement('span')
      title.setAttribute('slot', 'title')
      title.textContent = 'Launch failed'
      host.appendChild(title)

      const t = document.createTextNode(' ')
      host.appendChild(t)
      const stop = bindText(t, () => msg())

      document.body.appendChild(host)
      // recoverySurface observes the sail-root (use: target), not the CE host.
      const root = host.querySelector('.sail-root') as HTMLElement
      root.setAttribute('code', 'ERR_TIMEOUT')
      await drainMicrotasks(50)
      await flush()

      expect(mo.count()).toBeLessThan(80)
      const code = host.querySelector('.code')
      expect(code?.textContent).toBe('ERR_TIMEOUT')
      expect(host.querySelector('slot:not([name])')?.textContent).toContain('rpc 5xx')

      root.setAttribute('code', 'ERR_DOWN')
      await drainMicrotasks(20)
      await flush()
      expect(mo.count()).toBeLessThan(80)
      expect(code?.textContent).toBe('ERR_DOWN')

      stop()
      host.remove()
    } finally {
      mo.restore()
    }
  })

  it('details slot content stamps has-details without thrashing', async () => {
    const mo = watchMO(80)
    try {
      const tag = freshTag()
      registerErrorLike(tag)

      const host = document.createElement(tag)
      const title = document.createElement('span')
      title.setAttribute('slot', 'title')
      title.textContent = 'Launch failed'
      host.appendChild(title)

      const details = document.createElement('pre')
      details.setAttribute('slot', 'details')
      details.textContent = 'stack\ntrace'
      host.appendChild(details)

      const body = document.createTextNode('dynamic body')
      host.appendChild(body)

      document.body.appendChild(host)
      await drainMicrotasks(50)
      await flush()

      expect(mo.count()).toBeLessThan(80)
      const root = host.querySelector('.sail-root')
      expect(root?.getAttribute('data-state')).toContain('has-details')
      host.remove()
    } finally {
      mo.restore()
    }
  })
})
