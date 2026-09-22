<!--
  <sail-media-av> — video/audio ViewerRenderer arm. Source via `src` identity
  (never attribute, never fetch). Native <video>/<audio controls>; optional
  decorative waveform canvas beside accessible <audio>.
-->

<scope>
  props: {
    kind: 'auto' | 'video' | 'audio' = 'auto'
    autoplay = false
    controls = true
    waveform = false
  }
  style mediaChrome = { display(block).size({ w: 100% }).radius(md).bg(surface).border({ color: border, width: 1 }).elevation(xs) }
</scope>

<controller>
import { emit } from '@ship/sail-runtime/internal'
import type { Action } from '@ship/sail-runtime'
import { emitViewerError, emitViewerLoad } from '../engine/renderer-contract'
import { decodePeaks, drawWaveform } from './engine/waveform'
import { bytesOf, contentTypeOf, mediaUrlOf, type MediaSrc } from './media-src'
import type { Scope } from './sail-media-av.sail'

export default controller((_ctx: Scope) => {
    const srcSig = signal<MediaSrc>(null)
    // kind override read off host attribute (enum not always through Scope).
    const kindOverride = signal<string>('auto')

    const isVideo = derived<boolean>(() => {
      const k = kindOverride()
      if (k === 'video') return true
      if (k === 'audio') return false
      const ct = contentTypeOf(srcSig())
      if (ct.startsWith('audio/')) return false
      if (ct.startsWith('video/')) return true
      return true
    })

    const showWaveform = signal<boolean>(false)

    let hostEl: HostBag | null = null

    const wantsControls = (): boolean => hostGetAttr(hostEl, 'controls') !== 'false'
    const wantsAutoplay = (): boolean =>
      hostHasAttr(hostEl, 'autoplay') === true && hostGetAttr(hostEl, 'autoplay') !== 'false'

    let created: string[] = []
    const revokeCreated = (): void => {
      for (const u of created.splice(0)) revokeObjectUrl(u)
    }
    let lastUrl = ''

    // Action lifecycle replaces dual-ref + identity rebind when video/audio swaps.
    const mediaWire: Action<void> = (node) => {
      const el = node as HostBag
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
        if (wantsControls()) hostSetAttr(el as HostBag, 'controls', '')
        else hostSetAttr(el as HostBag, 'controls', null)
        if (wantsAutoplay()) hostSetAttr(el as HostBag, 'autoplay', '')

        const { url, created: didCreate } = mediaUrlOf(srcSig())
        if (didCreate && url !== lastUrl) created.push(url)
        lastUrl = url
        if (url) hostSetAttr(el as HostBag, 'src', url)
        else hostSetAttr(el as HostBag, 'src', null)
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

    const waveform: Action<void> = (node) => {
      const canvas = node as HostBag
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
      kindOverride.set(hostGetAttr(host, 'kind') ?? 'auto')

      const waveformOn =
        hostHasAttr(host, 'waveform') && hostGetAttr(host, 'waveform') !== 'false'
      const stopWaveformGate = effect(() => {
        showWaveform.set(waveformOn && !isVideo())
      })

      defineHostProperty(asSailHost(host) as HostBag, 'src', {
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
<if {isVideo()}>
  <video use:mediaWire style(mediaChrome)></video>
{else => <audio use:mediaWire style(mediaChrome)></audio>
  <if {showWaveform()}>
    <canvas aria-hidden="true" use:waveform style({ display(block).size({ w: 100% }).margin({ t: 2 }) })></canvas>
  </if>}</if>
