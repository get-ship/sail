import { afterEach, describe, expect, it } from 'vitest'
import { applyDocumentHead, applySailHead } from '../src/document-head'

describe('applyDocumentHead', () => {
  afterEach(() => {
    document.title = ''
    document.querySelectorAll('[data-sail-head]').forEach((el) => el.remove())
  })

  it('sets title, lang, and managed meta/link from __sailHead', () => {
    applyDocumentHead({
      __sailHead: {
        title: 'Getting started',
        lang: 'en',
        metas: [{ name: 'description', content: 'Install Sail.' }],
        links: [{ rel: 'canonical', href: '/docs' }],
      },
    })
    expect(document.title).toBe('Getting started')
    expect(document.documentElement.lang).toBe('en')
    const desc = document.querySelector('meta[name="description"]')
    expect(desc?.getAttribute('content')).toBe('Install Sail.')
    expect(desc?.hasAttribute('data-sail-head')).toBe(true)
    const link = document.querySelector('link[rel="canonical"]')
    expect(link?.getAttribute('href')).toBe('/docs')
  })

  it('applies a table head without a compiled module', () => {
    applySailHead({
      title: 'About',
      metas: [{ name: 'description', content: 'Who we are.' }],
    })
    expect(document.title).toBe('About')
    expect(document.querySelector('meta[name="description"]')?.getAttribute('content')).toBe(
      'Who we are.',
    )
  })

  it('replaces previous managed tags and restores fallback title', () => {
    applyDocumentHead({
      __sailHead: {
        title: 'A',
        metas: [{ name: 'description', content: 'one' }],
      },
    })
    applyDocumentHead({}, 'Site')
    expect(document.title).toBe('Site')
    expect(document.querySelectorAll('meta[data-sail-head]').length).toBe(0)
  })
})
