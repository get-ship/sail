<!--
  <sail-media-image> — raster + SVG viewer (source-in, never fetch).
  `src` is a host identity property (ViewerSource | ViewerSource[] | string) AND a
  string URL attribute (default PE — authors write src="https://…"). Zoom/pan via
  engine/zoom-pan; gallery strip when src is an array.
-->

<scope>
  props: {
    alt = ''
    fit: 'contain' | 'cover' | 'actual' = 'contain'
    zoomable = true
    gallery = false
  }
</scope>
<controller>
// Glue: route src identity → raster <img> / SVG bindHtml / gallery strip.
// Zoom math in engine/zoom-pan; lifecycle via emitViewer*.

import { emit } from '@ship/sail-runtime/internal'
import { bindHtml } from '@ship/sail-runtime/internal'
import type { Action } from '@ship/sail-runtime'
import type { ViewerSource } from '../engine/types'
import { emitViewerError, emitViewerLoad } from '../engine/renderer-contract'
import { attachZoomPan, type ZoomPanState } from './engine/zoom-pan'
import type { Scope } from './sail-media-image.sail'

type ImageSrc = ViewerSource | ViewerSource[] | string | null

interface GalleryItem {
  id: string
  url: string
  selected: boolean
}

function firstSource(src: ImageSrc): ViewerSource | string | null {
  if (src == null) return null
  if (Array.isArray(src)) return src[0] ?? null
  return src
}

function svgTextOf(src: ViewerSource | string | null): string | null {
  if (src == null || typeof src === 'string') return null
  const ct = src.contentType ?? ''
  const text = src.text ?? ''
  const looksSvg = ct.includes('svg') || /^\s*<svg[\s>]/.test(text)
  return looksSvg && text ? text : null
}

/** Raster url from pre-resolved `url` or object-URL from `bytes` (track `created` to revoke). */
function rasterUrlOf(src: ViewerSource | string | null): { url: string; created: boolean } {
  if (src == null) return { url: '', created: false }
  if (typeof src === 'string') return { url: src, created: false }
  if (src.url) return { url: src.url, created: false }
  if (src.bytes) {
    const blob = new Blob([src.bytes as BlobPart])
    return { url: URL.createObjectURL(blob), created: true }
  }
  return { url: '', created: false }
}

export default controller((_ctx: Scope) => {
    // Host `src` identity is defined in mount (not a reflectable prop).
    const srcSig = signal<ImageSrc>(null)
    const activeIndex = signal(0)

    const isSvg = derived<boolean>(() => svgTextOf(firstSource(srcSig())) != null)

    const sources = derived<ViewerSource[]>(() => {
      const s = srcSig()
      return Array.isArray(s) ? s : []
    })
    const isGallery = derived<boolean>(() => sources().length > 1)

    const galleryItems = derived<GalleryItem[]>(() => {
      const list = sources()
      const active = activeIndex()
      return list.map((s, i) => {
        const { url } = rasterUrlOf(s)
        return { id: `thumb-${i}`, url, selected: i === active }
      })
    })

    const zoomPanSig = signal<ZoomPanState>({ x: 0, y: 0, scale: 1 })

    let hostEl: HTMLElement | null = null

    let created: string[] = []
    const revokeCreated = (): void => {
      for (const u of created.splice(0)) URL.revokeObjectURL(u)
    }
    const zoomable = (): boolean => hostEl?.getAttribute('zoomable') !== 'false'

    let lastImgUrl = ''
    const rasterImg: Action<void> = (node) => {
      const img = node as HTMLImageElement
      const onImgLoad = (): void => { if (hostEl) emitViewerLoad(hostEl) }
      const onImgError = (): void => {
        if (hostEl) emitViewerError(hostEl, 'Failed to decode image')
      }
      img.addEventListener('load', onImgLoad)
      img.addEventListener('error', onImgError)

      const stop = effect(() => {
        if (isSvg()) return
        const { url, created: didCreate } = rasterUrlOf(firstSource(srcSig()))
        if (didCreate && url !== lastImgUrl) created.push(url)
        lastImgUrl = url
        if (url) img.setAttribute('src', url)
        else img.removeAttribute('src')
      })

      return {
        destroy: () => {
          stop()
          img.removeEventListener('load', onImgLoad)
          img.removeEventListener('error', onImgError)
        },
      }
    }

    const svgMarkup = (): string => svgTextOf(firstSource(srcSig())) ?? ''
    const svgInline: Action<void> = (node) => {
      const stop = bindHtml(node as HTMLElement, svgMarkup)
      return { destroy: stop }
    }

    // Transform stamped by use:zoomPan action (--zoom-tx); controller never touches el.style.
    const applyTransform = (_st: ZoomPanState): void => {}
    const onZoom = (scale: number): void => {
      if (hostEl) emit(hostEl, 'sail-viewer-zoom', { scale })
    }
    const zoomStage: Action<void> = (node) => {
      const detach = attachZoomPan({
        stage: node as HTMLElement,
        enabled: zoomable,
        onChange: applyTransform,
        onZoom,
      })
      return { destroy: () => detach?.() }
    }

    const onStripKey = (e: KeyboardEvent): void => {
      const count = sources().length
      if (count === 0) return
      let next = activeIndex()
      if (e.key === 'ArrowRight' || e.key === 'ArrowDown') next = Math.min(count - 1, next + 1)
      else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') next = Math.max(0, next - 1)
      else return
      e.preventDefault()
      if (next === activeIndex()) return
      activeIndex.set(next)
      if (hostEl) emit(hostEl, 'sail-viewer-index-change', { index: next })
    }
    const galleryStrip: Action<void> = (node) => {
      node.addEventListener('keydown', onStripKey as EventListener)
      return {
        destroy: () => node.removeEventListener('keydown', onStripKey as EventListener),
      }
    }

    mount((host) => {
      hostEl = host
      if (!host.hasAttribute('zoomable')) host.setAttribute('zoomable', '')

      // Identity prop — live reference for ViewerSource / arrays; string URLs
      // also work via the host `src` attribute (default PE — no author glue).
      Object.defineProperty(host, 'src', {
        configurable: true,
        get: () => srcSig(),
        set: (s: ImageSrc) => {
          srcSig.set(s)
          activeIndex.set(0)
        },
      })

      // Default PE: seed string URL from host attribute so authors write
      // `<sail-media-image src="https://…" />` without querySelector glue.
      // Identity setter still accepts ViewerSource / arrays for programmatic hosts.
      const applyAttrSrc = (): void => {
        const attrSrc = host.getAttribute('src')
        if (attrSrc == null) return
        const cur = srcSig()
        if (cur == null || typeof cur === 'string') {
          if (cur !== attrSrc) {
            srcSig.set(attrSrc)
            activeIndex.set(0)
          }
        }
      }
      applyAttrSrc()
      const mo = new MutationObserver(applyAttrSrc)
      mo.observe(host, { attributes: true, attributeFilter: ['src'] })

      let lastInlined: unknown = null
      const stopSvgEmit = effect(() => {
        if (!isSvg()) return
        const src = firstSource(srcSig())
        if (svgTextOf(src) && src !== lastInlined) {
          lastInlined = src
          emitViewerLoad(host)
        }
      })

      return () => {
        mo.disconnect()
        stopSvgEmit()
        revokeCreated()
      }
    })

    return { srcSig, activeIndex, isSvg, sources, isGallery, galleryItems, rasterImg, svgInline, zoomStage, galleryStrip, zoomPanSig }
  })

</controller>
<if {isSvg()}>
  <center
    role="img"
    aria-label={alt()}
    use:svgInline
    style({ box: position(relative).w(full).h(full).minH(28).overflow(hidden), paint: bg(surface-1).radius(md) })
  ></center>
{else =>
  <center
    tabindex="0"
    use:zoomStage
    style({ box: position(relative).w(full).h(full).minH(28).overflow(hidden), paint: bg(surface-1).radius(md) })
  >
    <img
      alt={alt()}
      use:rasterImg
      use:zoomPan={zoomPanSig()}
      style({ box: display(block).w(full).h(full).maxH(full), paint: objectFit(cover).transform(bind(zoomTx)) })
    >
  </center>
  <if {isGallery()}>
    <row role="listbox" aria-label="Image gallery" gap="2" use:galleryStrip style({ box: pad(2).overflowX(auto) })>
      <for {galleryItems() as item} key={item.id}>
        <row
          role="option"
          gap="0"
          aria-selected={item.selected ? 'true' : 'false'}
          style({
            box: grow(0).pad(0).overflow(hidden),
            paint: bg(surface-1).borderWidth(1).border(border).radius(sm).cursor(pointer),
            when: { selected: { paint: borderWidth(1).border(primary).shadow(accent) } },
          })
        >
          <img src={item.url} alt="" />
        </row>
      </for>
    </row>
  </if>
}</if>
