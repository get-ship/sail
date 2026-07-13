import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-markdown/sui-markdown.compiled.js'
import { renderMarkdown } from '../src/sui-markdown/sui-markdown.sui.controller.ts'
import { signal } from '@ship/sui-runtime'
import { bindHtml } from '@ship/sui-runtime'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => (el.shadowRoot ?? el)

describe('renderMarkdown', () => {
  it('renders h1–h3 headings', () => {
    const out = renderMarkdown('# H1\n## H2\n### H3')
    expect(out).toContain('<h1>H1</h1>')
    expect(out).toContain('<h2>H2</h2>')
    expect(out).toContain('<h3>H3</h3>')
  })

  it('renders unordered lists', () => {
    const out = renderMarkdown('- one\n- two')
    expect(out).toContain('<ul><li>one</li><li>two</li></ul>')
  })

  it('renders ordered lists', () => {
    const out = renderMarkdown('1. first\n2. second')
    expect(out).toContain('<ol><li>first</li><li>second</li></ol>')
  })

  it('renders bold and italic inline', () => {
    const out = renderMarkdown('**bold** and *italic*')
    expect(out).toContain('<strong>bold</strong>')
    expect(out).toContain('<em>italic</em>')
  })

  it('renders inline code', () => {
    const out = renderMarkdown('use `code` here')
    expect(out).toContain('<code>code</code>')
  })

  it('renders fenced code blocks with data-lang', () => {
    const out = renderMarkdown('```ts\nconst a = 1\n```')
    expect(out).toContain('<pre data-lang="ts">')
    expect(out).toContain('const a = 1')
  })

  it('escapes HTML entities in text', () => {
    const out = renderMarkdown('<script>bad</script>')
    expect(out).not.toContain('<script>')
    expect(out).toContain('&lt;script&gt;')
  })

  it('renders links with noopener noreferrer', () => {
    const out = renderMarkdown('[Ship](https://example.com)')
    expect(out).toContain('<a href="https://example.com" rel="noopener noreferrer">Ship</a>')
  })

  it('renders blockquotes', () => {
    const out = renderMarkdown('> a quoted line')
    expect(out).toContain('<blockquote>')
    expect(out).toContain('a quoted line')
  })
})

describe('bindHtml focus retention (used by sui-markdown)', () => {
  it('preserves input focus across content updates via morph', () => {
    const root = document.createElement('div')
    document.body.appendChild(root)

    const $html = signal('<p>First</p><input type="text" />')
    bindHtml(root, () => $html())

    const input = root.querySelector('input')!
    input.focus()
    expect(document.activeElement).toBe(input)

    $html.set('<p>Updated</p><input type="text" />')
    expect(document.activeElement).toBe(root.querySelector('input'))

    document.body.removeChild(root)
  })
})

describe('<sui-markdown>', () => {

  it('mounts and renders headings from value prop', async () => {
    const { el } = mount(reg, 'sui-markdown', { value: '# Hello' })
    await flush()
    expect(shadow(el).querySelector('h1')?.textContent).toBe('Hello')
  })

  it('mounts and renders paragraphs', async () => {
    const { el } = mount(reg, 'sui-markdown', { value: 'simple paragraph' })
    await flush()
    const p = shadow(el).querySelector('p')
    expect(p?.textContent).toContain('simple paragraph')
  })

  it('reacts to value attribute change', async () => {
    const { el } = mount(reg, 'sui-markdown', { value: '# First' })
    await flush()
    expect(shadow(el).querySelector('h1')?.textContent).toBe('First')
    el.setAttribute('value', '# Second')
    await flush()
    expect(shadow(el).querySelector('h1')?.textContent).toBe('Second')
  })

  it('renders fenced code block inside component', async () => {
    const { el } = mount(reg, 'sui-markdown', { value: '```js\nconsole.log(1)\n```' })
    await flush()
    expect(shadow(el).querySelector('pre')).not.toBeNull()
    expect(shadow(el).querySelector('code')?.textContent).toContain('console.log(1)')
  })

  it('empty value renders nothing significant', async () => {
    const { el } = mount(reg, 'sui-markdown', { value: '' })
    await flush()
    expect(shadow(el).querySelector('p, h1, ul, ol, blockquote, pre')).toBeNull()
  })
})
