<!--
  <sail-timeline-item> — one timeline entry: gutter connector + tone dot,
  head row (title slot + timestamp), body content. `interactive` makes the
  row a button (role/tabindex/sail-select from the controller; the pressed
  surface from the Style compound). The connector is two real segments
  around the dot — the first item hides the top run, the last hides the
  bottom run (visibility, so the gutter geometry never shifts).

  Tone: real Tone members only — omit for rest (never tone="default").
  Rest dot = foreground; brand = bound bg(tone). muted → muted-foreground.

  Residual (LANG-GAPS.md): forced-colors focus outline (system colors are
  not sx values); first/last detection is mount-time (static lists).
-->

<scope>
  props: {
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    at = ''
    interactive = false
  }

  style host = { box: position(relative) }
  style gutter = { box: w(5) }
  style connectorHide = { box: w(half).h(3), paint: visibility(hidden) }
  style connector = { box: w(half).h(3), paint: bg(border-subtle) }
  style connectorGrowHide = { box: grow(1).w(half), paint: visibility(hidden) }
  style connectorGrow = { box: grow(1).w(half), paint: bg(border-subtle) }
  // Rest = foreground; bound tone expands [tone='…'] gates over rest.
  style dot = { box: w(2_5).h(2_5), paint: radius(full).bg(foreground) }
  style bodyCol = { box: grow(1).minW(0).padY(2) }
  style head = { paint: fg(foreground) }
  style titleSlot = { box: display(contents), text: fontSize(sm).fontWeight(medium).fontFamily(body) }
  style atTime = { paint: fg(muted-foreground), text: fontSize(xs).tracking(wide).fontFamily(mono) }
  style body = { box: mt(1), paint: fg(muted-foreground) }
  style bodySlot = { box: display(contents), text: fontSize(sm).leading(normal) }
</scope>

<controller>

export default controller((ctx) => {
    const first = signal(false)
    const last = signal(false)

    mount((host) => {
      if (!host.hasAttribute('role')) host.setAttribute('role', 'listitem')
      first.set(host.matches(':first-of-type'))
      last.set(host.matches(':last-of-type'))

      const stopRole = effect(() => {
        if (ctx.interactive()) {
          host.setAttribute('role', 'button')
          if (!host.hasAttribute('tabindex')) host.setAttribute('tabindex', '0')
        } else {
          host.setAttribute('role', 'listitem')
          host.removeAttribute('tabindex')
        }
      })

      const emit = () => {
        ;(host as any).onSelect?.({})
      }

      const onClick = () => { if (ctx.interactive()) emit() }

      const onKeydown = (e: KeyboardEvent) => {
        if (!ctx.interactive()) return
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault()
          emit()
        }
      }

      host.addEventListener('click', onClick)
      host.addEventListener('keydown', onKeydown)

      return () => {
        stopRole()
        host.removeEventListener('click', onClick)
        host.removeEventListener('keydown', onKeydown)
      }
    })

    return { first, last }
  })

</controller>
<row gap="2" align="stretch" style(host, { compound: [ { interactive: true, paint: cursor(pointer).radius(sm), when: { hover: { paint: bg(muted) }, focus: { paint: outline(solid).outline(2).outline(ring).outlineOffset(2) } } } ] })>
  <column align="center" gap="0" style(gutter)>
    <if {first()}>
      <span style(connectorHide)></span>
    {else => <span style(connector)></span>}</if>
    <span style(dot, { paint: bg(tone), compound: [ { tone: muted, paint: bg(muted-foreground) } ] })></span>
    <if {last()}>
      <span style(connectorGrowHide)></span>
    {else => <span style(connectorGrow)></span>}</if>
  </column>
  <column gap="0" style(bodyCol)>
    <row align="baseline" justify="between" gap="2" style(head)>
      <span style(titleSlot)>
        <slot name="title"></slot>
      </span>
      <if {at()}>
        <time style(atTime)>{at()}</time>
      </if>
    </row>
    <div style(body)>
      <span style(bodySlot)>
        <slot></slot>
      </span>
    </div>
  </column>
</row>
