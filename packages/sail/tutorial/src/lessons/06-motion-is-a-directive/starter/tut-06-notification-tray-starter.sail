<!--
  Lesson 06 starter — everything works, nothing moves.

  Add Style motion methods only (.enter/.exit/.hover/.press/.loop) — the controller does not change. Dismiss
  currently SNAPS the row away; opening the tray teleports it in; the
  unread dot just sits there. Each TODO names the trigger and property
  to reach for; the presets are a closed set the compiler enforces.
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
  const unreadDot = style({ box: display(inline-block).w(1_5).h(1_5), paint: radius(full).bg(destructive) })
  const noteRow = style({ box: pad(3), paint: bg(surface-1).borderWidth(1).border(border-subtle).radius(md) })
  const noteText = style({ text: fontSize(sm) })
  const empty = style({ box: pad(3), paint: fg(muted-foreground), text: fontSize(sm) })
</scope>

<column gap="4">
  <row align="center" justify="between">
    <h3>Notifications</h3>
    <row gap="2" align="center">
      <!-- TODO 1: style({ animate: hover({ rotate: 'spring:bouncy' }) }) as a block — a playful hint. -->
      <button variant="ghost" size="sm" on:click={() => restock()}>Restock</button>
      <!-- TODO 2: style({ animate: hover({ scale: 'spring:gentle' }) }) + style({ paint: press({ scale:
           'press') — tactile feedback belongs on the trigger. -->
      <button variant="outline" tone="primary" size="sm" on:click={() => toggle()}>
        {open() ? 'Close tray' : 'Open tray'}
        <if {unread() > 0}>
          <!-- TODO 3: style({ animate: loop({ scale: 'pulse' }) }) — ambient, and it
               stops under prefers-reduced-motion for free. -->
          <span>
            style(unreadDot)
          </span>
        </if>
      </button>
    </row>
  </row>

  <if {open()}>
    <!-- TODO 4: the panel should arrive (style.enter — opacity fade +
         translate-x spring:snappy) and leave (style.exit — fade-out).
         exit is the one thing CSS can't do alone: the framework holds the
         node in the tree until the animation finishes. -->
    <column gap="2">
      <for {notes()} as="n" key={n.id}>
        <!-- TODO 5: rows should cascade in (style.enter with a stagger
             duration) and fade out on dismiss (style.exit). -->
        <row gap="3" align="center">
          style(noteRow)

          <badge tone={n.tone} variant="soft" size="sm">{n.kind}</badge>
          <p>
            style([ style(noteText), style({ box: grow(1) }) ])
{n.text}</p>
          <button variant="ghost" size="sm" on:click={() => dismiss(n.id)}>Dismiss</button>
        </row>
      </for>
      <if {notes().length === 0}>
        <p>
          style(empty)
All clear.</p>
      </if>
    </column>
  </if>
</column>
 })