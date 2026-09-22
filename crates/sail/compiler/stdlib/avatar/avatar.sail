<!--
  <avatar> — L1 language element. Style value language only.
  Child img pack lives in platform/avatar.css: cover for photo src;
  contain+pad when icon-slug. tone defaults to muted (surface disc).
  Explicit tone always paints.
  ring: outer border (border-strong, or tone when set).
  presence: status pip via host ::after.
  stack: negative margin-inline-end so a row of avatars overlap.

  Look: host Style owns paint — no DOM.
  State: this file's <controller> (src/icon-slug/name/initials → image vs initials).
  Listen: on:error on the <img> flips to initials; src change retries.
-->

<scope>
  props: {
    src = ''
    'icon-slug' = ''
    name = ''
    initials = ''
    tone: 'primary' | 'accent' | 'info' | 'success' | 'warning' | 'danger' | 'muted' = 'muted'
    size: 'md' | 'xs' | 'sm' | 'lg' | 'xl' = 'md'
    shape: 'round' | 'square' = 'round'
    ring = false
    presence: 'off' | 'online' | 'busy' | 'offline' = 'off'
    stack = false
  }

  style base = {position(relative).overflow({ all: hidden }).size(7).fg(foreground).bg(muted).radius(full).elevation({ shadow: inset-ring }).select(none).align(middle).type({ family: body, weight: bold, leading: none, tracking: wide, size: base })}
  style sizeMd = {size(7).type({ size: base })}
  style sizeXs = {size(5).type({ size: xs })}
  style sizeSm = {size(control-h-sm).type({ size: xs })}
  style sizeLg = {size(control-h-lg).type({ size: lg })}
  style sizeXl = {size(control-h-xl).type({ size: 2xl })}
  style ringOn = {boxSizing(border).border({ color: border-strong, width: 2, style: solid })}
  style stackOn = {margin({ e: -2 })}
  // Inset so overflow(hidden) + radius still clips the photo but keeps the pip.
  style pip = {position({ scheme: absolute, r: px, b: px, z: 1 }).size(1_5).content(empty).radius(full).border({ color: background, width: 1 }).pointerEvents(none)}
  style pipOnline = {when: {after: style(pip, {bg(success)})}}
  style pipBusy = {when: {after: style(pip, {bg(danger)})}}
  style pipOffline = {when: {after: style(pip, {bg(muted-foreground)})}}
</scope>

<controller>
export default controller((init) => {
  function read(v) { return typeof v === 'function' ? v() : v }
  const error = signal(false)
  const src = derived(() => {
    const slug = String(read(init['icon-slug']) ?? '').trim()
    if (slug) return `https://cdn.simpleicons.org/${slug}`
    return String(read(init.src) ?? '').trim()
  })
  effect(() => { src(); error.set(false) })
  const showImg = derived(() => !!src() && !error())
  const initials = derived(() => {
    const explicit = String(read(init.initials) ?? '').trim()
    if (explicit) return explicit.slice(0, 2).toUpperCase()
    const name = String(read(init.name) ?? '').trim()
    return name.slice(0, 1).toUpperCase() || '?'
  })
  const alt = derived(() => String(read(init.name) ?? ''))
  function onError() { error.set(true) }
  return { src, showImg, initials, alt, onError }
})
</controller>

<row
  inline
  nowrap
  align="center"
  justify="center"
  style(base, {shrink(0), match: {size: { md: sizeMd, xs: sizeXs, sm: sizeSm, lg: sizeLg, xl: sizeXl }, shape: {round: radius(full), square: radius(sm)}, ring: {true: ringOn}, stack: {true: stackOn}, presence: {online: pipOnline, busy: pipBusy, offline: pipOffline}}, compound: [
      {tone: *, bg(tone).fg(tone.foreground)},
      {tone: muted, bg(muted).fg(foreground)},
      {ring: true, tone: *, border({ color: tone })},
      {stack: true, when: {last: margin({ e: 0 })}},
    ]})
>
  <if {showImg()}>
    <img alt={alt()} src={src()} on:error={onError}>
    {else => <span>{initials()}</span>}
  </if>
</row>
