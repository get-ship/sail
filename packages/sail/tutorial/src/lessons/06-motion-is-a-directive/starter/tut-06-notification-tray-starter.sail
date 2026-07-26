<!--
  Lesson 06 starter — everything works, nothing moves.

  Add fx: directives only — the controller does not change. Dismiss
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
  const unreadDot = style.display(inline-block).w(1_5).h(1_5).radius(full).bg(destructive)
  const noteRow = style.pad(3).bg(surface-1).borderWidth(1).border(border-subtle).radius(md)
  const noteText = style.fontSize(sm)
  const empty = style.pad(3).fontSize(sm).fg(muted-foreground)
</scope>

<column gap="4">
  <row align="center" justify="between">
    <h3>Notifications</h3>
    <row gap="2" align="center">
      <!-- TODO 1: fx:hover={{ rotate: 'spring:bouncy' }} — a playful hint. -->
      <button variant="ghost" size="sm" on:click={() => restock()}>Restock</button>
      <!-- TODO 2: fx:hover (scale, spring:gentle) + fx:press (scale,
           'press') — tactile feedback belongs on the trigger. -->
      <button variant="outline" tone="primary" size="sm" on:click={() => toggle()}>
        {open() ? 'Close tray' : 'Open tray'}
        {#if unread() > 0}
          <!-- TODO 3: fx:loop={{ scale: 'pulse' }} — ambient, and it
               stops under prefers-reduced-motion for free. -->
          <span sx={unreadDot}></span>
        {/if}
      </button>
    </row>
  </row>

  {#if open()}
    <!-- TODO 4: the panel should arrive (fx:enter — opacity fade +
         translate-x spring:snappy) and leave (fx:exit — fade-out).
         exit is the one thing CSS can't do alone: the framework holds the
         node in the tree until the animation finishes. -->
    <column gap="2">
      {#for n of notes() key={n.id}}
        <!-- TODO 5: rows should cascade in (fx:enter with a stagger
             duration) and fade out on dismiss (fx:exit). -->
        <row gap="3" align="center" sx={noteRow}>
          <badge tone={n.tone} variant="soft" size="sm">{n.kind}</badge>
          <p grow sx={noteText}>{n.text}</p>
          <button variant="ghost" size="sm" on:click={() => dismiss(n.id)}>Dismiss</button>
        </row>
      {/for}
      {#if notes().length === 0}
        <p sx={empty}>All clear.</p>
      {/if}
    </column>
  {/if}
</column>
