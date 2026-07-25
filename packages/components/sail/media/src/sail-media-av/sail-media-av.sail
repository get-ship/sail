<!--
  <sail-media-av> — the video/audio renderer arm of the sail-media family
  (Backbone #2 — ViewerRenderer).

  Read-only, source-in-never-fetch. A resolved source arrives via `src`
  identity (a video/audio object-url, or resolved bytes the consumer fetched)
  — NEVER an attribute, NEVER a fetch. The renderer auto-detects video-vs-audio
  from the descriptor (overridable via `kind`) and plays it through the NATIVE
  <video>/<audio controls> element, which carries built-in assistive-tech
  semantics. An optional decorative <canvas aria-hidden> draws decoded audio
  peaks beside the real <audio>, which stays the accessible control.

  Density is the platform axis (`<html data-density>`): this renderer is NOT
  row-shaped and exposes NEITHER `density` NOR `dense`.

  Props (build-spec §Props):
    kind      video|audio|auto   reflect — element selection (auto = sniff)
    autoplay  bool, false        reflect — begin playback on load
    controls  bool, true         reflect — native transport controls
    waveform  bool, false        reflect — audio waveform canvas

  Events (ViewerRenderer contract, via emit):
    sail-viewer-load              — native loadedmetadata
    sail-viewer-error {message}   — failed source
    sail-viewer-play/-pause/-ended — native transport
-->

<scope>
  props: {
    kind: 'auto' | 'video' | 'audio' = 'auto'
    autoplay = false
    controls = true
    waveform = false
  }
</scope>
<controller>
// sail-media-av — the video/audio renderer controller (tier-2, glue-only).
// A resolved source arrives via `src` identity. The controller:
//   - picks <video> vs <audio> from the `kind` override or a content-type sniff;
//   - points the native element at the resolved url (minting an object-url from
//     resolved bytes when the consumer did not pre-resolve a url, and revoking
//     it on unmount — no leak);
//   - applies `controls`/`autoplay` and wires the native transport events
//     (loadedmetadata/play/pause/ended/error) to the ViewerRenderer lifecycle;
//   - draws the optional decorative waveform via `engine/waveform.ts` (the
//     pre-committed split) while the real <audio> stays the accessible control.
//
// Source-in, never fetch: no `fetch`/`EventSource`/`ship.*`. Peak decode lives
// in the engine; lifecycle emits go through the sanctioned `emitViewer*`
// helpers (the free `emit`).
//
// Authority: build-spec Slice 7, §"Backbone #2 — ViewerRenderer contract",
// §"Props" (sail-media-av), §Accessibility, behavioral spec #5.

import { controller, derived, effect, emit, signal } from '@ship/sail-runtime'
import type { Action } from '@ship/sail-runtime'
import type { ViewerSource } from '../engine/types'
import { emitViewerError, emitViewerLoad } from '../engine/renderer-contract'
import { decodePeaks, drawWaveform } from './engine/waveform'
import type { Scope } from './sail-media-av.sail'

type MediaSrc = ViewerSource | string | null

/** Resolve a media url for a source: a pre-resolved `url`, else an object URL
 *  minted from resolved `bytes`. Returns `{ url, created }` so only minted
 *  urls are revoked. */
function mediaUrlOf(src: MediaSrc): { url: string; created: boolean } {
  if (src == null) return { url: '', created: false }
  if (typeof src === 'string') return { url: src, created: false }
  if (src.url) return { url: src.url, created: false }
  if (src.bytes) {
    const blob = new Blob([src.bytes as BlobPart], {
      ...(src.contentType ? { type: src.contentType } : {}),
    })
    return { url: URL.createObjectURL(blob), created: true }
  }
  return { url: '', created: false }
}

/** Content-type of a source (sniff input for video-vs-audio). */
function contentTypeOf(src: MediaSrc): string {
  if (src == null || typeof src === 'string') return ''
  return src.contentType ?? ''
}

/** Resolved-bytes of a source, for waveform decode. */
function bytesOf(src: MediaSrc): ArrayBuffer | Uint8Array | null {
  if (src == null || typeof src === 'string') return null
  return src.bytes ?? null
}

export default controller((_ctx: Scope) => {
    // src identity — assigned to the host `src` property in mount.
    const srcSig = signal<MediaSrc>(null)
    // The `kind` override is read off the host attribute in `isVideo` so the
    // enum prop need not surface through the typed Scope.
    const kindOverride = signal<string>('auto')

    const isVideo = derived<boolean>(() => {
      const k = kindOverride()
      if (k === 'video') return true
      if (k === 'audio') return false
      const ct = contentTypeOf(srcSig())
      // auto: video unless the type is explicitly audio.
      if (ct.startsWith('audio/')) return false
      if (ct.startsWith('video/')) return true
      return true
    })

    const showWaveform = signal<boolean>(false)

    // Set by mount before any action body can run.
    let hostEl: HTMLElement | null = null

    const wantsControls = (): boolean => hostEl?.getAttribute('controls') !== 'false'
    const wantsAutoplay = (): boolean =>
      hostEl?.hasAttribute('autoplay') === true && hostEl.getAttribute('autoplay') !== 'false'

    let created: string[] = []
    const revokeCreated = (): void => {
      for (const u of created.splice(0)) URL.revokeObjectURL(u)
    }
    let lastUrl = ''

    // --- native element wiring: src + transport events → lifecycle ------------
    // `mediaEl` used to be a ref bound on BOTH the <video> and the <audio>
    // branch, with a `wired` identity check to re-bind listeners when {#if}
    // swapped one for the other. The action IS that lifecycle: it runs on the
    // element that mounts and destroys with the element that leaves, so the
    // identity bookkeeping is gone.
    const mediaWire: Action<void> = (node) => {
      const el = node as HTMLMediaElement
      const onLoadedMeta = (): void => { if (hostEl) emitViewerLoad(hostEl) }
      const onPlay = (): void => { if (hostEl) emit(hostEl, 'sail-viewer-play') }
      const onPause = (): void => { if (hostEl) emit(hostEl, 'sail-viewer-pause') }
      const onEnded = (): void => { if (hostEl) emit(hostEl, 'sail-viewer-ended') }
      const onError = (): void => {
        if (hostEl) emitViewerError(hostEl, 'Failed to load media source')
      }
      el.addEventListener('loadedmetadata', onLoadedMeta)
      el.addEventListener('play', onPlay)
      el.addEventListener('pause', onPause)
      el.addEventListener('ended', onEnded)
      el.addEventListener('error', onError)

      const stop = effect(() => {
        if (wantsControls()) el.setAttribute('controls', '')
        else el.removeAttribute('controls')
        if (wantsAutoplay()) el.setAttribute('autoplay', '')

        const { url, created: didCreate } = mediaUrlOf(srcSig())
        if (didCreate && url !== lastUrl) created.push(url)
        lastUrl = url
        if (url) el.setAttribute('src', url)
        else el.removeAttribute('src')
      })

      return {
        destroy: () => {
          stop()
          el.removeEventListener('loadedmetadata', onLoadedMeta)
          el.removeEventListener('play', onPlay)
          el.removeEventListener('pause', onPause)
          el.removeEventListener('ended', onEnded)
          el.removeEventListener('error', onError)
        },
      }
    }

    // --- decorative waveform: decode peaks, draw to the canvas ----------------
    const waveform: Action<void> = (node) => {
      const canvas = node as HTMLCanvasElement
      const stop = effect(() => {
        if (!showWaveform()) return
        const bytes = bytesOf(srcSig())
        void decodePeaks(bytes).then((peaks) => {
          if (canvas.isConnected) drawWaveform(canvas, peaks)
        })
      })
      return { destroy: stop }
    }

    mount((host) => {
      hostEl = host
      // Seed the kind read from the host attribute (enum prop not surfaced through
      // the typed Scope) so `isVideo` resolves correctly.
      kindOverride.set(host.getAttribute('kind') ?? 'auto')

      // The waveform canvas shows only for an AUDIO element with the bool prop on.
      // Keep it reactive on the source (the src identity supersedes the
      // initial null), so a late-assigned audio source still draws the canvas.
      const waveformOn =
        host.hasAttribute('waveform') && host.getAttribute('waveform') !== 'false'
      const stopWaveformGate = effect(() => {
        showWaveform.set(waveformOn && !isVideo())
      })

      // src identity setter — a live JS reference, never a stringified
      // attribute. Routes into the scope signal so render re-resolves.
      Object.defineProperty(host, 'src', {
        configurable: true,
        get: () => srcSig(),
        set: (s: MediaSrc) => {
          srcSig.set(s)
        },
      })

      return () => {
        stopWaveformGate()
        revokeCreated()
      }
    })

    return { srcSig, kindOverride, isVideo, showWaveform, mediaWire, waveform }
  })

</controller>
{#if isVideo()}
  <video use:mediaWire sx={style.display(block).w(full).radius(md).bg(surface-1).borderWidth(1).border(border).shadow(xs)}></video>
{:else}
  <audio use:mediaWire sx={style.display(block).w(full).radius(md).bg(surface-1).borderWidth(1).border(border).shadow(xs)}></audio>
  {#if showWaveform()}
    <canvas aria-hidden="true" use:waveform sx={style.display(block).w(full).mt(2)}></canvas>
  {/if}
{/if}
