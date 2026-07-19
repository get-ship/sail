<!--
  /showcase — SHANTY, a complete music player in one .sail file.

  Not a demo grid: a full app. Three panes (library / playlist / now-playing
  bar), a real playback engine (position ticks on an effect whose cleanup IS
  the pause button), queue advance with shuffle+repeat, liked tracks, a
  filterable library, and an equalizer that exists in the DOM only while
  playing. The frame wears any theme via data-theme-preview — the markup
  says what the app is, never what it looks like.
-->

<controller>
import { controller, signal, derived, effect, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

interface Track { id: number; title: string; artist: string; album: string; secs: number; len: string }
interface Playlist { id: string; name: string; tracks: number[] }

const TRACKS: Track[] = [
  { id: 1, title: 'Signal in the Rain', artist: 'The Reactive Quartet', album: 'Derived State', secs: 214, len: '3:34' },
  { id: 2, title: 'Mount and Destroy', artist: 'Lifecycle', album: 'The Interval', secs: 187, len: '3:07' },
  { id: 3, title: 'One File, One Tag', artist: 'SAIL House Band', album: 'Convention', secs: 243, len: '4:03' },
  { id: 4, title: 'Spring Gentle', artist: 'Motion Tokens', album: 'Presets', secs: 156, len: '2:36' },
  { id: 5, title: 'The Refusal', artist: 'Strict Mode', album: 'Compile Errors', secs: 199, len: '3:19' },
  { id: 6, title: 'Closed Set Blues', artist: 'Strict Mode', album: 'Compile Errors', secs: 231, len: '3:51' },
  { id: 7, title: 'Hydrate', artist: 'Deck Hands', album: 'SSR', secs: 178, len: '2:58' },
  { id: 8, title: 'Eight-Bit Hull', artist: 'Chiptune Registry', album: 'Pixel Wake', secs: 142, len: '2:22' },
  { id: 9, title: 'Neon Brine', artist: 'Synthwave Dockyard', album: 'Backlit Acrylic', secs: 265, len: '4:25' },
  { id: 10, title: 'Quiet Substrate', artist: 'The Reactive Quartet', album: 'Derived State', secs: 208, len: '3:28' },
]

const PLAYLISTS: Playlist[] = [
  { id: 'harbor', name: 'Harbor Signals', tracks: [1, 4, 5, 10] },
  { id: 'shipyard', name: 'Shipyard Nights', tracks: [2, 3, 7] },
  { id: 'neon', name: 'Neon Wake', tracks: [8, 9, 4] },
  { id: 'strict', name: 'Strictly Compiled', tracks: [5, 6, 3] },
]

const THEMES = ['sail', 'brut', '8bit', 'synthwave', 'paper', 'console', 'glass']

const fmt = (s: number): string => `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, '0')}`

export default controller((_ctx: ControllerScope<Scope>) => {
    // ── library ──────────────────────────────────────────────────────────
    const activePlaylist = signal('harbor')
    const query = signal('')
    const filteredPlaylists = derived(() =>
      PLAYLISTS.filter((p) => p.name.toLowerCase().includes(query().toLowerCase())),
    )
    const playlist = derived(() => PLAYLISTS.find((p) => p.id === activePlaylist()) ?? PLAYLISTS[0]!)
    const tracks = derived(() => playlist().tracks.map((id) => TRACKS.find((t) => t.id === id)!))

    // ── playback engine ──────────────────────────────────────────────────
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
      if (next) { currentId.set(next.id); position.set(0) }
      else playing.set(false)
    }

    // The clock: an effect that TRACKS playing(). Pause re-runs it and the
    // cleanup stops the interval — the same cleanup that runs on unmount.
    effect(() => {
      if (!playing()) return
      const t = setInterval(() => {
        const total = current().secs
        if (position() + 1 >= total) advance(1)
        else position.set((p) => p + 1)
      }, 1000)
      return () => clearInterval(t)
    })

    const playTrack = (id: number): void => {
      currentId.set(id); position.set(0); playing.set(true)
    }
    const togglePlay = (): void => playing.set((p) => !p)

    // ── likes + volume + theme ───────────────────────────────────────────
    const liked = signal<number[]>([1, 9])
    const toggleLike = (id: number): void =>
      liked.set((l) => (l.includes(id) ? l.filter((x) => x !== id) : [...l, id]))
    const likedTracks = derived(() => TRACKS.filter((t) => liked().includes(t.id)))
    const visibleTracks = derived(() => (activePlaylist() === '__liked' ? likedTracks() : tracks()))
    const trackCount = derived(() => visibleTracks().length)

    const volume = signal(70)
    const theme = signal('sail')

    return {
      query, filteredPlaylists, activePlaylist, playlist, tracks, visibleTracks, trackCount,
      currentId, playing, position, current, advance, playTrack, togglePlay, positionLabel, totalLabel,
      shuffle, repeat, liked, toggleLike, likedTracks,
      volume, theme, THEMES,
    }
})
</controller>

<column data-theme-preview={theme()} sx="h-full min-h-screen bg-background text-foreground">
  <!-- ── app chrome: brand + theme picker ─────────────────────────────── -->
  <row sx="p-3 px-5 border-b border-border" align="center" gap="3">
    <strong sx="font-mono text-lg text-primary">shanty</strong>
    <p sx="text-sm text-muted-foreground grow-1">a full app in one .sail file</p>
    <row gap="1">
      {#for t of THEMES key={t}}
        <button on:click={() => theme.set(t)} variant={theme() === t ? 'solid' : 'outline'} size="sm">{t}</button>
      {/for}
    </row>
  </row>

  <row sx="grow-1 min-h-0" gap="0">
    <!-- ── library rail ───────────────────────────────────────────────── -->
    <column sx="w-64 p-4 border-r border-border" gap="3">
      <input placeholder="Filter playlists" bind:value={query} />
      <column gap="1">
        <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground">Playlists</p>
        {#for p of filteredPlaylists() key={p.id}}
          <row on:click={() => activePlaylist.set(p.id)}
               aria-current={activePlaylist() === p.id ? 'true' : null}
               sx="p-2 rounded-md cursor-pointer hover:bg-muted current:bg-muted" align="center" justify="between">
            <span>{p.name}</span>
            <badge tone="muted">{p.tracks.length}</badge>
          </row>
        {/for}
      </column>
      <separator></separator>
      <row on:click={() => activePlaylist.set('__liked')}
           aria-current={activePlaylist() === '__liked' ? 'true' : null}
           sx="p-2 rounded-md cursor-pointer hover:bg-muted current:bg-muted" align="center" justify="between">
        <span>♥ Liked</span>
        <badge tone="primary">{liked().length}</badge>
      </row>
    </column>

    <!-- ── playlist main ──────────────────────────────────────────────── -->
    <column sx="grow-1 min-w-0" gap="0">
      <column sx="p-6 pb-4" gap="1" motion:enter={{ opacity: 'fade', translate-y: 'spring:gentle' }}>
        <p sx="font-mono text-xs uppercase tracking-wide text-accent">Playlist</p>
        <h1 sx="text-3xl">{activePlaylist() === '__liked' ? 'Liked Tracks' : playlist().name}</h1>
        <p sx="text-sm text-muted-foreground">
          {trackCount()} tracks · click a row to play
        </p>
      </column>
      <scroll-area sx="grow-1 min-h-0 px-6 pb-4">
        <column gap="1">
          {#for t of visibleTracks() key={t.id}}
            <row on:click={() => playTrack(t.id)}
                 aria-current={currentId() === t.id ? 'true' : null}
                 sx="p-2 px-3 rounded-md cursor-pointer hover:bg-muted current:bg-muted"
                 align="center" gap="3"
                 motion:enter={{ opacity: 'fade', stagger: '0.03s' }}>
              <row justify="center" sx="w-6 shrink-0 font-mono text-sm text-muted-foreground">
                {#if currentId() === t.id && playing()}
                  <row gap="1" align="end" justify="center" sx="min-h-4">
                    <i sx="w-1 h-3 rounded-full bg-primary" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.08s' }}></i>
                    <i sx="w-1 h-4 rounded-full bg-primary" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.08s' }}></i>
                    <i sx="w-1 h-2 rounded-full bg-primary" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.08s' }}></i>
                  </row>
                {:else}
                  {t.id}
                {/if}
              </row>
              <column gap="0" sx="grow-1 min-w-0">
                <strong sx="group-current:text-primary">{t.title}</strong>
                <p sx="text-sm text-muted-foreground">{t.artist}</p>
              </column>
              <p sx="text-sm text-muted-foreground hidden desktop:block">{t.album}</p>
              <button on:click={(e) => { e.stopPropagation(); toggleLike(t.id) }}
                    variant="ghost" size="sm" tone={liked().includes(t.id) ? 'primary' : 'default'}>
                ♥
              </button>
              <span sx="font-mono text-sm text-muted-foreground">{t.len}</span>
            </row>
          {/for}
        </column>
      </scroll-area>
    </column>
  </row>

  <!-- ── now-playing bar ──────────────────────────────────────────────── -->
  <column sx="border-t border-border p-3 px-5" gap="2">
    <row align="center" gap="4">
      <row align="center" gap="3" sx="min-w-0 grow-1">
        <div sx="w-11 h-11 rounded-md bg-tone-primary-bg border border-primary shrink-0">
          <center sx="h-full"><strong sx="text-lg text-primary">♪</strong></center>
        </div>
        <column gap="0" sx="min-w-0">
          <strong>{current().title}</strong>
          <p sx="text-sm text-muted-foreground">{current().artist}</p>
        </column>
      </row>
      <column align="center" gap="1" sx="grow-1">
        <row gap="2" align="center">
          <button on:click={() => shuffle.set(!shuffle())} variant={shuffle() ? 'solid' : 'ghost'} size="sm">shuffle</button>
          <button on:click={() => advance(-1)} variant="ghost">⏮</button>
          <button on:click={togglePlay} variant="solid" tone="primary" motion:press={{ scale: 'press' }}>
            {playing() ? '⏸' : '▶'}
          </button>
          <button on:click={() => advance(1)} variant="ghost">⏭</button>
          <button on:click={() => repeat.set(!repeat())} variant={repeat() ? 'solid' : 'ghost'} size="sm">repeat</button>
        </row>
        <row gap="2" align="center" sx="w-full max-w-panel-max-w-md">
          <span sx="font-mono text-xs text-muted-foreground">{positionLabel()}</span>
          <progress value={position()} max={current().secs} sx="grow-1"></progress>
          <span sx="font-mono text-xs text-muted-foreground">{totalLabel()}</span>
        </row>
      </column>
      <row gap="2" align="center" justify="end" sx="grow-1">
        <button on:click={() => volume.set(Math.max(0, volume() - 10))} variant="ghost" size="sm">−</button>
        <progress value={volume()} max="100" sx="w-20"></progress>
        <button on:click={() => volume.set(Math.min(100, volume() + 10))} variant="ghost" size="sm">+</button>
      </row>
    </row>
  </column>
</column>
