<!--
  Lesson 06 solved — motion lives on Style's animate channel.

  The trigger says WHEN (hover/press/loop/enter/exit), the body says WHAT
  (property: preset). Note the variety: rotate on the restock hint, scale
  on the trigger, pulse on the unread dot, translate-x on the panel,
  staggered rows, fade-out exits. Dismiss demonstrates exit-hold — the
  framework keeps the row in the tree until its animation finishes.
-->

<controller>
interface Note {
  id: number
  kind: string
  tone: 'info' | 'success' | 'warning' | 'destructive'
  text: string
}

const SEED: Note[] = [
  { id: 1, kind: 'deploy', tone: 'success', text: 'production is live at v2.4.0' },
  { id: 2, kind: 'alert', tone: 'warning', text: 'p95 latency crossed 400ms on api-eu' },
  { id: 3, kind: 'review', tone: 'info', text: 'sam requested review on #482' },
  { id: 4, kind: 'incident', tone: 'destructive', text: 'billing webhook failing since 09:12' },
]

// State stays boring on purpose — motion is a Style directive on the
// template, not logic in the controller.
export default controller(() => {
    const open = signal(true)
    const notes = signal<Note[]>(SEED)
    const unread = derived(() => notes().length)

    return {
      open,
      notes,
      unread,
      toggle: (): void => open.set(!open()),
      dismiss: (id: number): void => notes.set(notes().filter((n) => n.id !== id)),
      restock: (): void => notes.set(SEED),
    }
  })
</controller>

<scope>
  style unreadDot = { box: flow({ display: inline-block }).size(1_5), paint: radius(full).fill({ color: destructive }) }
  style noteRow = { box: space({ pad: 3 }), paint: fill({ color: surface-1 }).border({ color: border-subtle, width: 1 }).radius(md) }
  style noteText = { text: type({ size: sm }) }
  style empty = { box: space({ pad: 3 }), paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
</scope>

<column gap="4">
  <row align="center" justify="between">
    <h3>Notifications</h3>
    <row gap="2" align="center">
      <button
        variant="ghost"
        size="sm"
        on:click={() => restock()}
        style({ animate: hover({ rotate: 'spring:bouncy' }) })
      >Restock</button>
      <button
        variant="outline"
        tone="primary"
        size="sm"
        on:click={() => toggle()}
        style({ animate: hover({ scale: 'spring:gentle' }).press({ scale: 'press' }) })
      >
        {open() ? 'Close tray' : 'Open tray'}
        <if {unread() > 0}>
          <span style(unreadDot, { animate: loop({ scale: 'pulse' }) })></span>
        </if>
      </button>
    </row>
  </row>

  <if {open()}>
    <column
      gap="2"
      style({ animate: enter({ opacity: 'fade', translate-x: 'spring:snappy' }).exit({ opacity: 'fade-out' }) })
    >
      <for {notes() as n} key={n.id}>
        <row
          gap="3"
          align="center"
          style(noteRow, {
            animate: enter({ opacity: 'fade', translate-x: 'spring:gentle', stagger: '50ms' }).exit({ opacity: 'fade-out' }),
})
        >
          <badge tone={n.tone} variant="soft" size="sm">{n.kind}</badge>
          <p style(noteText, { box: flow({ grow: 1 }) })>{n.text}</p>
          <button variant="ghost" size="sm" on:click={() => dismiss(n.id)}>Dismiss</button>
        </row>
      </for>
      <if {notes().length === 0}>
        <p style(empty, { animate: enter({ opacity: 'fade' }) })>All clear.</p>
      </if>
    </column>
  </if>
</column>
