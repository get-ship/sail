// Confinement proof, productionized from .artifacts/ses-spike/proof.mjs: a
// controller endowed with ONLY the runtime surface + its declared ops can use
// them, but cannot reach ambient authority or tamper with intrinsics.
//
// lockdown() is process-wide (freezes intrinsics). vitest isolates test files
// in their own workers, so locking here doesn't bleed into other suites.

import { beforeAll, describe, expect, it } from 'vitest'

import { confineController, createConfinedFetch, isRealmLocked, lockdownRealm } from '../src/confine.ts'

interface Probe {
  count: number
  branch: string
  probes: {
    fetch: string
    XMLHttpRequest: string
    process: string
    tampered?: boolean
    tamperThrew?: string
  }
}

describe('SES confined controller execution', () => {
  beforeAll(() => lockdownRealm())

  it('endows runtime + ops, denies ambient authority, freezes intrinsics', () => {
    const auditLog: string[] = []

    // Stand-ins for the endowment surface the real runner injects.
    function signal(v: number) {
      let _v = v
      const s = () => _v
      ;(s as unknown as { set: (n: number) => void }).set = (n: number) => {
        _v = n
      }
      return s
    }
    const ship = {
      git: {
        overview: () => {
          auditLog.push('ship.git.overview')
          return { branch: 'main', clean: true }
        },
      },
    }

    // A controller that uses its endowments, then probes for ambient authority
    // and attempts to tamper with a shared intrinsic.
    const src = `
      const count = signal(0)
      count.set(count() + 41)
      const ov = ship.git.overview()
      const probes = {}
      probes.fetch = typeof fetch
      probes.XMLHttpRequest = typeof XMLHttpRequest
      probes.process = typeof process
      try { Array.prototype.push = () => {}; probes.tampered = true }
      catch (e) { probes.tamperThrew = e.constructor.name }
      ;({ count: count(), branch: ov.branch, probes })
    `

    const result = confineController<Probe>(src, {
      signal,
      ship,
      console: { info: (...a: unknown[]) => auditLog.push(String(a)) },
    })

    // Endowed capabilities work.
    expect(result.count).toBe(41)
    expect(result.branch).toBe('main')
    // Ambient authority is unreachable.
    expect(result.probes.fetch).toBe('undefined')
    expect(result.probes.XMLHttpRequest).toBe('undefined')
    expect(result.probes.process).toBe('undefined')
    // Intrinsics are frozen — the tamper didn't apply.
    expect(result.probes.tampered).not.toBe(true)
    // The host only saw the one op the controller was endowed + called.
    expect(auditLog).toEqual(['ship.git.overview'])
    expect(isRealmLocked()).toBe(true)
  })

  it('denies an un-endowed op (deny-by-default capability boundary)', () => {
    const src = `
      let reached = false
      try { ship.secrets.read(); reached = true } catch (e) { reached = e.constructor.name }
      ;({ reached })
    `
    const result = confineController<{ reached: unknown }>(src, {
      ship: { git: { overview: () => ({}) } },
    })
    // ship.secrets is not endowed → access throws, not silently succeeds.
    expect(result.reached).not.toBe(true)
  })
})

describe('createConfinedFetch — the runtime network-capability wall', () => {
  it('delegates to the real fetch for a granted origin', async () => {
    let called: [RequestInfo | URL, RequestInit | undefined] | undefined
    const realFetch = ((input: RequestInfo | URL, init?: RequestInit) => {
      called = [input, init]
      return Promise.resolve(new Response('ok'))
    }) as typeof fetch

    const confined = createConfinedFetch(['https://api.example.com'], realFetch)
    const res = await confined('https://api.example.com/x')
    expect(await res.text()).toBe('ok')
    expect(called?.[0]).toBe('https://api.example.com/x')
  })

  it('throws for an origin outside the allowlist, never reaching the real fetch', async () => {
    let reached = false
    const realFetch = (() => {
      reached = true
      return Promise.resolve(new Response('ok'))
    }) as typeof fetch

    const confined = createConfinedFetch(['https://api.example.com'], realFetch)
    await expect(confined('https://evil.com/x')).rejects.toThrow()
    expect(reached).toBe(false)
  })

  it('"self" grants a relative target but not an absolute cross-origin one', async () => {
    const realFetch = (() => Promise.resolve(new Response('ok'))) as typeof fetch
    const confined = createConfinedFetch(['self'], realFetch)
    await expect(confined('/api/x')).resolves.toBeInstanceOf(Response)
    await expect(confined('https://evil.com/x')).rejects.toThrow()
  })

  it('accepts URL and Request inputs, matching the same allowlist', async () => {
    const realFetch = (() => Promise.resolve(new Response('ok'))) as typeof fetch
    const confined = createConfinedFetch(['https://api.example.com'], realFetch)
    await expect(confined(new URL('https://api.example.com/x'))).resolves.toBeInstanceOf(Response)
    await expect(
      confined(new Request('https://evil.com/x')),
    ).rejects.toThrow()
  })
})
