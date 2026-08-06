<!--
  Lesson 06 solved — one directive, one system.

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

// This lesson's state is deliberately boring — every interesting line in
// it lives in the TEMPLATE, because motion is a directive, not code.
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
  const unreadDot = style.display(inline-block).w(1_5).h(1_5).radius(full).bg(destructive)
  const noteRow = style.pad(3).bg(surface-1).borderWidth(1).border(border-subtle).radius(md)
  const noteText = style.fontSize(sm)
  const empty = style.pad(3).fontSize(sm).fg(muted-foreground)
</scope>

<column gap="4">
  <row align="center" justify="between">
    <h3>Notifications</h3>
    <row gap="2" align="center">
      <button variant="ghost" size="sm" on:click={() => restock()}>
        style.hover({ rotate: 'spring:bouncy' })
Restock</button>
      <button variant="outline" tone="primary" size="sm"
on:click={() => toggle()}>
        style.hover({ scale: 'spring:gentle' }).press({ scale: 'press' })

        {open() ? 'Close tray' : 'Open tray'}
        <if {unread() > 0}>
          <span>
            unreadDot.loop({ scale: 'pulse' })
</span>
        </if>
      </button>
    </row>
  </row>

  <if {open()}>
    <column gap="2">
      style.enter({ opacity: 'fade', translate-x: 'spring:snappy' }).exit({ opacity: 'fade-out' })

      <for {notes()} as="n" key={n.id}>
        <row gap="3" align="center">
          noteRow.enter({ opacity: 'fade', translate-x: 'spring:gentle', stagger: '50ms' }).exit({ opacity: 'fade-out' })

          <badge tone={n.tone} variant="soft" size="sm">{n.kind}</badge>
          <p grow>
            style(noteText)
{n.text}</p>
          <button variant="ghost" size="sm" on:click={() => dismiss(n.id)}>Dismiss</button>
        </row>
      </for>
      <if {notes().length === 0}>
        <p>
          empty.enter({ opacity: 'fade' })
All clear.</p>
      </if>
    </column>
  </if>
</column>
