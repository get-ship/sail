<!--
  <sail-timeline-item> — gutter connector + tone dot, title/at head, body slot.
  `interactive` makes the row a button (role/tabindex + onSelect).
  First/last items hide the top/bottom connector run (visibility, not layout shift).
  Tone: real Tone members only — omit for rest (never tone="default").
  Rest dot = foreground; brand = bound bg(tone); muted → muted-foreground.
-->

<scope>
  props: {
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    at = ''
    interactive = false
  }

  style host = { position(relative) }
  style gutter = { size({ w: 5 }) }
  style connectorHide = { size({ w: half, h: 3 }).filter({ opacity: 0 }) }
  style connector = { size({ w: half, h: 3 }).bg(border-subtle) }
  style connectorGrowHide = { grow(1).size({ w: half }).filter({ opacity: 0 }) }
  style connectorGrow = { grow(1).size({ w: half }).bg(border-subtle) }
  style dot = { size({ w: 2_5, h: 2_5 }).radius(full).bg(foreground) }
  style bodyCol = { grow(1).size({ min-w: 0 }).pad({ y: 2 }) }
  style head = { fg(foreground) }
  style titleSlot = { display(contents).type({ family: body, size: sm, weight: medium }) }
  style atTime = { fg(muted-foreground).type({ family: mono, size: xs, tracking: wide }) }
  style body = { margin({ t: 1 }).fg(muted-foreground) }
  style bodySlot = { display(contents).type({ size: sm, leading: normal }) }
</scope>

<controller>

export default controller((ctx) => {
    const first = signal(false)
    const last = signal(false)

    mount((host) => {
      if (!hostHasAttr(host, 'role')) hostSetAttr(host, 'role', 'listitem')
      // Static lists: first/last of type at mount.
      first.set(host.matches(':first-of-type'))
      last.set(host.matches(':last-of-type'))

      const stopRole = effect(() => {
        if (ctx.interactive()) {
          hostSetAttr(host, 'role', 'button')
          if (!hostHasAttr(host, 'tabindex')) hostSetAttr(host, 'tabindex', '0')
        } else {
          hostSetAttr(host, 'role', 'listitem')
          hostSetAttr(host, 'tabindex', null)
        }
      })

      const emit = () => {
        ;(host).onSelect?.({})
      }

      const onClick = () => { if (ctx.interactive()) emit() }

      const onKeydown = (e: DomEv) => {
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
<row
  gap="2"
  align="stretch"
  style(host, { compound: [{ interactive: true, cursor(pointer).radius(sm), when: { hover: bg(muted), focus: border({ kind: outline, color: ring, width: 2, style: solid }) } }] })
>
  <column align="center" gap="0" style(gutter)>
    <if {first()}>
      <span style(connectorHide)></span>
    {else =>
      <span style(connector)></span>
    }</if>
    <span style(dot, { bg(tone), compound: [{ tone: muted, bg(muted-foreground) }] })></span>
    <if {last()}>
      <span style(connectorGrowHide)></span>
    {else =>
      <span style(connectorGrow)></span>
    }</if>
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
