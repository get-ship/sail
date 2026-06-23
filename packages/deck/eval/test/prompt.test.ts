import { describe, expect, it } from 'vitest'
import {
  buildPrompt,
  parseModelReply,
  renderCatalog,
} from '../src/prompt'
import { mockCatalog } from '../src/mock-catalog'

describe('renderCatalog', () => {
  it('emits each primitive with props, slots, events', () => {
    const out = renderCatalog(mockCatalog)
    for (const p of mockCatalog.primitives) {
      expect(out).toContain(`<${p.tag}>`)
    }
    expect(out).toContain('tone: neutral|primary|danger')
    expect(out).toContain('bindable')
    expect(out).toContain('click: Fires')
  })
})

describe('buildPrompt', () => {
  const parts = buildPrompt({
    description: 'A counter view.',
    catalog: mockCatalog,
    fixtures: { template: '' },
  })

  it('includes the system primer with the hard rules', () => {
    expect(parts.system).toContain('html.set')
    expect(parts.system).toContain('No raw <script>')
    expect(parts.system).toContain('customElements.define')
  })

  it('includes the catalog version, primitives, few-shots, and feature spec', () => {
    expect(parts.user).toContain(mockCatalog.version)
    expect(parts.user).toContain('<ship-button>')
    expect(parts.user).toContain('Example feature')
    expect(parts.user).toContain('A counter view.')
  })

  it('omits the starter fixture block when no fixture provided', () => {
    expect(parts.user).not.toContain('Starter fixture')
  })

  it('includes the starter fixture block when a fixture is provided', () => {
    const withFixture = buildPrompt({
      description: 'extend this',
      catalog: mockCatalog,
      fixtures: { template: '<state name="x" value="1" />', setup: 'export default () => ({})' },
    })
    expect(withFixture.user).toContain('Starter fixture')
    expect(withFixture.user).toContain('<state name="x" value="1" />')
    expect(withFixture.user).toContain('export default')
  })
})

describe('parseModelReply', () => {
  it('extracts an html.set block', () => {
    const out = parseModelReply('here you go:\n\n```html.set\n<h1>hi</h1>\n```\n')
    expect(out.template).toBe('<h1>hi</h1>')
    expect(out.setup).toBeUndefined()
  })

  it('extracts both html.set and ts blocks', () => {
    const reply = '```html.set\n<h1>hi</h1>\n```\n\n```ts\nexport default () => ({})\n```'
    const out = parseModelReply(reply)
    expect(out.template).toBe('<h1>hi</h1>')
    expect(out.setup).toBe('export default () => ({})')
  })

  it('falls back to a plain html fence', () => {
    const out = parseModelReply('```html\n<p>x</p>\n```')
    expect(out.template).toBe('<p>x</p>')
  })

  it('throws when no html block is present', () => {
    expect(() => parseModelReply('no fences here')).toThrow(/html\.set/)
  })

  it('tolerates trailing tokens on the info string', () => {
    const out = parseModelReply('```html.set view\n<p>ok</p>\n```')
    expect(out.template).toBe('<p>ok</p>')
  })
})
