<!--
  Lesson 08 starter — THIS FILE DOES NOT COMPILE, on purpose.

  Four defects are planted below. Do not hunt for them by eye: run

    sail-compiler build --mode component tut-08-release-notes-starter.sail

  read the first rejection, fix exactly what it names, and run again.
  The compiler reports one costed rejection at a time, and each message
  carries the allowed vocabulary — the error IS the documentation.
-->

<controller>
import { controller, signal } from '@ship/sail-runtime'

interface Note {
  id: number
  tag: string
  text: string
}

// This controller is fine. Every planted defect in this lesson lives in
// the template — run the compiler on it and let the errors lead you.
export default controller(() => {
    const publishedAt = signal(new Date('2026-07-01T09:00:00Z'))
    const notes = signal<Note[]>([
      { id: 1, tag: 'feat', text: 'motion:loop — ambient animation as a directive' },
      { id: 2, tag: 'fix', text: 'exit-hold no longer races the reconciler' },
      { id: 3, tag: 'docs', text: 'the tutorial you are reading right now' },
    ])

    return {
      publishedAt,
      notes,
      formatDate: (d: Date): string =>
        d.toLocaleDateString('en-GB', { day: 'numeric', month: 'long', year: 'numeric' }),
    }
  })
</controller>

<column gap="4" sx={style.pad(12px)}>
  <row align="center" justify="between">
    <h3>Release notes</h3>
    <badge tone="primary" variant="soft" motion:enter={{ opacity: 'whoosh' }}>v2.4.0</badge>
  </row>

  <p sx={style.fontSize(sm).fg(muted-foreground)}>published {formatDate(publishedAt())}</p>

  <column gap="1">
    {#for note of notes() key={note.id}}
      <row gap="3" align="center" sx={style.padX(3).padY(2).bg(surface-1).radius(md)}>
        <badge tone="muted" variant="outline" size="sm">{note.tag}</badge>
        <p sx={style.fontSize(sm)}>{note.text}</p>
      </row>
    {/for}
  </column>
</column>

<style>
  .notes { color: #ff0000; }
</style>
