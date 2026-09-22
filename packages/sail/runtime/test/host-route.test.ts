import { afterEach, describe, expect, it } from 'vitest'
import { hostRoute, setHostRoute } from '../src/host-route'
import { signal } from '../src/signal'

afterEach(() => {
  setHostRoute(null)
})

function fakeRoute() {
  return {
    params: {},
    query: {},
    pathname: signal('/docs'),
    hash: signal(''),
  }
}

describe('host route inject', () => {
  it('is unset until the host registers one', () => {
    expect(hostRoute()).toBeUndefined()
  })

  it('reads back the registered handle', () => {
    const route = fakeRoute()
    const navigate = () => {}
    setHostRoute({ route, navigate })
    expect(hostRoute()?.route).toBe(route)
    expect(hostRoute()?.navigate).toBe(navigate)
  })

  it('clears on null', () => {
    setHostRoute({ route: fakeRoute(), navigate: () => {} })
    setHostRoute(null)
    expect(hostRoute()).toBeUndefined()
  })
})
