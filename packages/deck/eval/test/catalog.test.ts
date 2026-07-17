import { describe, expect, it } from 'vitest'
import { loadCatalog, parseCatalog } from '../src/catalog'
import { mockCatalog } from '../src/mock-catalog'
import { renderCatalog } from '../src/prompt'

// P1.5: the `covers the five required primitives` describe block tested
// `mockCatalog.primitives.map((p) => p.tag).sort()` — asserting on a
// private fixture against itself. The behavior the test was guarding
// (every primitive a user of the `sail-eval` CLI sees) is what
// `renderCatalog(catalog)` emits into the prompt. Rewrote to assert
// on the rendered CLI output: a model that reads the prompt sees
// every primitive tag + its props + its slots.
describe('renderCatalog (CLI-visible primitive surface)', () => {
  it('emits a <tag> header for every primitive in the catalog', () => {
    const rendered = renderCatalog(mockCatalog)
    for (const p of mockCatalog.primitives) {
      expect(rendered, `tag ${p.tag} should appear in CLI output`).toContain(`<${p.tag}>`)
    }
  })

  it('lists sail-input.value as a string-typed prop in the rendered output', () => {
    const rendered = renderCatalog(mockCatalog)
    // The model sees `props:` followed by `name:type` for each prop.
    // sail-input has a `value` prop of type 'string' (the bindable
    // marker is an internal annotation; the CLI surface shows type).
    const inputSection = rendered.split('<sail-input>')[1]?.split('<')[0] ?? ''
    expect(inputSection).toMatch(/value\s*:\s*string/)
  })

  it('marks sail-button.disabled as bool-typed (not string-typed) in the rendered output', () => {
    const rendered = renderCatalog(mockCatalog)
    const buttonSection = rendered.split('<sail-button>')[1]?.split('<')[0] ?? ''
    expect(buttonSection).toMatch(/disabled\s*:\s*bool/)
  })
})

describe('loadCatalog', () => {
  it('returns the mock catalog with stable shape', async () => {
    const cat = await loadCatalog()
    expect(cat.version).toBeTruthy()
    expect(cat.primitives.length).toBeGreaterThan(0)
  })
})

describe('parseCatalog', () => {
  it('round-trips the mock catalog through JSON', () => {
    const json = JSON.parse(JSON.stringify(mockCatalog))
    const parsed = parseCatalog(json)
    expect(parsed.primitives.length).toBe(mockCatalog.primitives.length)
    expect(parsed.primitives[0]?.tag).toBe(mockCatalog.primitives[0]?.tag)
  })

  it('rejects a non-object root', () => {
    expect(() => parseCatalog([] as unknown)).toThrow(/object/)
    expect(() => parseCatalog(null as unknown)).toThrow(/object/)
  })

  it('rejects a missing version', () => {
    expect(() => parseCatalog({ primitives: [] })).toThrow(/version/)
  })

  it('rejects an unknown prop type', () => {
    expect(() =>
      parseCatalog({
        version: 'v1',
        primitives: [{ tag: 'x', props: [{ name: 'p', type: 'date' }] }],
      }),
    ).toThrow(/type/)
  })

  it('requires non-empty values[] on a union prop', () => {
    expect(() =>
      parseCatalog({
        version: 'v1',
        primitives: [{ tag: 'x', props: [{ name: 'p', type: 'union' }] }],
      }),
    ).toThrow(/values/)
  })

  it('keeps optional summary when present', () => {
    const parsed = parseCatalog({
      version: 'v1',
      primitives: [{ tag: 'x', summary: 'a thing' }],
    })
    expect(parsed.primitives[0]?.summary).toBe('a thing')
  })
})
