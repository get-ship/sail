<!--
  Lesson 06 starter — everything works, nothing moves.

  Add Style animate only (enter / exit / hover / press / loop) — the
  controller does not change. Dismiss currently SNAPS the row away;
  opening the tray teleports it in; the unread dot just sits there.
  Each TODO names the trigger and property to reach for; the presets
  are a closed set the compiler enforces.
-->

<scope>
  style unreadDot = { size(1_5).radius(full).bg(danger) }
  style noteRow = { pad(3).bg(surface).border(subtle).radius(md) }
  style noteText = { type(body-sm).size(grow) }
  style empty = { pad(3).fg(muted).type(body-sm) }
</scope>

<controller>
interface Note {
  id: number
  kind: string
  tone: 'info' | 'success' | 'warning' | 'danger'
  text: string
}

const SEED: Note[] = [
  { id: 1, kind: 'deploy', tone: 'success', text: 'production is live at v2.4.0' },
  { id: 2, kind: 'alert', tone: 'warning', text: 'p95 latency crossed 400ms on api-eu' },
  { id: 3, kind: 'review', tone: 'info', text: 'sam requested review on #482' },
  { id: 4, kind: 'incident', tone: 'danger', text: 'billing webhook failing since 09:12' },
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

<column gap="4">
  <row align="center" justify="between">
    <h3>Notifications</h3>
    <row gap="2" align="center">
      <!-- TODO 1: animate: { hover: play(fade) } — a playful hint. -->
      <button variant="ghost" size="sm" on:click={() => restock()}>Restock</button>
      <!-- TODO 2: animate: { hover: play(fade), press: play(fade) } —
           tactile feedback belongs on the trigger. -->
      <button variant="outline" tone="primary" size="sm" on:click={() => toggle()}>
        {open() ? 'Close tray' : 'Open tray'}
        <if {unread() > 0}>
          <!-- TODO 3: compose unreadDot with animate: { loop: play(fade) } —
               ambient, and it stops under prefers-reduced-motion for free. -->
          <strong style(unreadDot)></strong>
        </if>
      </button>
    </row>
  </row>

  <if {open()}>
    <!-- TODO 4: the panel should arrive (animate: { enter: play(fade),
         exit: play(fade) }). Exit is the one thing CSS can't do alone:
         the framework holds the node until the animation ends. -->
    <column gap="2">
      <for {notes() as n} key={n.id}>
        <!-- TODO 5: rows should cascade in (noteRow + animate:
             { enter: play(slide-up), exit: play(fade) })
             and fade out on dismiss. Compose named paint + animate in one style(). -->
        <row gap="3" align="center" style(noteRow)>
          <badge tone={n.tone} variant="soft" size="sm">{n.kind}</badge>
          <p style(noteText)>{n.text}</p>
          <button variant="ghost" size="sm" on:click={() => dismiss(n.id)}>Dismiss</button>
        </row>
      </for>
      <if {notes().length === 0}>
        <p style(empty)>All clear.</p>
      </if>
    </column>
  </if>
</column>
