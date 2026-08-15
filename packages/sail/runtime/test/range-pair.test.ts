// @vitest-environment jsdom
// rangePair — L1 <slider> host. Copies attrs onto the inner range inputs
// and exposes host.value so bind:value on the wrapper drives the thumb.
import { describe, expect, it, afterEach } from 'vitest'
import { rangePair } from '../src/actions/range-pair'
import { bindValue } from '../src/bind'
import { signal } from '../src/signal'

function makeHost(opts: {
  value?: string
  valueEnd?: string
  min?: string
  max?: string
  step?: string
  range?: boolean
  size?: string
  ticks?: string
} = {}): { host: HTMLElement; a: HTMLInputElement; b: HTMLInputElement } {
  const host = document.createElement('div')
  host.className = 'sail-slider'
  if (opts.value != null) host.setAttribute('value', opts.value)
  if (opts.valueEnd != null) host.setAttribute('value-end', opts.valueEnd)
  if (opts.min != null) host.setAttribute('min', opts.min)
  if (opts.max != null) host.setAttribute('max', opts.max)
  if (opts.step != null) host.setAttribute('step', opts.step)
  if (opts.range) host.setAttribute('range', '')
  if (opts.size != null) host.setAttribute('size', opts.size)
  if (opts.ticks != null) host.setAttribute('ticks', opts.ticks)
  const a = document.createElement('input')
  a.type = 'range'
  const b = document.createElement('input')
  b.type = 'range'
  host.append(a, b)
  document.body.append(host)
  return { host, a, b }
}

const flush = () => Promise.resolve()

describe('rangePair (use: action)', () => {
  afterEach(() => {
    document.body.replaceChildren()
  })

  it('copies host value onto the first thumb', () => {
    const { host, a } = makeHost({ value: '40', min: '0', max: '100' })
    rangePair(host)
    expect(a.value).toBe('40')
    expect(a.min).toBe('0')
    expect(a.max).toBe('100')
  })

  it('host.value setter moves the first thumb', () => {
    const { host, a } = makeHost({ value: '10', min: '0', max: '100' })
    rangePair(host)
    ;(host as HTMLInputElement).value = '70'
    expect(a.value).toBe('70')
    expect(host.getAttribute('value')).toBe('70')
  })

  it('host.value getter reads the first thumb', () => {
    const { host, a } = makeHost({ value: '10', min: '0', max: '100' })
    rangePair(host)
    a.value = '55'
    expect((host as HTMLInputElement).value).toBe('55')
  })

  it('user input writes host value attribute', () => {
    const { host, a } = makeHost({ value: '10', min: '0', max: '100' })
    rangePair(host)
    a.value = '33'
    a.dispatchEvent(new Event('input', { bubbles: true }))
    expect(host.getAttribute('value')).toBe('33')
    expect((host as HTMLInputElement).value).toBe('33')
  })

  it('bind:value on the wrapper drives the thumb and writes back', () => {
    const { host, a } = makeHost({ min: '0', max: '100' })
    rangePair(host)
    const $v = signal('25')
    bindValue(host, $v)
    expect(a.value).toBe('25')
    $v.set('80')
    expect(a.value).toBe('80')
    a.value = '12'
    a.dispatchEvent(new Event('input', { bubbles: true }))
    expect($v()).toBe('12')
  })

  it('number signal coerces so size knobs move the thumb', () => {
    const { host, a } = makeHost({ min: '0', max: '2', step: '1' })
    rangePair(host)
    const $idx = signal(1)
    bindValue(host, $idx as unknown as ReturnType<typeof signal<string>>)
    expect(a.value).toBe('1')
    $idx.set(2)
    expect(a.value).toBe('2')
  })

  it('copies size onto both thumbs so Style match applies', () => {
    const { host, a, b } = makeHost({ size: 'sm', value: '50' })
    rangePair(host)
    expect(a.getAttribute('size')).toBe('sm')
    expect(b.getAttribute('size')).toBe('sm')
  })

  it('copies ticks onto the first thumb', () => {
    const { host, a, b } = makeHost({ ticks: 'sm,md,lg', value: '1' })
    rangePair(host)
    expect(a.getAttribute('ticks')).toBe('sm,md,lg')
    expect(b.getAttribute('ticks')).toBeNull()
  })

  it('range keeps start ≤ end when the first thumb overshoots', () => {
    const { host, a, b } = makeHost({
      range: true,
      value: '20',
      valueEnd: '80',
      min: '0',
      max: '100',
    })
    rangePair(host)
    expect(a.value).toBe('20')
    expect(b.value).toBe('80')
    expect(b.hidden).toBe(false)
    a.value = '90'
    a.dispatchEvent(new Event('input', { bubbles: true }))
    expect(host.getAttribute('value')).toBe('80')
    expect(a.value).toBe('80')
  })

  it('syncs when thumbs are appended after the action attaches', async () => {
    const host = document.createElement('div')
    host.setAttribute('value', '42')
    host.setAttribute('min', '0')
    host.setAttribute('max', '100')
    document.body.append(host)
    rangePair(host)
    const a = document.createElement('input')
    a.type = 'range'
    host.append(a)
    await flush()
    expect(a.value).toBe('42')
  })
})
