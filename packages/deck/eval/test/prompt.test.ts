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
    expect(out).toContain('tone: default|primary|destructive')
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
    expect(parts.system).toContain('```sui```')
    expect(parts.system).toContain('No raw <script>')
    expect(parts.system).toContain('customElements.define')
    // The retired generation must not leak back into the primer.
    expect(parts.system).not.toContain('<state')
    expect(parts.system).not.toContain('html.set')
  })

  it('includes the catalog version, primitives, few-shots, and feature spec', () => {
    expect(parts.user).toContain(mockCatalog.version)
    expect(parts.user).toContain('<sui-button>')
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
      fixtures: { template: '<column gap="2"><p>seed</p></column>', setup: 'export default () => ({})' },
    })
    expect(withFixture.user).toContain('Starter fixture')
    expect(withFixture.user).toContain('<column gap="2"><p>seed</p></column>')
    expect(withFixture.user).toContain('export default')
  })
})

describe('parseModelReply', () => {
  it('extracts a sui block', () => {
    const out = parseModelReply('here you go:\n\n```sui\n<h1>hi</h1>\n```\n')
    expect(out.template).toBe('<h1>hi</h1>')
    expect(out.setup).toBeUndefined()
  })

  it('keeps the inline controller inside the single sui block', () => {
    const file = '<controller>\nexport default null\n</controller>\n\n<h1>hi</h1>'
    const out = parseModelReply('```sui\n' + file + '\n```')
    expect(out.template).toBe(file)
    expect(out.setup).toBeUndefined()
  })

  it('does not honor the retired html.set fence', () => {
    expect(() => parseModelReply('```html.set\n<p>x</p>\n```')).toThrow(/sui/)
  })

  it('throws when no sui block is present', () => {
    expect(() => parseModelReply('no fences here')).toThrow(/sui/)
  })

  it('tolerates trailing tokens on the info string', () => {
    const out = parseModelReply('```sui view\n<p>ok</p>\n```')
    expect(out.template).toBe('<p>ok</p>')
  })
})
