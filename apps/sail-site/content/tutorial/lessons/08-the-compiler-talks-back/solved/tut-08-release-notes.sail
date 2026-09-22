<!--
  Lesson 08 solved — every rejection answered on the compiler's terms.

  style&#40;&#123; pad(12px) &#125;&#41; became the token pad(4); the motion preset is 'fade'
  (the closed set); the date is a zero-arg derived read; the <style>
  block is gone — the row surface it hand-painted is tip Style on the note row.
  No fix fought the language.
-->

<scope>
  style page = { pad(4) }
  style meta = { fg(muted).type(body-sm) }
  style noteRow = { pad(3).bg(surface).radius(md) }
  style noteText = { type(body-sm) }
  style badgeEnter = { animate: { enter: play(fade) } }
</scope>

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

  // Starter called formatDate(publishedAt()) in the template — argument-taking
  // calls are forbidden in renderable expressions. Do the work here; expose a
  // zero-arg read.
  const published = derived(() =>
    publishedAt().toLocaleDateString('en-GB', { day: 'numeric', month: 'long', year: 'numeric' }),
  )

  return { notes, published }
})
</controller>

<column gap="4" style(page)>
  <row align="center" justify="between">
    <h3>Release notes</h3>
    <badge tone="primary" variant="soft" style(badgeEnter)>v2.4.0</badge>
  </row>

  <p style(meta)>published {published()}</p>

  <column gap="1">
    <for {notes() as note} key={note.id}>
      <row gap="3" align="center" style(noteRow)>
        <badge tone="muted" variant="outline" size="sm">{note.tag}</badge>
        <p style(noteText)>{note.text}</p>
      </row>
    </for>
  </column>
</column>
