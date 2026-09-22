import { describe, expect, it } from 'vitest'
import { snapshotSail } from '../src/snapshot'

describe('snapshotSail', () => {
  it('reports unavailable without a host bridge (honest debt)', async () => {
    const r = await snapshotSail('<p>hi</p>')
    expect(r.status).toBe('unavailable')
    expect(r.png).toBeNull()
    expect(r.reason).toMatch(/Takumi/)
  })

  it('uses host-provided render when supplied', async () => {
    const bytes = new Uint8Array([1, 2, 3])
    const r = await snapshotSail('<p>hi</p>', {
      render: async () => bytes,
    })
    expect(r.status).toBe('ok')
    expect(r.png).toBe(bytes)
  })
})
