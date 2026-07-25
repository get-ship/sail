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

<column data-theme-preview={theme()} sx={style.h(full).minH(screen).bg(background).fg(foreground)}>
  <!-- ── app chrome: brand + theme picker ─────────────────────────────── -->
  <row sx={style.pad(3).padX(5).borderBWidth(1).border(border)} align="center" gap="3">
    <strong sx={style.fontFamily(mono).fontSize(lg).fg(primary)}>shanty</strong>
    <p sx={style.fontSize(sm).fg(muted-foreground).grow(1)}>a full app in one .sail file</p>
    <row gap="1">
      {#for t of THEMES key={t}}
        <button on:click={() => theme.set(t)} variant={theme() === t ? 'solid' : 'outline'} size="sm">{t}</button>
      {/for}
    </row>
  </row>

  <row sx={style.grow(1).minH(0)} gap="0">
    <!-- ── library rail ───────────────────────────────────────────────── -->
    <column sx={style.w(64).pad(4).borderRWidth(1).border(border)} gap="3">
      <input placeholder="Filter playlists" bind:value={query} />
      <column gap="1">
        <p sx={style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(muted-foreground)}>Playlists</p>
        {#for p of filteredPlaylists() key={p.id}}
          <row on:click={() => activePlaylist.set(p.id)}
               aria-current={activePlaylist() === p.id ? 'true' : null}
               sx={style.pad(2).radius(md).cursor(pointer).when({ hover: style.bg(muted), current: style.bg(muted) })} align="center" justify="between">
            <span>{p.name}</span>
            <badge tone="muted">{p.tracks.length}</badge>
          </row>
        {/for}
      </column>
      <separator></separator>
      <row on:click={() => activePlaylist.set('__liked')}
           aria-current={activePlaylist() === '__liked' ? 'true' : null}
           sx={style.pad(2).radius(md).cursor(pointer).when({ hover: style.bg(muted), current: style.bg(muted) })} align="center" justify="between">
        <span>♥ Liked</span>
        <badge tone="primary">{liked().length}</badge>
      </row>
    </column>

    <!-- ── playlist main ──────────────────────────────────────────────── -->
    <column sx={style.grow(1).minW(0)} gap="0">
      <column sx={style.pad(6).padB(4)} gap="1" motion:enter={{ opacity: 'fade', translate-y: 'spring:gentle' }}>
        <p sx={style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent)}>Playlist</p>
        <h1 sx={style.fontSize(3xl)}>{activePlaylist() === '__liked' ? 'Liked Tracks' : playlist().name}</h1>
        <p sx={style.fontSize(sm).fg(muted-foreground)}>
          {trackCount()} tracks · click a row to play
        </p>
      </column>
      <scroll-area sx={style.grow(1).minH(0).padX(6).padB(4)}>
        <column gap="1">
          {#for t of visibleTracks() key={t.id}}
            <row on:click={() => playTrack(t.id)}
                 aria-current={currentId() === t.id ? 'true' : null}
                 sx={style.pad(2).padX(3).radius(md).cursor(pointer).when({ hover: style.bg(muted), current: style.bg(muted) })}
                 align="center" gap="3"
                 motion:enter={{ opacity: 'fade', stagger: '0.03s' }}>
              <row justify="center" sx={style.w(6).shrink(0).fontFamily(mono).fontSize(sm).fg(muted-foreground)}>
                {#if currentId() === t.id && playing()}
                  <row gap="1" align="end" justify="center" sx={style.minH(4)}>
                    <i sx={style.w(1).h(3).radius(full).bg(primary)} motion:loop={{ translate-y: 'spring:gentle', stagger: '0.08s' }}></i>
                    <i sx={style.w(1).h(4).radius(full).bg(primary)} motion:loop={{ translate-y: 'spring:gentle', stagger: '0.08s' }}></i>
                    <i sx={style.w(1).h(2).radius(full).bg(primary)} motion:loop={{ translate-y: 'spring:gentle', stagger: '0.08s' }}></i>
                  </row>
                {:else}
                  {t.id}
                {/if}
              </row>
              <column gap="0" sx={style.grow(1).minW(0)}>
                <strong sx={when({ groupCurrent: style.fg(primary) })}>{t.title}</strong>
                <p sx={style.fontSize(sm).fg(muted-foreground)}>{t.artist}</p>
              </column>
              <p sx={style.fontSize(sm).fg(muted-foreground).display(none).when({ desktop: style.display(block) })}>{t.album}</p>
              <button on:click={(e) => { e.stopPropagation(); toggleLike(t.id) }}
                    variant="ghost" size="sm" tone={liked().includes(t.id) ? 'primary' : 'default'}>
                ♥
              </button>
              <span sx={style.fontFamily(mono).fontSize(sm).fg(muted-foreground)}>{t.len}</span>
            </row>
          {/for}
        </column>
      </scroll-area>
    </column>
  </row>

  <!-- ── now-playing bar ──────────────────────────────────────────────── -->
  <column sx={style.borderTWidth(1).border(border).pad(3).padX(5)} gap="2">
    <row align="center" gap="4">
      <row align="center" gap="3" sx={style.minW(0).grow(1)}>
        <div sx={style.w(11).h(11).radius(md).bg(muted).borderWidth(1).border(primary).shrink(0)}>
          <center sx={style.h(full)}><strong sx={style.fontSize(lg).fg(primary)}>♪</strong></center>
        </div>
        <column gap="0" sx={style.minW(0)}>
          <strong>{current().title}</strong>
          <p sx={style.fontSize(sm).fg(muted-foreground)}>{current().artist}</p>
        </column>
      </row>
      <column align="center" gap="1" sx={style.grow(1)}>
        <row gap="2" align="center">
          <button on:click={() => shuffle.set(!shuffle())} variant={shuffle() ? 'solid' : 'ghost'} size="sm">shuffle</button>
          <button on:click={() => advance(-1)} variant="ghost">⏮</button>
          <button on:click={togglePlay} variant="solid" tone="primary" motion:press={{ scale: 'press' }}>
            {playing() ? '⏸' : '▶'}
          </button>
          <button on:click={() => advance(1)} variant="ghost">⏭</button>
          <button on:click={() => repeat.set(!repeat())} variant={repeat() ? 'solid' : 'ghost'} size="sm">repeat</button>
        </row>
        <row gap="2" align="center" sx={style.w(full).maxW(panel-max-w-md)}>
          <span sx={style.fontFamily(mono).fontSize(xs).fg(muted-foreground)}>{positionLabel()}</span>
          <progress value={position()} max={current().secs} sx={style.grow(1)}></progress>
          <span sx={style.fontFamily(mono).fontSize(xs).fg(muted-foreground)}>{totalLabel()}</span>
        </row>
      </column>
      <row gap="2" align="center" justify="end" sx={style.grow(1)}>
        <button on:click={() => volume.set(Math.max(0, volume() - 10))} variant="ghost" size="sm">−</button>
        <progress value={volume()} max="100" sx={style.w(20)}></progress>
        <button on:click={() => volume.set(Math.min(100, volume() + 10))} variant="ghost" size="sm">+</button>
      </row>
    </row>
  </column>
</column>
