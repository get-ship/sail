import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { accessibleName, getByRole, getByText, queryByRole } from '@ship/sail-test'

const TONES = ['muted', 'info', 'success', 'warning', 'danger'] as const

describe('l1 alert', () => {
  it('is a status named by its body', async () => {
    const v = renderRoute(`<alert>Saved just now.</alert>`)
    await v.flush()
    expect(getByRole(v.root, 'status', { name: 'Saved just now.' })).toBeTruthy()
    v.unmount()
  })

  it('title and body are locatable by text', async () => {
    const v = renderRoute(`<alert>
  <strong slot="title">Saved</strong>
  Your changes are live.
</alert>`)
    await v.flush()
    expect(getByText(v.root, 'Saved')).toBeTruthy()
    expect(getByText(v.root, 'Your changes are live.')).toBeTruthy()
    v.unmount()
  })

  it('dismissible exposes a dismiss button', async () => {
    const v = renderRoute(`<alert dismissible>Saved just now.</alert>`)
    await v.flush()
    expect(getByRole(v.root, 'button', { name: 'Dismiss' })).toBeTruthy()
    v.unmount()
  })

  it('tone does not change the body accessible name to the tone word', async () => {
    for (const tone of TONES) {
      const v = renderRoute(`<alert tone="${tone}">Disk is full</alert>`)
      await v.flush()
      expect(getByText(v.root, 'Disk is full')).toBeTruthy()
      const live = queryByRole(v.root, 'status') ?? queryByRole(v.root, 'alert')
      expect(live).toBeTruthy()
      expect(accessibleName(live!)).not.toBe(tone)
      v.unmount()
    }
  })
})
