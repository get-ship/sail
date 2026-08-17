<!--
  SHANTY — full player. Lives in L2 so play() / enter fade / EQ stay.
  Route is a single tag. Do not strip motion to compile.
-->
<scope>
  type PlaylistRowProps = {
    name: string
    count: number
    selected: boolean
    tone: string
    onPick: any
  }

  style headIn = { animate: { enter: play(slide-up) } }
</scope>

<controller>
import type { UiEvent } from '../compositions/ui-event'
import { createShanty } from '../compositions/shanty-vm'

export default controller(() => {
  const s = createShanty()
  const stopAndLike = (e: UiEvent, id: number): void => {
    e.stopPropagation()
    s.toggleLike(id)
  }
  const volDown = (): void => {
    s.volume.set(Math.max(0, s.volume() - 10))
  }
  const volUp = (): void => {
    s.volume.set(Math.min(100, s.volume() + 10))
  }
  const toggleShuffle = (): void => {
    s.shuffle.set(!s.shuffle())
  }
  const toggleRepeat = (): void => {
    s.repeat.set(!s.repeat())
  }
  const prev = (): void => {
    s.advance(-1)
  }
  const next = (): void => {
    s.advance(1)
  }
  const playPause = (): void => {
    s.togglePlay()
  }
  const pickLiked = (): void => {
    s.activePlaylist.set('__liked')
  }
  const themeChips = derived(() =>
    s.THEMES.map((t: string) => ({
      id: t,
      current: s.theme() === t,
      pick: (): void => {
        s.setTheme(t)
      },
    })),
  )
  const playlistRows = derived(() =>
    s.filteredPlaylists().map((p: { id: string; name: string; tracks: unknown[] }) => ({
      id: p.id,
      name: p.name,
      count: p.tracks.length,
      selected: s.activePlaylist() === p.id,
      pick: (): void => {
        s.activePlaylist.set(p.id)
      },
    })),
  )
  const trackRows = derived(() =>
    s.visibleTracks().map((t: { id: number; title: string; artist: string; album: string; len: string; cover: string }) => ({
      ...t,
      isCurrent: s.currentId() === t.id,
      isPlaying: s.playing(),
      isLiked: s.liked().includes(t.id),
      play: (): void => {
        s.playTrack(t.id)
      },
    })),
  )
  return {
    query: s.query,
    filteredPlaylists: s.filteredPlaylists,
    activePlaylist: s.activePlaylist,
    playlist: s.playlist,
    visibleTracks: s.visibleTracks,
    trackCount: s.trackCount,
    currentId: s.currentId,
    playing: s.playing,
    trackPos: s.position,
    current: s.current,
    playTrack: s.playTrack,
    playPause,
    positionLabel: s.positionLabel,
    totalLabel: s.totalLabel,
    shuffle: s.shuffle,
    repeat: s.repeat,
    liked: s.liked,
    volume: s.volume,
    theme: s.theme,
    setTheme: s.setTheme,
    THEMES: s.THEMES,
    stopAndLike,
    volDown,
    volUp,
    toggleShuffle,
    toggleRepeat,
    prev,
    next,
    pickLiked,
    themeChips,
    playlistRows,
    trackRows,
  }
})
</controller>

<partial playlist-row props: PlaylistRowProps>
  <row nowrap> gap="3" on:click={onPick}>
    <span>{name}</span>
    <badge tone={tone} variant={selected ? 'solid' : 'soft'}>{count}</badge>
  </row>
</partial>

<column id="shanty-frame" theme={theme()} gap="0">
  <row nowrap> gap="2">
    <strong>shanty</strong>
    <p>a full app in one .sail file</p>
    <row gap="1">
      <for {themeChips() as t} key={t.id}>
        <button on:click={t.pick} variant={t.current ? 'solid' : 'soft'} tone="primary" size="sm">{t.id}</button>
      </for>
    </row>
  </row>

  <row align="stretch" gap="0">
    <column gap="3">
      <input id="shanty-filter" type="text" placeholder="Filter playlists" bind:value={query} />
      <column gap="1">
        <p>Playlists</p>
        <for {playlistRows() as p} key={p.id}>
          <playlist-row
            name={p.name}
            count={p.count}
            selected={p.selected}
            tone="muted"
            onPick={p.pick}
          ></playlist-row>
        </for>
      </column>
      <separator></separator>
      <playlist-row
        name="♥ Liked"
        count={liked().length}
        selected={activePlaylist() === '__liked'}
        tone="primary"
        onPick={pickLiked}
      ></playlist-row>
    </column>

    <column gap="0">
      <card style(headIn)>
        <p>Playlist</p>
        <h1>{activePlaylist() === '__liked' ? 'Liked Tracks' : playlist().name}</h1>
        <p>
          {trackCount()} tracks · click a row to play
        </p>
      </card>
      <scroll-area>
        <column gap="1">
          <for {trackRows() as t} key={t.id}>
            <shanty-track-row
              id={t.id}
              title={t.title}
              artist={t.artist}
              album={t.album}
              len={t.len}
              cover={t.cover}
              isCurrent={t.isCurrent}
              isPlaying={t.isPlaying}
              isLiked={t.isLiked}
              onPlay={t.play}
              onLike={stopAndLike}
            ></shanty-track-row>
          </for>
        </column>
      </scroll-area>
    </column>
  </row>

  <card>
    <row align="center" gap="4">
      <row align="center" gap="3">
        <img src={current().cover} alt="">
        <column gap="0">
          <strong>{current().title}</strong>
          <p>{current().artist} · {current().album}</p>
        </column>
      </row>
      <column align="center" gap="1">
        <row gap="2" align="center">
          <button on:click={toggleShuffle} variant={shuffle() ? 'solid' : 'ghost'} size="sm">shuffle</button>
          <button on:click={prev} variant="ghost">⏮</button>
          <button on:click={playPause} variant="solid" tone="primary">
            {playing() ? '⏸' : '▶'}
          </button>
          <button on:click={next} variant="ghost">⏭</button>
          <button on:click={toggleRepeat} variant={repeat() ? 'solid' : 'ghost'} size="sm">repeat</button>
        </row>
        <row gap="2" align="center" nowrap>
          <span>{positionLabel()}</span>
          <progress value={trackPos()} max={current().secs}></progress>
          <span>{totalLabel()}</span>
        </row>
      </column>
      <row gap="2" align="center" justify="end" nowrap>
        <button on:click={volDown} variant="ghost" size="sm">−</button>
        <progress value={volume()} max="100"></progress>
        <button on:click={volUp} variant="ghost" size="sm">+</button>
      </row>
    </row>
  </card>
</column>
