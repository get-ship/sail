import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

const REPO = resolve(dirname(fileURLToPath(import.meta.url)), '../../../../../')

function injectFrameworkCss() {
  const css = [
    readFileSync(resolve(REPO, 'packages/sail/tokens/src/tokens.css'), 'utf8'),
    readFileSync(resolve(REPO, 'packages/sail/tokens/src/element-skins.css'), 'utf8'),
  ].join('\n')
  const el = document.createElement('style')
  el.setAttribute('data-card-bleed', '')
  el.textContent = css
  document.head.appendChild(el)
  return el
}

const VARIANTS = ['default', 'outline', 'soft', 'elevated', 'ghost'] as const
const TONES = ['primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted'] as const
const PADDINGS = ['none', 'sm', 'md', 'lg'] as const
const ACCENT = ['off', 'hover', 'static'] as const

const CARD = `<card>
  <h2 slot="header">Overview</h2>
  <p>Surface, radius, elevation on the element.</p>
  <span slot="footer">Updated just now</span>
</card>`

describe('l1 card', () => {
  it('default is a region named Card with its header', () => {
    const v = renderRoute(CARD)
    expect(getByRole(v.root, 'region', { name: 'Card' })).toBeTruthy()
    expect(getByRole(v.root, 'heading', { name: 'Overview', level: 2 })).toBeTruthy()
    expect(getByText(v.root, 'Surface, radius, elevation on the element.')).toBeTruthy()
    expect(getByText(v.root, 'Updated just now')).toBeTruthy()
    v.unmount()
  })

  it('each variant is still a named Card', () => {
    for (const variant of VARIANTS) {
      const v = renderRoute(`<card variant="${variant}">
  <h2 slot="header">Overview</h2>
  <p>Body</p>
</card>`)
      expect(getByRole(v.root, 'region', { name: 'Card' })).toBeTruthy()
      expect(getByRole(v.root, 'heading', { name: 'Overview', level: 2 })).toBeTruthy()
      v.unmount()
    }
  })

  it('each tone is still a named Card', () => {
    for (const tone of TONES) {
      const v = renderRoute(`<card tone="${tone}">
  <h2 slot="header">Overview</h2>
</card>`)
      expect(getByRole(v.root, 'region', { name: 'Card' })).toBeTruthy()
      v.unmount()
    }
  })

  it('each padding is still a named Card', () => {
    for (const padding of PADDINGS) {
      const v = renderRoute(`<card padding="${padding}">
  <h2 slot="header">Overview</h2>
</card>`)
      expect(getByRole(v.root, 'region', { name: 'Card' })).toBeTruthy()
      v.unmount()
    }
  })

  it('interactive is still a named Card', () => {
    const v = renderRoute(`<card interactive>
  <h2 slot="header">Overview</h2>
</card>`)
    expect(getByRole(v.root, 'region', { name: 'Card' })).toBeTruthy()
    v.unmount()
  })

  it('each accent-bar is still a named Card', () => {
    for (const accent of ACCENT) {
      const v = renderRoute(`<card accent-bar="${accent}">
  <h2 slot="header">Overview</h2>
</card>`)
      expect(getByRole(v.root, 'region', { name: 'Card' })).toBeTruthy()
      v.unmount()
    }
  })

  it('status-rail, fill, and spotlight are still a named Card', () => {
    for (const attrs of ['status-rail', 'fill', 'spotlight'] as const) {
      const v = renderRoute(`<card ${attrs}>
  <h2 slot="header">Overview</h2>
</card>`)
      expect(getByRole(v.root, 'region', { name: 'Card' })).toBeTruthy()
      v.unmount()
    }
  })

  it('padding=lg image wrap margin cancels host pad-lg', () => {
    const sheet = injectFrameworkCss()
    const v = renderRoute(`<card padding="lg">
  <img slot="image" src="/photo.jpg" alt="Hero">
  <p>Body copy</p>
</card>`)
    const img = getByRole(v.root, 'img', { name: 'Hero' })
    const wrap = img.parentElement
    expect(wrap).toBeTruthy()
    expect(wrap!.className).toMatch(/grouppaddinglg-margin/)
    const host = wrap!.closest('.sail-grp')
    expect(host?.getAttribute('padding')).toBe('lg')
    // Happy-dom does not resolve token calc() on descendants; the live sheet
    // + wrap class are the contract a browser would apply.
    expect(sheet.textContent).toContain(".sail-grp[padding='lg'] .s-grouppaddinglg-marginb5t5x6")
    expect(sheet.textContent).toContain('margin-top:calc(-1 * var(--space-5))')
    expect(sheet.textContent).toContain('margin-inline:calc(-1 * var(--space-6))')
    expect(wrap!.className).toMatch(/s-grouppaddinglg-marginb5t5x6/)
    v.unmount()
    sheet.remove()
  })
})
