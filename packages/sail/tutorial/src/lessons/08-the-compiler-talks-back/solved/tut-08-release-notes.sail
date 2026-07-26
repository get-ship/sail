<!--
  Lesson 08 solved — every rejection answered on the compiler's terms.

  style.pad(12px) became the token pad(4); the motion preset is 'fade'
  (the closed set); the date is a zero-arg derived read; the <style>
  block is gone — the row surface it hand-painted is Style on sx now.
  No fix fought the language.
-->

<controller>
interface Note {
  id: number
  tag: string
  text: string
}

export default controller(() => {
    const publishedAt = signal(new Date('2026-07-01T09:00:00Z'))
    const notes = signal<Note[]>([
      { id: 1, tag: 'feat', text: 'style.loop — ambient animation on Style' },
      { id: 2, tag: 'fix', text: 'exit-hold no longer races the reconciler' },
      { id: 3, tag: 'docs', text: 'the tutorial you are reading right now' },
    ])

    // The starter called formatDate(publishedAt()) IN THE TEMPLATE — an
    // argument-taking call, which renderable expressions forbid. The fix
    // is always the same move: do the work here, expose a zero-arg read.
    const published = derived(() =>
      publishedAt().toLocaleDateString('en-GB', { day: 'numeric', month: 'long', year: 'numeric' }),
    )

    return { notes, published }
  })
</controller>

<scope>
  const page = style.pad(4)
  const meta = style.fontSize(sm).fg(muted-foreground)
  const noteRow = style.padX(3).padY(2).bg(surface-1).radius(md)
  const noteText = style.fontSize(sm)
</scope>

<column gap="4" sx={page}>
  <row align="center" justify="between">
    <h3>Release notes</h3>
    <badge tone="primary" variant="soft" sx={style.enter({ opacity: 'fade' })}>v2.4.0</badge>
  </row>

  <p sx={meta}>published {published()}</p>

  <column gap="1">
    <for as="note" of={notes()} key={note.id}>
      <row gap="3" align="center" sx={noteRow}>
        <badge tone="muted" variant="outline" size="sm">{note.tag}</badge>
        <p sx={noteText}>{note.text}</p>
      </row>
    </for>
  </column>
</column>
