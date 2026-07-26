<!--
  <sail-media-image> — the raster + SVG image renderer arm of the sail-media
  family (Backbone #2 — ViewerRenderer).

  Read-only, source-in-never-fetch. A resolved source arrives via `src`
  identity (a raster object-url/bytes, or SVG text the consumer already fetched)
  — NEVER an attribute, NEVER a fetch. A raster source paints into the native
  <img> (whose `load`/`error` drive the lifecycle events) inside a zoom/pan
  stage. An SVG TEXT source inlines through `bindHtml` into a sandboxed
  `role="img"` container (no raster <img>, no innerHTML in the controller). A
  `gallery` source array renders a `role="listbox"` thumbnail strip with
  arrow-key navigation.

  Density is the platform axis (`<html data-density>`): this renderer is NOT
  row-shaped and exposes NEITHER `density` NOR `dense`.

  Props (build-spec §Props):
    alt       string                    reflect — the real accessible name (<img alt> / SVG aria-label)
    fit       contain|cover|actual      reflect — object-fit
    zoomable  bool, true                reflect — wheel/keyboard zoom + pan
    gallery   bool, false               reflect — multi-image strip (sources via src array)

  Events (ViewerRenderer contract, via emit):
    sail-viewer-load              — first successful raster load / SVG inline
    sail-viewer-error {message}   — raster decode failure
    sail-viewer-zoom {scale}      — wheel/keyboard zoom
    sail-viewer-index-change {index} — gallery arrow-navigation
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
// sail-media-image — the raster + SVG image renderer controller (tier-2, glue-
// only). A resolved source arrives via `src` identity (single ViewerSource
// OR an array for `gallery`). The controller:
//   - routes raster sources into the use:rasterImg <img> (resolving bytes→object-url
//     when the consumer did not pre-resolve a url) and wires load/error → the
//     ViewerRenderer lifecycle events;
//   - inlines an SVG TEXT source through `bindHtml` (no innerHTML) into the
//     sandboxed role=img container, emitting `sail-viewer-load` once;
//   - delegates zoom/pan to `engine/zoom-pan.ts` (the pre-committed split),
//     emitting `sail-viewer-zoom {scale}`;
//   - drives the `gallery` strip's active index + arrow navigation, emitting
//     `sail-viewer-index-change {index}`.
//
// Source-in, never fetch: no `fetch`/`EventSource`/`ship.*`. Heavy interaction
// math lives in the engine; lifecycle emits go through the sanctioned
// `emitViewer*` helpers (the free `emit`).
//
// Authority: build-spec Slice 6, §"Backbone #2 — ViewerRenderer contract",
// §"Props" (sail-media-image), §Accessibility, behavioral spec #4.

import { bindHtml, controller, derived, effect, emit, signal } from '@ship/sail-runtime'
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

/** First source of a (possibly array) src assignment. */
function firstSource(src: ImageSrc): ViewerSource | string | null {
  if (src == null) return null
  if (Array.isArray(src)) return src[0] ?? null
  return src
}

/** The SVG markup of a source, if it is an SVG text source. */
function svgTextOf(src: ViewerSource | string | null): string | null {
  if (src == null || typeof src === 'string') return null
  const ct = src.contentType ?? ''
  const text = src.text ?? ''
  const looksSvg = ct.includes('svg') || /^\s*<svg[\s>]/.test(text)
  return looksSvg && text ? text : null
}

/** A raster object-url for a source: a pre-resolved `url`, else an object URL
 *  minted from resolved `bytes`. Returns `{ url, created }` so the caller can
 *  revoke only what it created. */
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
    // src identity — assigned to the host `src` property in mount.
    const srcSig = signal<ImageSrc>(null)
    const activeIndex = signal(0)


    // SVG text source → role=img inline path; else the raster <img> path.
    const isSvg = derived<boolean>(() => svgTextOf(firstSource(srcSig())) != null)

    // Gallery is on when the `gallery` bool prop is set AND the source is an
    // array. Read off the live signal so re-assigning the source recomputes.
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

    // Set by mount before any action body can run.
    let hostEl: HTMLElement | null = null

    let created: string[] = []
    const revokeCreated = (): void => {
      for (const u of created.splice(0)) URL.revokeObjectURL(u)
    }
    const zoomable = (): boolean => hostEl?.getAttribute('zoomable') !== 'false'

    // --- Raster <img> wiring: src + load/error → lifecycle events ------------
    // Each of the four actions below replaces a ref PLUS a hand-rolled
    // "did the element change? tear down the old one" effect. The action's own
    // lifecycle IS that bookkeeping, so the identity locals are gone.
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

    // --- SVG inline path: bindHtml into the sandboxed role=img container ------
    // `bindHtml` owns its own reactive re-read of the markup, so it is wired
    // once per mounted container — which is exactly once per action run.
    const svgMarkup = (): string => svgTextOf(firstSource(srcSig())) ?? ''
    const svgInline: Action<void> = (node) => {
      const stop = bindHtml(node as HTMLElement, svgMarkup)
      return { destroy: stop }
    }

    // --- zoom/pan: attach to the stage, apply transform, emit sail-viewer-zoom
    const applyTransform = (_st: ZoomPanState): void => {
      // Runtime geometry goes through an action: the <img> carries
      // use:zoomPan and that action stamps --zoom-tx, which sx reads as
      // transform-(--zoom-tx). The controller never touches el.style.
    }
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

    // --- gallery: active-index arrow navigation -------------------------------
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
      // `zoomable` defaults to true; reflect the default so `.sail-media-image[zoomable]`
      // CSS + the affordance check see it when the consumer left it unset.
      if (!host.hasAttribute('zoomable')) host.setAttribute('zoomable', '')

      // src identity setter — a live JS reference (or array), never a
      // stringified attribute. Routes into the scope signal so render re-resolves.
      Object.defineProperty(host, 'src', {
        configurable: true,
        get: () => srcSig(),
        set: (s: ImageSrc) => {
          srcSig.set(s)
          activeIndex.set(0)
        },
      })

      // A separate effect emits `sail-viewer-load` once per distinct source
      // inlined (re-assigning a new source re-fires), mirroring the raster
      // <img> load lifecycle. It needs no element, so it stays in mount.
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
        stopSvgEmit()
        revokeCreated()
      }
    })

    return { srcSig, activeIndex, isSvg, sources, isGallery, galleryItems, rasterImg, svgInline, zoomStage, galleryStrip, zoomPanSig }
  })

</controller>
{#if isSvg()}
  <center role="img" aria-label={alt()} use:svgInline sx={style.position(relative).overflow(hidden).bg(surface-1).radius(md)}></center>
{:else}
  <center tabindex="0" use:zoomStage sx={style.position(relative).overflow(hidden).bg(surface-1).radius(md)}>
    <img alt={alt()} use:rasterImg use:zoomPan={zoomPanSig()} sx={style.display(block).maxH(full).transform(bind(zoomTx))} />
  </center>
  {#if isGallery()}
    <row role="listbox" aria-label="Image gallery" gap="2" use:galleryStrip sx={style.pad(2).overflowX(auto)}>
      {#for item of galleryItems() key={item.id}}
        <row role="option" gap="0" grow="0" aria-selected={item.selected ? 'true' : 'false'} sx={style.pad(0).overflow(hidden).bg(surface-1).borderWidth(1).border(border).radius(sm).cursor(pointer).when({ selected: style.borderWidth(1).border(primary).shadow(accent) })}>
          <img src={item.url} alt="" />
        </row>
      {/for}
    </row>
  {/if}
{/if}
