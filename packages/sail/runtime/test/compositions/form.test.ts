import { describe, expect, it, vi } from 'vitest'
import { form } from '../../src/index'

interface SettingsValues extends Record<string, unknown> {
  theme: string
  accent: string
}

const hex = (v: string): string | null =>
  /^#[0-9a-f]{6}$/i.test(v) ? null : 'must be hex color'

describe('form()', () => {
  it('exposes one signal per field, plus a values snapshot', () => {
    const f = form<SettingsValues>({ initial: { theme: 'system', accent: '#3366ff' } })

    expect(f.fields.theme()).toBe('system')
    expect(f.fields.accent()).toBe('#3366ff')
    expect(f.values()).toEqual({ theme: 'system', accent: '#3366ff' })
  })

  it('errors are per field and null by default', () => {
    const f = form<SettingsValues>({
      initial: { theme: 'system', accent: 'bad' },
      validate: ({ accent }) => ({ accent: hex(accent) }),
    })

    expect(f.errors().accent).toBe('must be hex color')
    expect(f.errors().theme ?? null).toBeNull()
    expect(f.valid()).toBe(false)

    f.fields.accent.set('#3366ff')
    expect(f.errors().accent).toBeNull()
    expect(f.valid()).toBe(true)
  })

  it('dirty flips on a field change and clears when the value returns', () => {
    const f = form<SettingsValues>({ initial: { theme: 'system', accent: '#000000' } })
    expect(f.dirty()).toBe(false)

    f.fields.theme.set('dark')
    expect(f.dirty()).toBe(true)

    f.fields.theme.set('system')
    expect(f.dirty()).toBe(false)
  })

  it('reset writes the initial values back and clears pending + lastError', async () => {
    const f = form<SettingsValues>({
      initial: { theme: 'system', accent: '#abcabc' },
      submit: () => {
        throw new Error('boom')
      },
    })
    f.fields.theme.set('dark')
    await f.submit()
    expect(f.lastError()).toBeInstanceOf(Error)

    f.reset()
    expect(f.fields.theme()).toBe('system')
    expect(f.fields.accent()).toBe('#abcabc')
    expect(f.pending()).toBe(false)
    expect(f.lastError()).toBeNull()
    expect(f.dirty()).toBe(false)
  })

  it('submit hands the current values to the user submit and settles pending', async () => {
    const submit = vi.fn()
    const f = form<SettingsValues>({
      initial: { theme: 'system', accent: '#000000' },
      submit,
    })
    f.fields.theme.set('dark')
    f.fields.accent.set('#ffffff')

    expect(f.pending()).toBe(false)
    await f.submit()

    expect(submit).toHaveBeenCalledWith({ theme: 'dark', accent: '#ffffff' })
    expect(f.pending()).toBe(false)
  })

  it('submit bails on a validation error without invoking the user submit', async () => {
    const submit = vi.fn()
    const f = form<SettingsValues>({
      initial: { theme: 'system', accent: 'bad' },
      validate: ({ accent }) => ({ accent: hex(accent) }),
      submit,
    })

    await f.submit()
    expect(submit).not.toHaveBeenCalled()
  })

  it('a throwing submit lands on lastError and releases pending', async () => {
    const f = form<SettingsValues>({
      initial: { theme: 'system', accent: '#000000' },
      submit: () => {
        throw new Error('boom')
      },
    })

    await f.submit()
    expect(f.pending()).toBe(false)
    expect(f.lastError()).toBeInstanceOf(Error)
    expect(f.lastError()!.message).toBe('boom')
  })

  it('submit preventDefaults the event it is given', async () => {
    const f = form<SettingsValues>({
      initial: { theme: 'system', accent: '#000000' },
      submit: () => {},
    })
    const event = { preventDefault: vi.fn() } as unknown as Event

    await f.submit(event)
    expect(event.preventDefault).toHaveBeenCalledOnce()
  })
})
