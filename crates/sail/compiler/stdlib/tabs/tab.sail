<!--
  <tab> — trigger + panel in one tag.

  Primary:
    <tabs>
      <tab title="Overview">panel…</tab>
    </tabs>

  Rich label: <span slot="title">…</span> plus the default slot (panel).
  `{title()}` is a compile-time prop hole — splice fills it from the
  use-site title attr (static first paint; `{expr}` stays a page binding).

  Parts form (vertical rail, sticky strip): empty body + sibling <tab-panel>
  with a matching `name`, or <tab-list> wrapping titled triggers.

  Host is display(contents) so the trigger and body are flex items of
  <tabs> or <tab-list>. Projection is Style order + the wrapping row on
  <tabs> — not platform CSS.

  State: parent `active` (signal) + `select` as props, filled by the
  inliner. Click is the platform <button>. Selected look is Style
  when { group-selected }.
-->

<scope>
  props: {
    title = ''
    name = ''
    disabled = false
    variant: 'line' | 'pill' | 'enclosed' = 'line'
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  style host = {display(contents).group()}
  style trigger = {position({ order: 0 }).pad({ y: 2, x: 3 }).fg(muted-foreground).bg(transparent).border({ width: 0, edge: l }).border({ width: 0, edge: r }).border({ width: 0, edge: t }).border({ width: 2, edge: b, color: transparent }).radius({ t: sm }).cursor(pointer).select(none).tapHighlight(none).type({ family: body, weight: medium, leading: none, wrap: nowrap, size: sm }), animate: { transition: play(colors, fast) }}
  style sizeMd = {pad({ y: 2, x: 3 }).type({ size: sm })}
  style sizeSm = {pad({ y: 1_5, x: 2_5 }).type({ size: xs })}
  style sizeLg = {pad({ y: 2_5, x: 4 }).type({ size: base })}
  style pill = {border({ width: 0, edge: b }).radius(sm)}
  style enclosed = {border({ color: transparent, width: 1, edge: l }).border({ color: transparent, width: 1, edge: r }).border({ color: transparent, width: 1, edge: t }).border({ color: transparent, width: 0, edge: b }).radius({ t: md })}
  style body = {display(none).position({ order: 1 }).pad({ t: 4 }).size({ w: 100%, min-w: 0 })}
  style bodyShow = {display(block)}
</scope>

<controller>
export default controller((init) => {
  const name = String(init.name || init.title || '')
  function onSelect() {
    if (init.disabled === '' || init.disabled === true || init.disabled === 'true') return
    if (typeof init.select === 'function') init.select(name)
  }
  const selected = derived(() => {
    const active = init.active
    return !!(typeof active === 'function' && name && active() === name)
  })
  return { onSelect, selected }
})
</controller>

<span style(host) selected={selected()}>
  <button
    type="button"
    anatomy
    on:click={onSelect}
    selected={selected()}
    style(trigger, {when: {hover: fg(foreground), group-selected: fg(foreground).border({ color: primary, edge: b }).type({ weight: semibold }), focus: border({ style: solid, kind: outline, width: ring-width, color: ring, offset: ring-inset }), disabled: filter({ opacity: 50 }).cursor(not-allowed)}, match: {size: { md: sizeMd, sm: sizeSm, lg: sizeLg }, variant: {line: border({ color: transparent, edge: b }), pill: pill, enclosed: enclosed}}, compound: [
      {variant: pill, when: {group-selected: fg(foreground).bg(muted).elevation(xs).border({ color: transparent, edge: b })}},
      {variant: enclosed, when: {group-selected: fg(foreground).border({ color: border }).bg(muted).translateY(px)}},
    ]})
  >
    {title()}
    <slot name="title"></slot>
  </button>
  <div
    anatomy
    style(body, {when: {empty: display(none), group-selected: bodyShow}})
  >
    <slot></slot>
  </div>
</span>
