// The <sail-media-image> renderer (raster + SVG, zoom/pan, gallery/lightbox).
//
// Harness: `renderComponent` from `@ship/test-setup` compiles the renderer
// `.sail` under the `sail-media-image` tag and registers it as a custom element
// (the new-tag-via-harness path — no WASM rebuild). The image renderer receives
// an already-resolved source via the `prop:src` identity property (a raster
// object-url/bytes, or SVG text the consumer already fetched) — it NEVER fetches
// its own bytes. happy-dom does not decode pixels, so lifecycle is driven by
// dispatching the native `<img>` `load`/`error` events the renderer listens for
// (the dispatcher-spec precedent: synthetic native events drive the emit seam).
//
// The component is LIGHT DOM and hand-authors no classes, so every query below
// is a real semantic: the raster <img>, the focusable zoom/pan stage that holds
// it, role=img for the inlined SVG, role=listbox/option for the gallery strip.
//
// Authority: build-spec §"Props" (sail-media-image), §"Backbone #2 —
// ViewerRenderer contract" (read-only, source-in-never-fetch,
// sail-viewer-load/-error via the free `emit`), §"Accessibility" (<img alt>, SVG
// role=img, gallery role=listbox/option + aria-selected), and behavioral spec
// #4 (image zoom/pan/gallery).

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { afterEach, describe, expect, it } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import imageRenderer from '../src/sail-media-image/sail-media-image.sail.controller.ts'

const SOURCE_URL = new URL('../src/sail-media-image/sail-media-image.sail', import.meta.url)
const SOURCE = readFileSync(SOURCE_URL, 'utf8')
// On-disk path so the compiler pairs the controller and runs the
// controller-aware passes (bind:this targets must be `ref(null)`) — the
// harness then rejects exactly what the daemon + CLI do (#263-A).
const SOURCE_PATH = fileURLToPath(SOURCE_URL)

interface ViewerSource {
  contentType?: string
  filename?: string
  text?: string
  bytes?: ArrayBuffer | Uint8Array
  url?: string
}

/** The raster `<img>` the renderer paints into. */
const imgEl = (el: HTMLElement): HTMLImageElement | null => el.querySelector('img')

/** The zoom/pan surface — the focusable box the raster <img> sits in. */
const stageEl = (el: HTMLElement): HTMLElement | null =>
  el.querySelector('[tabindex="0"]')

const live: Array<{ unmount(): void }> = []
afterEach(() => {
  for (const v of live.splice(0)) {
    try {
      v.unmount()
    } catch {
      /* best-effort */
    }
  }
})

/** Mount the image renderer and assign a live ViewerSource via the `prop:src`
 *  identity property (never a stringified attribute). */
function mountImage(src: ViewerSource, attrs?: Record<string, string | boolean>) {
  const v = renderComponent(SOURCE, 'sail-media-image', {
    controller: imageRenderer,
    sourcePath: SOURCE_PATH,
    ...(attrs ? { attrs } : {}),
  })
  ;(v.el as unknown as { src: ViewerSource }).src = src
  return v
}

const PNG_URL = 'blob:http://localhost/raster-fixture'

describe('<sail-media-image> — registration + raster', () => {

  it('renders a raster source into an <img>', async () => {
    const v = mountImage({ contentType: 'image/png', url: PNG_URL })
    live.push(v)
    await v.flush()
    expect(imgEl(v.el)).not.toBeNull()
  })

  it('points the <img> src at the resolved object-url (never fetched by the renderer)', async () => {
    const v = mountImage({ contentType: 'image/png', url: PNG_URL })
    live.push(v)
    await v.flush()
    expect(imgEl(v.el)!.getAttribute('src')).toBe(PNG_URL)
  })

  it('mounts the <img> inside a focusable zoom/pan stage (keyboard-operable)', async () => {
    const v = mountImage({ contentType: 'image/png', url: PNG_URL })
    live.push(v)
    await v.flush()
    const stage = stageEl(v.el)
    expect(stage).not.toBeNull()
    expect(stage!.contains(imgEl(v.el))).toBe(true)
  })
})

describe('<sail-media-image> — alt + fit props (#4 / a11y)', () => {
  it('reflects the alt prop onto the <img alt> (the real accessible name)', async () => {
    const v = mountImage({ contentType: 'image/png', url: PNG_URL }, { alt: 'A diagram' })
    live.push(v)
    await v.flush()
    expect(imgEl(v.el)!.getAttribute('alt')).toBe('A diagram')
  })

  it('reflects the fit enum onto the host (the styling channel reads it there)', async () => {
    const v = mountImage({ contentType: 'image/png', url: PNG_URL }, { fit: 'cover' })
    live.push(v)
    await v.flush()
    expect(v.el.getAttribute('fit')).toBe('cover')
  })
})

describe('<sail-media-image> — SVG text source (#4)', () => {
  const SVG = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10"/></svg>'

  it('renders an SVG text source with role=img (NOT a raster <img>)', async () => {
    const v = mountImage({ contentType: 'image/svg+xml', text: SVG })
    live.push(v)
    await v.flush()
    const svgHost = v.el.querySelector('[role="img"]')
    expect(svgHost).not.toBeNull()
  })

  it('inlines the SVG markup (bindHtml into a sandboxed container, not a fetch)', async () => {
    const v = mountImage({ contentType: 'image/svg+xml', text: SVG })
    live.push(v)
    await v.flush()
    expect(v.el.querySelector('svg')).not.toBeNull()
  })

  it('names the SVG region with an aria-label', async () => {
    const v = mountImage({ contentType: 'image/svg+xml', text: SVG }, { alt: 'Logo mark' })
    live.push(v)
    await v.flush()
    const svgHost = v.el.querySelector('[role="img"]')
    expect(svgHost!.getAttribute('aria-label')).toBe('Logo mark')
  })
})

describe('<sail-media-image> — zoom/pan (#4)', () => {
  it('reflects zoomable by default (zoom/pan affordance on)', async () => {
    const v = mountImage({ contentType: 'image/png', url: PNG_URL })
    live.push(v)
    await v.flush()
    expect(v.el.hasAttribute('zoomable')).toBe(true)
  })

  it('emits sail-viewer-zoom {scale} on a wheel-zoom over the stage', async () => {
    const v = mountImage({ contentType: 'image/png', url: PNG_URL })
    live.push(v)
    await v.flush()
    let scale: number | undefined
    v.el.addEventListener('sail-viewer-zoom', (e) => {
      scale = (e as CustomEvent<{ scale: number }>).detail.scale
    })
    const stage = stageEl(v.el)!
    stage.dispatchEvent(new WheelEvent('wheel', { deltaY: -120, bubbles: true, cancelable: true }))
    await v.flush()
    expect(typeof scale).toBe('number')
  })

  it('zooms via the "+" key (keyboard-operable)', async () => {
    const v = mountImage({ contentType: 'image/png', url: PNG_URL })
    live.push(v)
    await v.flush()
    let zooms = 0
    v.el.addEventListener('sail-viewer-zoom', () => zooms++)
    const stage = stageEl(v.el)!
    stage.dispatchEvent(new KeyboardEvent('keydown', { key: '+', bubbles: true }))
    await v.flush()
    expect(zooms).toBeGreaterThan(0)
  })

  it('does NOT emit zoom when zoomable is disabled', async () => {
    const v = mountImage({ contentType: 'image/png', url: PNG_URL }, { zoomable: 'false' })
    live.push(v)
    await v.flush()
    let zooms = 0
    v.el.addEventListener('sail-viewer-zoom', () => zooms++)
    stageEl(v.el)!.dispatchEvent(
      new WheelEvent('wheel', { deltaY: -120, bubbles: true, cancelable: true }),
    )
    await v.flush()
    expect(zooms).toBe(0)
  })
})

describe('<sail-media-image> — gallery strip (#4)', () => {
  const GALLERY: ViewerSource[] = [
    { contentType: 'image/png', url: 'blob:http://localhost/a' },
    { contentType: 'image/png', url: 'blob:http://localhost/b' },
    { contentType: 'image/png', url: 'blob:http://localhost/c' },
  ]

  /** Mount with an array of sources via the prop:src identity property. */
  function mountGallery() {
    const v = renderComponent(SOURCE, 'sail-media-image', {
      controller: imageRenderer,
      sourcePath: SOURCE_PATH,
      attrs: { gallery: '' },
    })
    ;(v.el as unknown as { src: ViewerSource[] }).src = GALLERY
    return v
  }

  it('renders a role=listbox strip of thumbnails (one option per source)', async () => {
    const v = mountGallery()
    live.push(v)
    await v.flush()
    const strip = v.el.querySelector('[role="listbox"]')
    expect(strip).not.toBeNull()
    expect(strip!.querySelectorAll('[role="option"]').length).toBe(GALLERY.length)
  })

  it('marks the active thumbnail aria-selected', async () => {
    const v = mountGallery()
    live.push(v)
    await v.flush()
    const selected = v.el.querySelectorAll('[role="option"][aria-selected="true"]')
    expect(selected.length).toBe(1)
  })

  it('emits sail-viewer-index-change {index} on arrow-key navigation', async () => {
    const v = mountGallery()
    live.push(v)
    await v.flush()
    let index: number | undefined
    v.el.addEventListener('sail-viewer-index-change', (e) => {
      index = (e as CustomEvent<{ index: number }>).detail.index
    })
    const strip = v.el.querySelector('[role="listbox"]') as HTMLElement
    strip.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }))
    await v.flush()
    expect(index).toBe(1)
  })
})

describe('<sail-media-image> — ViewerRenderer lifecycle events', () => {
  it('fires sail-viewer-load when the raster <img> loads', async () => {
    const v = mountImage({ contentType: 'image/png', url: PNG_URL })
    live.push(v)
    await v.flush()
    let loads = 0
    v.el.addEventListener('sail-viewer-load', () => loads++)
    imgEl(v.el)!.dispatchEvent(new Event('load'))
    await v.flush()
    expect(loads).toBeGreaterThan(0)
  })

  it('the load event bubbles + crosses the shadow boundary (composed)', async () => {
    const v = mountImage({ contentType: 'image/png', url: PNG_URL })
    live.push(v)
    await v.flush()
    let caught: CustomEvent | null = null
    v.root.addEventListener('sail-viewer-load', (e) => { caught = e as CustomEvent })
    imgEl(v.el)!.dispatchEvent(new Event('load'))
    await v.flush()
    expect(caught).not.toBeNull()
  })

  it('fires sail-viewer-error {message} when the raster <img> fails to decode', async () => {
    const v = mountImage({ contentType: 'image/png', url: PNG_URL })
    live.push(v)
    await v.flush()
    let caught: CustomEvent | null = null
    v.el.addEventListener('sail-viewer-error', (e) => { caught = e as CustomEvent })
    imgEl(v.el)!.dispatchEvent(new Event('error'))
    await v.flush()
    expect(caught).not.toBeNull()
    expect((caught as unknown as CustomEvent).detail).toHaveProperty('message')
  })

  it('fires sail-viewer-load once on a successful SVG inline', async () => {
    const v = mountImage({
      contentType: 'image/svg+xml',
      text: '<svg xmlns="http://www.w3.org/2000/svg"></svg>',
    })
    live.push(v)
    let loads = 0
    v.el.addEventListener('sail-viewer-load', () => loads++)
    ;(v.el as unknown as { src: ViewerSource }).src = {
      contentType: 'image/svg+xml',
      text: '<svg xmlns="http://www.w3.org/2000/svg"></svg>',
    }
    await v.flush()
    expect(loads).toBeGreaterThan(0)
  })
})

describe('<sail-media-image> — density is NOT a prop here (#11)', () => {
  it('exposes no density/dense prop on the renderer', async () => {
    const v = mountImage({ contentType: 'image/png', url: PNG_URL })
    live.push(v)
    await v.flush()
    const proto = Object.getPrototypeOf(v.el) as object
    expect(Object.getOwnPropertyNames(proto)).not.toContain('density')
    expect(Object.getOwnPropertyNames(proto)).not.toContain('dense')
  })
})
