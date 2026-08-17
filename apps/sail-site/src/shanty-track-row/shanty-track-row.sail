<!--
  Showcase track row — enter fade, EQ while this row is the playing track.
-->
<scope>
  props: {
    id = 0
    title = ''
    artist = ''
    album = ''
    len = ''
    cover = ''
    isCurrent = false
    isPlaying = false
    isLiked = false
    onPlay: any = null
    onLike: any = null
  }

  style enter = { animate: { enter: play(fade) } }
</scope>

<controller>
export default controller((ctx) => {
  const play = (): void => {
    const fn = ctx.onPlay()
    if (typeof fn === 'function') fn()
  }
  const like = (e: unknown): void => {
    const fn = ctx.onLike()
    if (typeof fn === 'function') fn(e, ctx.id())
  }
  return { play, like }
})
</controller>

<row
  nowrap
  gap="3"
  on:click={play}
  style(enter)
>
  <img src={cover()} alt="">
  <column gap="0">
    <strong>{title()}</strong>
    <p>{artist()}</p>
  </column>
  <p>{album()}</p>
  <button
    on:click={like}
    variant={isCurrent() ? 'soft' : 'ghost'}
    size="sm"
    tone={isLiked() ? 'primary' : undefined}
  >
    ♥
  </button>
  <span>{len()}</span>
  <if {isCurrent() && isPlaying()}>
    <shanty-eq></shanty-eq>
  </if>
</row>
