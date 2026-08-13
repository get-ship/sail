<!--
  /showcase — SHANTY, a complete music player in one .sail file.

  Three panes (library / playlist / now-playing bar), playback engine in
  src/compositions/shanty-vm, queue advance with shuffle+repeat, liked tracks,
  filterable library. Frame wears any theme via theme=.
-->

<scope>
  style root = { box: size({ h: fill, min-h: 0 }).overflow({ all: hidden }), paint: fill({ color: background }).ink({ color: foreground }) }
  style chrome = { box: flow({ shrink: 1 }).space({ pad: 3, pad-x: 5 }), paint: border({ color: border, width: 1, edge: b }) }
  style brand = { paint: ink({ color: primary }), text: type({ family: mono, size: lg }) }
  style tagline = { box: flow({ grow: 1 }), paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style body = { box: flow({ grow: 1 }).size({ min-h: 0 }) }
  style rail = { box: size({ w: 64, min-h: 0 }).space({ pad: 4 }).overflow({ y: auto }), paint: border({ color: border, width: 1, edge: r }) }
  style sectionLabel = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style listRow = { box: space({ pad: 2 }), paint: radius(md).cursor(pointer), when: { hover: { paint: fill({ color: muted }) }, current: { paint: fill({ color: muted }) } } }
  style mainCol = { box: flow({ grow: 1 }).size({ min-w: 0, min-h: 0 }) }
  style playlistHead = { box: flow({ shrink: 1 }).space({ pad: 6, pad-b: 4 }), animate: enter({ opacity: 'fade', translate-y: 'spring:gentle' }) }
  style accentLabel = { paint: ink({ color: accent }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style title = { text: type({ size: 3xl }) }
  style mutedSm = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style trackScroll = { box: flow({ grow: 1 }).size({ min-h: 0 }).space({ pad-x: 6, pad-b: 4 }) }
  style trackRow = { box: space({ pad: 2, pad-x: 3 }), paint: radius(md).cursor(pointer), animate: enter({ opacity: 'fade', stagger: '0.03s' }), when: { hover: { paint: fill({ color: muted }) }, current: { paint: fill({ color: muted }) } } }
  style cover = { box: flow({ shrink: 0 }).place({ position: relative }).size(10).overflow({ all: hidden }), paint: radius(sm).fill({ color: muted }) }
  style coverImg = { box: flow({ display: block }).size(full), paint: objectFit(cover) }
  style playingMask = { box: place({ position: absolute, inset: 0 }), paint: fill({ color: background }).filter({ opacity: 70 }) }
  style eqBar = { box: size({ w: 1, h: 3 }), paint: radius(full).fill({ color: primary }), animate: loop({ translate-y: 'spring:gentle', stagger: '0.08s' }) }
  style eqBarMid = { box: size({ w: 1, h: 4 }), paint: radius(full).fill({ color: primary }), animate: loop({ translate-y: 'spring:gentle', stagger: '0.08s' }) }
  style eqBarShort = { box: size({ w: 1, h: 2 }), paint: radius(full).fill({ color: primary }), animate: loop({ translate-y: 'spring:gentle', stagger: '0.08s' }) }
  style trackMeta = { box: flow({ grow: 1 }).size({ min-w: 0 }) }
  style trackTitle = { when: { groupCurrent: { paint: ink({ color: primary }) } } }
  style albumCol = { box: flow({ display: none }), paint: ink({ color: muted-foreground }), text: type({ size: sm }), when: { desktop: { box: flow({ display: block }) } } }
  style len = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: sm }) }
  style nowPlaying = { box: flow({ shrink: 1 }).space({ pad: 3, pad-x: 5 }), paint: border({ color: border, width: 1, edge: t }) }
  style nowCover = { box: flow({ shrink: 0 }).size(12).overflow({ all: hidden }), paint: radius(md).fill({ color: muted }).border({ color: border, width: 1 }) }
  style growMin = { box: flow({ grow: 1 }).size({ min-w: 0 }) }
  style transport = { box: flow({ grow: 1 }) }
  style progressShell = { box: flow({ grow: 1 }).size({ min-w: 0 }) }
  style time = { box: flow({ shrink: 1 }), paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style volumeTrack = { box: size({ w: 20, min-w: 20 }) }
  style pressBtn = { animate: press({ scale: 'press' }) }

  type PlaylistRowProps = {
    name: string
    count: number
    selected: boolean
    tone: string
    onPick: any
  }
  type TrackRowProps = {
    id: number
    title: string
    artist: string
    album: string
    len: string
    cover: string
    isCurrent: boolean
    isPlaying: boolean
    isLiked: boolean
    onPlay: any
    onLike: any
  }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import type { UiEvent } from '../../src/compositions/ui-event'
import { createShanty } from '../../src/compositions/shanty-vm'

export default controller((_ctx: ControllerScope<Scope>) => {
  const s = createShanty()
  // stopPropagation + like in one controller handler (template bans multi-statement).
  const stopAndLike = (e: UiEvent, id: number): void => {
    e.stopPropagation()
    s.toggleLike(id)
  }
  return {
    query: s.query,
    filteredPlaylists: s.filteredPlaylists,
    activePlaylist: s.activePlaylist,
    playlist: s.playlist,
    tracks: s.tracks,
    visibleTracks: s.visibleTracks,
    trackCount: s.trackCount,
    currentId: s.currentId,
    playing: s.playing,
    position: s.position,
    current: s.current,
    advance: s.advance,
    playTrack: s.playTrack,
    togglePlay: s.togglePlay,
    positionLabel: s.positionLabel,
    totalLabel: s.totalLabel,
    shuffle: s.shuffle,
    repeat: s.repeat,
    liked: s.liked,
    toggleLike: s.toggleLike,
    likedTracks: s.likedTracks,
    volume: s.volume,
    theme: s.theme,
    setTheme: s.setTheme,
    THEMES: s.THEMES,
    stopAndLike,
  }
})
</controller>

<partial playlist-row props: PlaylistRowProps>
  <row
    on:click={onPick}
    current={selected ? 'true' : null}
    align="center"
    justify="between"
    style(listRow)
  >
    <span>{name}</span>
    <badge tone={tone}>{count}</badge>
  </row>
</partial>

<partial track-row props: TrackRowProps>
  <row
    on:click={onPlay}
    current={isCurrent ? 'true' : null}
    align="center"
    gap="3"
    style(trackRow)
  >
    <div style(cover)>
      <img src={cover} alt="" style(coverImg)>
      <if {isCurrent && isPlaying}>
        <center style(playingMask)>
          <row gap="1" align="end" justify="center" style({ box: size({ min-h: 4 }) })>
            <i style(eqBar)></i>
            <i style(eqBarMid)></i>
            <i style(eqBarShort)></i>
          </row>
        </center>
      </if>
    </div>
    <column gap="0" style(trackMeta)>
      <strong style(trackTitle)>{title}</strong>
      <p style(mutedSm)>{artist}</p>
    </column>
    <p style(albumCol)>{album}</p>
    <button
      on:click={(e) => onLike(e, id)}
      variant="ghost"
      size="sm"
      tone={isLiked ? 'primary' : undefined}
    >
      ♥
    </button>
    <span style(len)>{len}</span>
  </row>
</partial>

<!-- Fill remaining viewport under sticky site chrome (`h:fill`).
     Nested theme root: theme= attr + applyNestedTheme stamps data-mode
     (C-TH-NEST-1; data-* banned in app .sail markup — mode is runtime). -->
<column id="shanty-frame" theme={theme()} gap="0" style(root)>
  <row align="center" gap="3" style(chrome)>
    <strong style(brand)>shanty</strong>
    <p style(tagline)>a full app in one .sail file</p>
    <row gap="1">
      <for {THEMES as t} key={t}>
        <button on:click={() => setTheme(t)} variant={theme() === t ? 'solid' : 'soft'} tone="primary" size="sm">{t}</button>
      </for>
    </row>
  </row>

  <row align="stretch" gap="0" style(body)>
    <column gap="3" style(rail)>
      <input id="shanty-filter" type="text" placeholder="Filter playlists" bind:value={query} />
      <column gap="1">
        <p style(sectionLabel)>Playlists</p>
        <for {filteredPlaylists() as p} key={p.id}>
          <playlist-row
            name={p.name}
            count={p.tracks.length}
            selected={activePlaylist() === p.id}
            tone="muted"
            onPick={() => activePlaylist.set(p.id)}
          ></playlist-row>
        </for>
      </column>
      <separator></separator>
      <playlist-row
        name="♥ Liked"
        count={liked().length}
        selected={activePlaylist() === '__liked'}
        tone="primary"
        onPick={() => activePlaylist.set('__liked')}
      ></playlist-row>
    </column>

    <column gap="0" style(mainCol)>
      <column gap="1" style(playlistHead)>
        <p style(accentLabel)>Playlist</p>
        <h1 style(title)>{activePlaylist() === '__liked' ? 'Liked Tracks' : playlist().name}</h1>
        <p style(mutedSm)>
          {trackCount()} tracks · click a row to play
        </p>
      </column>
      <scroll-area style(trackScroll)>
        <column gap="1">
          <for {visibleTracks() as t} key={t.id}>
            <track-row
              id={t.id}
              title={t.title}
              artist={t.artist}
              album={t.album}
              len={t.len}
              cover={t.cover}
              isCurrent={currentId() === t.id}
              isPlaying={playing()}
              isLiked={liked().includes(t.id)}
              onPlay={() => playTrack(t.id)}
              onLike={stopAndLike}
            ></track-row>
          </for>
        </column>
      </scroll-area>
    </column>
  </row>

  <column gap="2" style(nowPlaying)>
    <row align="center" gap="4">
      <row align="center" gap="3" style(growMin)>
        <div style(nowCover)>
          <img src={current().cover} alt="" style(coverImg)>
        </div>
        <column gap="0" style({ box: size({ min-w: 0 }) })>
          <strong>{current().title}</strong>
          <p style(mutedSm)>{current().artist} · {current().album}</p>
        </column>
      </row>
      <column align="center" gap="1" style(transport)>
        <row gap="2" align="center">
          <button on:click={() => shuffle.set(!shuffle())} variant={shuffle() ? 'solid' : 'ghost'} size="sm">shuffle</button>
          <button on:click={() => advance(-1)} variant="ghost">⏮</button>
          <button on:click={() => togglePlay()} variant="solid" tone="primary" style(pressBtn)>
            {playing() ? '⏸' : '▶'}
          </button>
          <button on:click={() => advance(1)} variant="ghost">⏭</button>
          <button on:click={() => repeat.set(!repeat())} variant={repeat() ? 'solid' : 'ghost'} size="sm">repeat</button>
        </row>
        <row gap="2" align="center" nowrap style({ box: size({ w: full, max-w: panel-max-w-md }) })>
          <span style(time)>{positionLabel()}</span>
          <row style(progressShell)>
            <progress value={position()} max={current().secs}></progress>
          </row>
          <span style(time)>{totalLabel()}</span>
        </row>
      </column>
      <row gap="2" align="center" justify="end" nowrap style(growMin)>
        <button on:click={() => volume.set(Math.max(0, volume() - 10))} variant="ghost" size="sm">−</button>
        <div style(volumeTrack)>
          <progress value={volume()} max="100"></progress>
        </div>
        <button on:click={() => volume.set(Math.min(100, volume() + 10))} variant="ghost" size="sm">+</button>
      </row>
    </row>
  </column>
</column>
