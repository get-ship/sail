<!--
  /showcase — SHANTY, a complete music player in one .sail file.

  Not a demo grid: a full app. Three panes (library / playlist / now-playing
  bar), a real playback engine (position ticks on an effect whose cleanup IS
  the pause button), queue advance with shuffle+repeat, liked tracks, a
  filterable library, and an equalizer that exists in the DOM only while
  playing. The frame wears any theme via theme — the markup
  says what the app is, never what it looks like.
-->

<controller>
import { interval, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

interface Track {
  id: number
  title: string
  artist: string
  album: string
  secs: number
  len: string
  /** Same-origin album art under /shanty/ (public). */
  cover: string
}
interface Playlist { id: string; name: string; tracks: number[] }

// Seven album covers from Downloads → public/cover-0N.webp (deck serves single-segment public files).
const TRACKS: Track[] = [
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
    // lifecycle interval clears — same as unmount.
    effect(() => {
      if (!playing()) return
      interval(() => {
        const total = current().secs
        if (position() + 1 >= total) advance(1)
        else position.set((p) => p + 1)
      }, 1000)
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

<!--
  Fill remaining viewport under sticky site chrome (`h:fill` + use:chromeOffset
  on the navbar). Row defaults to align=center — app panes need stretch so the
  library rail + track list occupy the body instead of floating in dead space.
-->
<column theme={theme()} gap="0">
  style({ box: h(fill).minH(0).overflow(hidden), paint: bg(background).fg(foreground) })

  <!-- ── app chrome: brand + theme picker ─────────────────────────────── -->
  <row align="center" gap="3">
    style({  box: shrink(1).pad(3).padX(5), paint: borderBWidth(1).border(border)  })

    <strong>
      style({ paint: fg(primary), text: fontFamily(mono).fontSize(lg) })
shanty</strong>
    <p>
      style({ box: grow(1), paint: fg(muted-foreground), text: fontSize(sm) })
a full app in one .sail file</p>
    <row gap="1">
      <for {THEMES as t} key={t}>
        <button on:click={() => theme.set(t)} variant={theme() === t ? 'solid' : 'soft'} tone="primary" size="sm">{t}</button>
      </for>
    </row>
  </row>

  <row align="stretch" gap="0">
    style({  box: grow(1).minH(0)  })

    <!-- ── library rail ───────────────────────────────────────────────── -->
    <column gap="3">
      style({ box: w(64).pad(4).minH(0).overflowY(auto), paint: borderRWidth(1).border(border) })

      <input placeholder="Filter playlists" bind:value={query} />
      <column gap="1">
        <p>
          style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })
Playlists</p>
        <for {filteredPlaylists() as p} key={p.id}>
          <row on:click={() => activePlaylist.set(p.id)}
               current={activePlaylist() === p.id ? 'true' : null}
 align="center" justify="between">
            style({ box: pad(2), paint: radius(md).cursor(pointer) }).when(hover, { paint: bg(muted) }).when(current, { paint: bg(muted) })

            <span>{p.name}</span>
            <badge tone="muted">{p.tracks.length}</badge>
          </row>
        </for>
      </column>
      <separator></separator>
      <row on:click={() => activePlaylist.set('__liked')}
           current={activePlaylist() === '__liked' ? 'true' : null}
 align="center" justify="between">
        style({ box: pad(2), paint: radius(md).cursor(pointer) }).when(hover, { paint: bg(muted) }).when(current, { paint: bg(muted) })

        <span>♥ Liked</span>
        <badge tone="primary">{liked().length}</badge>
      </row>
    </column>

    <!-- ── playlist main ──────────────────────────────────────────────── -->
    <column gap="0">
      style({  box: grow(1).minW(0).minH(0)  })

      <column gap="1">
        style({  box: shrink(1).pad(6).padB(4), animate: enter({ opacity: 'fade', translate-y: 'spring:gentle' })  })

        <p>
          style({ paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })
Playlist</p>
        <h1>
          style({ text: fontSize(3xl) })
{activePlaylist() === '__liked' ? 'Liked Tracks' : playlist().name}</h1>
        <p>
          style({ paint: fg(muted-foreground), text: fontSize(sm) })

          {trackCount()} tracks · click a row to play
        </p>
      </column>
      <scroll-area>
        style({  box: grow(1).minH(0).padX(6).padB(4)  })

        <column gap="1">
          <for {visibleTracks() as t} key={t.id}>
            <row on:click={() => playTrack(t.id)}
                 current={currentId() === t.id ? 'true' : null}

                 align="center" gap="3">
              style({ box: pad(2).padX(3), paint: radius(md).cursor(pointer), animate: enter({ opacity: 'fade', stagger: '0.03s' }) }).when(hover, { paint: bg(muted) }).when(current, { paint: bg(muted) })

              <div>
                style({  box: shrink(0).position(relative).w(10).h(10).overflow(hidden), paint: radius(sm).bg(muted)  })

                <img src={t.cover} alt="">
                  style({ box: display(block).w(full).h(full), paint: objectFit(cover) })
                </img>
                <if {currentId() === t.id && playing()}>
                  <center>
                    style({ box: position(absolute).inset(0), paint: bg(background).opacity(70) })

                    <row gap="1" align="end" justify="center">
                      style({ box: minH(4) })

                      <i>
                        style({ box: w(1).h(3), paint: radius(full).bg(primary), animate: loop({ translate-y: 'spring:gentle', stagger: '0.08s' }) })
</i>
                      <i>
                        style({ box: w(1).h(4), paint: radius(full).bg(primary), animate: loop({ translate-y: 'spring:gentle', stagger: '0.08s' }) })
</i>
                      <i>
                        style({ box: w(1).h(2), paint: radius(full).bg(primary), animate: loop({ translate-y: 'spring:gentle', stagger: '0.08s' }) })
</i>
                    </row>
                  </center>
                </if>
              </div>
              <column gap="0">
                style({ box: grow(1).minW(0) })

                <strong>
                  when({ groupCurrent: style({ paint: fg(primary) }) })
{t.title}</strong>
                <p>
                  style({ paint: fg(muted-foreground), text: fontSize(sm) })
{t.artist}</p>
              </column>
              <p>
                style({ box: display(none), paint: fg(muted-foreground), text: fontSize(sm) }).when(desktop, { box: display(block) })
{t.album}</p>
              <button on:click={(e) => { e.stopPropagation(); toggleLike(t.id) }}
                    variant="ghost" size="sm" tone={liked().includes(t.id) ? 'primary' : 'default'}>
                ♥
              </button>
              <span>
                style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(sm) })
{t.len}</span>
            </row>
          </for>
        </column>
      </scroll-area>
    </column>
  </row>

  <!-- ── now-playing bar ──────────────────────────────────────────────── -->
  <column gap="2">
    style({  box: shrink(1).pad(3).padX(5), paint: borderTWidth(1).border(border)  })

    <row align="center" gap="4">
      <row align="center" gap="3">
        style({ box: grow(1).minW(0) })

        <div>
          style({  box: shrink(0).w(12).h(12).overflow(hidden), paint: radius(md).bg(muted).borderWidth(1).border(border)  })

          <img src={current().cover} alt="">
            style({ box: display(block).w(full).h(full), paint: objectFit(cover) })
          </img>
        </div>
        <column gap="0">
          style({ box: minW(0) })

          <strong>{current().title}</strong>
          <p>
            style({ paint: fg(muted-foreground), text: fontSize(sm) })
{current().artist} · {current().album}</p>
        </column>
      </row>
      <column align="center" gap="1">
        style({ box: grow(1) })

        <row gap="2" align="center">
          <button on:click={() => shuffle.set(!shuffle())} variant={shuffle() ? 'solid' : 'ghost'} size="sm">shuffle</button>
          <button on:click={() => advance(-1)} variant="ghost">⏮</button>
          <button on:click={togglePlay} variant="solid" tone="primary">
            style({ animate: press({ scale: 'press' }) })

            {playing() ? '⏸' : '▶'}
          </button>
          <button on:click={() => advance(1)} variant="ghost">⏭</button>
          <button on:click={() => repeat.set(!repeat())} variant={repeat() ? 'solid' : 'ghost'} size="sm">repeat</button>
        </row>
        <row gap="2" align="center" nowrap>
          style({ box: w(full).maxW(panel-max-w-md) })

          <span>
            style({ box: shrink(1), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
{positionLabel()}</span>
          <!-- progress host paints w-full; grow shell owns the flex share so times stay on one line -->
          <row>
            style({  box: grow(1).minW(0)  })

            <progress value={position()} max={current().secs}></progress>
          </row>
          <span>
            style({ box: shrink(1), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
{totalLabel()}</span>
        </row>
      </column>
      <row gap="2" align="center" justify="end" nowrap>
        style({ box: grow(1).minW(0) })

        <button on:click={() => volume.set(Math.max(0, volume() - 10))} variant="ghost" size="sm">−</button>
        <div>
          style({ box: w(20).minW(20) })

          <progress value={volume()} max="100"></progress>
        </div>
        <button on:click={() => volume.set(Math.min(100, volume() + 10))} variant="ghost" size="sm">+</button>
      </row>
    </row>
  </column>
</column>
