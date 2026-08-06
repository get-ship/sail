<!--
  <sail-timeline-item> — one timeline entry: gutter connector + tone dot,
  head row (title slot + timestamp), body content. `interactive` makes the
  row a button (role/tabindex/sail-select from the controller; the pressed
  surface from the compound variant). The connector is two real segments
  around the dot — the first item hides the top run, the last hides the
  bottom run (visibility, so the gutter geometry never shifts).

  Residual (LANG-GAPS.md): forced-colors focus outline (system colors are
  not sx values); first/last detection is mount-time (static lists).
-->

<scope>
  props: {
    tone = 'default'
    at = ''
    interactive = false
  }
  compound: [
    { when: { interactive: true }, sx: 'cursor-pointer rounded-sm hover:bg-muted focus:ring-ring' },
  ]
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
        host.dispatchEvent(
          new CustomEvent('sail-select', { bubbles: true, composed: true, detail: {} }),
        )
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
<row gap="2" align="stretch">
  style.position(relative)

  <column align="center" gap="0">
    style.w(5)

    <if {first()}>
      <span>
        style.w(half).h(3).visibility(hidden)
</span>
    <else>
      <span>
        style.w(half).h(3).bg(border-subtle)
</span>
    </else></if>
    <span>
      style.w(2_5).h(2_5).radius(full).bg(foreground)
</span>
    <if {last()}>
      <span grow>
        style.w(half).visibility(hidden)
</span>
    <else>
      <span grow>
        style.w(half).bg(border-subtle)
</span>
    </else></if>
  </column>
  <column gap="0" grow min-w="0">
    style.padY(2)

    <row align="baseline" justify="between" gap="2">
      style.fontSize(sm).fontWeight(medium).fg(foreground).fontFamily(body)

      <slot name="title"></slot>
      <if {at()}>
        <time>
          style.fg(muted-foreground).fontSize(xs).tracking(wide).fontFamily(mono)
{at()}</time>
      </if>
    </row>
    <div>
      style.fontSize(sm).fg(muted-foreground).leading(normal).mt(1)
<slot></slot></div>
  </column>
</row>
