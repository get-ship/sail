// The <sail-media-av> renderer (video/audio + optional waveform).
//
// Harness: `renderComponent` from `@ship/test-setup` compiles the renderer
// `.sui` under the `sail-media-av` tag and registers it as a custom element
// (the new-tag-via-harness path — no WASM rebuild). The media renderer receives
// an already-resolved source via the `prop:src` identity property (a video/audio
// object-url or resolved bytes the consumer fetched) — it NEVER fetches its own
// bytes. happy-dom does not load real media, so lifecycle is driven by
// dispatching the native media events the renderer listens for (`loadedmetadata`,
// `play`, `pause`, `ended`, `error`) — the dispatcher-spec precedent.
//
// The component is LIGHT DOM and hand-authors no classes, so every query below
// is a real semantic: the native <video>/<audio> tag, the aria-hidden <canvas>.
//
// Authority: build-spec §"Props" (sail-media-av), §"Backbone #2 —
// ViewerRenderer contract" (read-only, source-in-never-fetch),
// §"Accessibility" (native <video>/
// <audio controls> carry built-in AT; the waveform canvas is aria-hidden and the
// real <audio> stays the accessible control), and behavioral spec #5 (media
// plays resolved sources; object URLs revoked on unmount).

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import mediaRenderer from '../src/sail-media-av/sail-media-av.sui.controller.ts'

const SOURCE_URL = new URL('../src/sail-media-av/sail-media-av.sui', import.meta.url)
const SOURCE = readFileSync(SOURCE_URL, 'utf8')
// On-disk path so the compiler pairs the controller and runs the
// controller-aware passes (bind:this targets must be `ref(null)`) — the
// harness then rejects exactly what the daemon + CLI do (#263-A).
const SOURCE_PATH = fileURLToPath(SOURCE_URL)

interface ViewerSource {
  contentType?: string
  filename?: string
  bytes?: ArrayBuffer | Uint8Array
  url?: string
}

const videoEl = (el: HTMLElement): HTMLVideoElement | null => el.querySelector('video')
const audioEl = (el: HTMLElement): HTMLAudioElement | null => el.querySelector('audio')
/** The decorative waveform strip — aria-hidden, beside the real <audio>. */
const waveformEl = (el: HTMLElement): HTMLCanvasElement | null =>
  el.querySelector('canvas[aria-hidden="true"]')
/** Whichever native media element the renderer chose for this source. */
const mediaEl = (el: HTMLElement): HTMLMediaElement | null =>
  (videoEl(el) ?? audioEl(el)) as HTMLMediaElement | null

const VIDEO_URL = 'blob:http://localhost/clip.mp4'
const AUDIO_URL = 'blob:http://localhost/track.mp3'

// happy-dom does not implement the object-URL lifecycle; stub it so the
// renderer's create/revoke seam is observable (and deterministic) in the suite.
let createSpy: ReturnType<typeof vi.fn>
let revokeSpy: ReturnType<typeof vi.fn>
beforeEach(() => {
  createSpy = vi.fn(() => 'blob:http://localhost/created')
  revokeSpy = vi.fn()
  ;(URL as unknown as { createObjectURL: unknown }).createObjectURL = createSpy
  ;(URL as unknown as { revokeObjectURL: unknown }).revokeObjectURL = revokeSpy
})

const live: Array<{ unmount(): void }> = []
afterEach(() => {
  for (const v of live.splice(0)) {
    try {
      v.unmount()
    } catch {
      /* best-effort */
    }
  }
  vi.restoreAllMocks()
})

/** Mount the media renderer and assign a live ViewerSource via the `prop:src`
 *  identity property (never a stringified attribute). */
function mountMedia(src: ViewerSource, attrs?: Record<string, string | boolean>) {
  const v = renderComponent(SOURCE, 'sail-media-av', {
    controller: mediaRenderer,
    sourcePath: SOURCE_PATH,
    ...(attrs ? { attrs } : {}),
  })
  ;(v.el as unknown as { src: ViewerSource }).src = src
  return v
}

describe('<sail-media-av> — registration + kind routing', () => {

  it('auto-detects a video source into a <video> element', async () => {
    const v = mountMedia({ contentType: 'video/mp4', url: VIDEO_URL })
    live.push(v)
    await v.flush()
    expect(videoEl(v.el)).not.toBeNull()
    expect(audioEl(v.el)).toBeNull()
  })

  it('auto-detects an audio source into an <audio> element', async () => {
    const v = mountMedia({ contentType: 'audio/mpeg', url: AUDIO_URL })
    live.push(v)
    await v.flush()
    expect(audioEl(v.el)).not.toBeNull()
    expect(videoEl(v.el)).toBeNull()
  })

  it('renders the native element with controls by default (built-in AT)', async () => {
    const v = mountMedia({ contentType: 'audio/mpeg', url: AUDIO_URL })
    live.push(v)
    await v.flush()
    expect(audioEl(v.el)!.hasAttribute('controls')).toBe(true)
  })

  it('points the native element src at the resolved object-url (never fetched)', async () => {
    const v = mountMedia({ contentType: 'video/mp4', url: VIDEO_URL })
    live.push(v)
    await v.flush()
    expect(videoEl(v.el)!.getAttribute('src')).toBe(VIDEO_URL)
  })

  it('respects an explicit kind="audio" override (forces <audio>)', async () => {
    const v = mountMedia({ contentType: 'video/mp4', url: VIDEO_URL }, { kind: 'audio' })
    live.push(v)
    await v.flush()
    expect(audioEl(v.el)).not.toBeNull()
  })
})

describe('<sail-media-av> — props reflect for CSS/AT', () => {
  it('reflects the kind enum onto the host', async () => {
    const v = mountMedia({ contentType: 'video/mp4', url: VIDEO_URL }, { kind: 'video' })
    live.push(v)
    await v.flush()
    expect(v.el.getAttribute('kind')).toBe('video')
  })

  it('exposes no density/dense prop on the renderer (#11)', async () => {
    const v = mountMedia({ contentType: 'audio/mpeg', url: AUDIO_URL })
    live.push(v)
    await v.flush()
    const proto = Object.getPrototypeOf(v.el) as object
    expect(Object.getOwnPropertyNames(proto)).not.toContain('density')
    expect(Object.getOwnPropertyNames(proto)).not.toContain('dense')
  })
})

describe('<sail-media-av> — waveform (#5)', () => {
  it('draws a decorative aria-hidden canvas for audio + waveform', async () => {
    const v = mountMedia({ contentType: 'audio/mpeg', url: AUDIO_URL }, { waveform: '' })
    live.push(v)
    await v.flush()
    const wave = waveformEl(v.el)
    expect(wave).not.toBeNull()
    expect(wave!.getAttribute('aria-hidden')).toBe('true')
  })

  it('keeps the real <audio controls> as the accessible control beside the waveform', async () => {
    const v = mountMedia({ contentType: 'audio/mpeg', url: AUDIO_URL }, { waveform: '' })
    live.push(v)
    await v.flush()
    expect(audioEl(v.el)).not.toBeNull()
    expect(audioEl(v.el)!.hasAttribute('controls')).toBe(true)
  })

  it('draws no waveform canvas when the waveform prop is off', async () => {
    const v = mountMedia({ contentType: 'audio/mpeg', url: AUDIO_URL })
    live.push(v)
    await v.flush()
    expect(waveformEl(v.el)).toBeNull()
  })
})

describe('<sail-media-av> — ViewerRenderer lifecycle events (#5)', () => {
  it('fires sail-viewer-load on the native loadedmetadata event', async () => {
    const v = mountMedia({ contentType: 'video/mp4', url: VIDEO_URL })
    live.push(v)
    await v.flush()
    let loads = 0
    v.el.addEventListener('sail-viewer-load', () => loads++)
    mediaEl(v.el)!.dispatchEvent(new Event('loadedmetadata'))
    await v.flush()
    expect(loads).toBeGreaterThan(0)
  })

  it('the load event bubbles + crosses the shadow boundary (composed)', async () => {
    const v = mountMedia({ contentType: 'video/mp4', url: VIDEO_URL })
    live.push(v)
    await v.flush()
    let caught: CustomEvent | null = null
    v.root.addEventListener('sail-viewer-load', (e) => { caught = e as CustomEvent })
    mediaEl(v.el)!.dispatchEvent(new Event('loadedmetadata'))
    await v.flush()
    expect(caught).not.toBeNull()
  })

  it('fires sail-viewer-play / -pause / -ended from the native transport', async () => {
    const v = mountMedia({ contentType: 'audio/mpeg', url: AUDIO_URL })
    live.push(v)
    await v.flush()
    const seen: string[] = []
    v.el.addEventListener('sail-viewer-play', () => seen.push('play'))
    v.el.addEventListener('sail-viewer-pause', () => seen.push('pause'))
    v.el.addEventListener('sail-viewer-ended', () => seen.push('ended'))
    const m = mediaEl(v.el)!
    m.dispatchEvent(new Event('play'))
    m.dispatchEvent(new Event('pause'))
    m.dispatchEvent(new Event('ended'))
    await v.flush()
    expect(seen).toEqual(['play', 'pause', 'ended'])
  })

  it('fires sail-viewer-error {message} on a failed source', async () => {
    const v = mountMedia({ contentType: 'video/mp4', url: VIDEO_URL })
    live.push(v)
    await v.flush()
    let caught: CustomEvent | null = null
    v.el.addEventListener('sail-viewer-error', (e) => { caught = e as CustomEvent })
    mediaEl(v.el)!.dispatchEvent(new Event('error'))
    await v.flush()
    expect(caught).not.toBeNull()
    expect((caught as unknown as CustomEvent).detail).toHaveProperty('message')
  })
})

describe('<sail-media-av> — object-URL lifecycle (#5)', () => {
  it('creates an object URL from resolved bytes (consumer did not pre-resolve a url)', async () => {
    const v = mountMedia({ contentType: 'video/mp4', bytes: new Uint8Array([0, 0, 0, 1]) })
    live.push(v)
    await v.flush()
    expect(createSpy).toHaveBeenCalled()
  })

  it('revokes the created object URL on unmount (no leak)', async () => {
    const v = mountMedia({ contentType: 'video/mp4', bytes: new Uint8Array([0, 0, 0, 1]) })
    await v.flush()
    expect(createSpy).toHaveBeenCalled()
    v.unmount()
    expect(revokeSpy).toHaveBeenCalled()
  })

  it('does NOT create an object URL when the consumer already supplied a url', async () => {
    const v = mountMedia({ contentType: 'video/mp4', url: VIDEO_URL })
    live.push(v)
    await v.flush()
    expect(createSpy).not.toHaveBeenCalled()
  })
})
