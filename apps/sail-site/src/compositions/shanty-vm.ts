// SHANTY showcase playback + library graph. Controller wires interval/signals only.
// Nested frame mode stamping uses a handed host via `use:bindFrame` + the
// tokens theme-change subscription API (no ambient document lookup).

import type { Action } from '@ship/sail-runtime'
import { applyNestedTheme, resolveNestedPaintMode, subscribeThemeChange } from '../vendor/tokens-themes'

import { signal, derived, effect, interval } from '../vendor/runtime'
export interface Track {
  id: number
  title: string
  artist: string
  album: string
  secs: number
  len: string
  /** Same-origin album art under /shanty/ (public). */
  cover: string
}

export interface Playlist {
  id: string
  name: string
  tracks: number[]
}

export const TRACKS: Track[] = [
  { id: 1, title: 'Signal in the Rain', artist: 'The Reactive Quartet', album: 'Derived State', secs: 214, len: '3:34', cover: '/cover-01.webp' },
  { id: 2, title: 'Mount and Destroy', artist: 'Lifecycle', album: 'The Interval', secs: 187, len: '3:07', cover: '/cover-02.webp' },
  { id: 3, title: 'One File, One Tag', artist: 'SAIL House Band', album: 'Convention', secs: 243, len: '4:03', cover: '/cover-03.webp' },
  { id: 4, title: 'Spring Gentle', artist: 'Motion Tokens', album: 'Presets', secs: 156, len: '2:36', cover: '/cover-04.webp' },
  { id: 5, title: 'The Refusal', artist: 'Strict Mode', album: 'Compile Errors', secs: 199, len: '3:19', cover: '/cover-05.webp' },
  { id: 6, title: 'Closed Set Blues', artist: 'Strict Mode', album: 'Compile Errors', secs: 231, len: '3:51', cover: '/cover-05.webp' },
  { id: 7, title: 'Hydrate', artist: 'Deck Hands', album: 'SSR', secs: 178, len: '2:58', cover: '/cover-06.webp' },
  { id: 8, title: 'Eight-Bit Hull', artist: 'Chiptune Registry', album: 'Pixel Wake', secs: 142, len: '2:22', cover: '/cover-07.webp' },
  { id: 9, title: 'Neon Brine', artist: 'Synthwave Dockyard', album: 'Backlit Acrylic', secs: 265, len: '4:25', cover: '/cover-02.webp' },
  { id: 10, title: 'Quiet Substrate', artist: 'The Reactive Quartet', album: 'Derived State', secs: 208, len: '3:28', cover: '/cover-01.webp' },
]

export const PLAYLISTS: Playlist[] = [
  { id: 'harbor', name: 'Harbor Signals', tracks: [1, 4, 5, 10] },
  { id: 'shipyard', name: 'Shipyard Nights', tracks: [2, 3, 7] },
  { id: 'neon', name: 'Neon Wake', tracks: [8, 9, 4] },
  { id: 'strict', name: 'Strictly Compiled', tracks: [5, 6, 3] },
]

export const SHANTY_THEMES = ['sail', 'brut', '8bit', 'synthwave', 'paper', 'console', 'glass']

const fmt = (s: number): string =>
  `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, '0')}`

export function createShanty() {
  const activePlaylist = signal('harbor')
  const query = signal('')
  const filteredPlaylists = derived(() =>
    PLAYLISTS.filter((p) => p.name.toLowerCase().includes(query().toLowerCase())),
  )
  const playlist = derived(() => PLAYLISTS.find((p) => p.id === activePlaylist()) ?? PLAYLISTS[0]!)
  const tracks = derived(() => playlist().tracks.map((id) => TRACKS.find((t) => t.id === id)!))

  const currentId = signal(TRACKS[0]!.id)
  const playing = signal(false)
  const position = signal(0)
  const shuffle = signal(false)
  const repeat = signal(false)
  const current = derived(() => TRACKS.find((t) => t.id === currentId()) ?? TRACKS[0]!)
  const positionLabel = derived(() => fmt(position()))
  const totalLabel = derived(() => fmt(current().secs))

  const advance = (dir: 1 | -1): void => {
    const list = tracks()
    const i = list.findIndex((t) => t.id === currentId())
    let next: Track | undefined
    if (shuffle() && dir === 1) {
      next = list[Math.floor(Math.random() * list.length)]
    } else {
      next = list[(i + dir + list.length) % list.length]
    }
    if (!next && repeat()) next = list[0]
    if (next) {
      currentId.set(next.id)
      position.set(0)
    } else {
      playing.set(false)
    }
  }

  // Clock: effect tracks playing(); pause re-runs and interval cleanup is the stop.
  effect(() => {
    if (!playing()) return
    interval(() => {
      const total = current().secs
      if (position() + 1 >= total) advance(1)
      else position.set((p) => p + 1)
    }, 1000)
  })

  const playTrack = (id: number): void => {
    currentId.set(id)
    position.set(0)
    playing.set(true)
  }
  const togglePlay = (): void => playing.set((p) => !p)

  const liked = signal<number[]>([1, 9])
  const toggleLike = (id: number): void =>
    liked.set((l) => (l.includes(id) ? l.filter((x) => x !== id) : [...l, id]))
  const likedTracks = derived(() => TRACKS.filter((t) => liked().includes(t.id)))
  const visibleTracks = derived(() =>
    activePlaylist() === '__liked' ? likedTracks() : tracks(),
  )
  const trackCount = derived(() => visibleTracks().length)

  const volume = signal(70)
  const theme = signal('sail')

  // Nested frame: theme + mode (C-TH-NEST-1). Template binds theme=;
  // applyNestedTheme stamps data-mode (data-* banned in app .sail markup).
  // Host is handed via use:bindFrame — never ambient getElementById.
  let frameHost: HTMLElement | null = null

  const stampFrameMode = (id: string): boolean => {
    if (!frameHost) return false
    applyNestedTheme(id, frameHost)
    return true
  }

  /** Stamp now, or rAF-retry until the frame host attaches. */
  const stampFrameModeSoon = (id: string): void => {
    if (stampFrameMode(id)) return
    if (typeof requestAnimationFrame !== 'function') return
    requestAnimationFrame(() => {
      if (!stampFrameMode(id)) requestAnimationFrame(() => stampFrameMode(id))
    })
  }

  effect(() => {
    stampFrameModeSoon(theme())
  })

  /**
   * Action: capture the nested theme host and subscribe to product theme
   * changes via the tokens runtime API (not ambient document listeners).
   */
  const bindFrame: Action<void> = (node) => {
    frameHost = node as HTMLElement
    stampFrameMode(theme())
    const unsub = subscribeThemeChange(() => stampFrameModeSoon(theme()))
    return {
      destroy: () => {
        unsub()
        if (frameHost === node) frameHost = null
      },
    }
  }

  const setTheme = (id: string): void => {
    theme.set(id)
    // Immediate stamp so chip click paints before the next microtask.
    stampFrameModeSoon(id)
  }

  return {
    query,
    filteredPlaylists,
    activePlaylist,
    playlist,
    tracks,
    visibleTracks,
    trackCount,
    currentId,
    playing,
    position,
    current,
    advance,
    playTrack,
    togglePlay,
    positionLabel,
    totalLabel,
    shuffle,
    repeat,
    liked,
    toggleLike,
    likedTracks,
    volume,
    theme,
    setTheme,
    bindFrame,
    THEMES: SHANTY_THEMES,
    // Expose for tests / debugging without re-importing tokens.
    nestedMode: () => resolveNestedPaintMode(theme()),
  }
}
