// Confinement proof, productionized from .artifacts/ses-spike/proof.mjs: a
// controller endowed with ONLY the runtime surface + its declared ops can use
// them, but cannot reach ambient authority or tamper with intrinsics.
//
// lockdown() is process-wide (freezes intrinsics). vitest isolates test files
// in their own workers, so locking here doesn't bleed into other suites.

import { beforeAll, describe, expect, it } from 'vitest'

import { confineController, isRealmLocked, lockdownRealm } from '../src/confine.ts'

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
