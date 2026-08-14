<!--
  Lesson 09 starter — it compiles, it runs, it demos well. Ship it?

  No. Five review flaws are planted in this template, and the compiler
  accepts every one of them. Read it the way you would read an agent's
  pull request: what breaks on the second render, the slow network, the
  reorder, the translation pass — the paths the demo never took.
-->
<scope>
  props: {
    title = 'Support inbox'
  }

  style messageRow = { pad({ x: 3, y: 2 }).bg(surface-1).border({ color: border-subtle, width: 1 }).radius(md) }
  style unreadDot = { display(inline-block).size(1_5).radius(full).bg(primary) }
  style from = { type({ size: sm }) }
  style muted = { fg(muted-foreground) }
  style emptyImprovised = { pad(4).fg(muted-foreground).type({ size: sm }) }
</scope>
<controller>
import { inboxClient } from '@/compositions/tut-09-inbox-client'

// This controller is sound. Every flaw in this lesson lives in the
// TEMPLATE, and every one of them compiles — that is the point: the
// compiler owns correctness of the surface, the reviewer owns intent.
export default controller(() => {
    const inbox = query(() => inboxClient.messages.list())
    const filter = signal<'all' | 'unread'>('all')
    const archived = signal<number[]>([])

    const visible = derived(() => {
      const state = inbox()
      if (state.kind !== 'ok') return []
      return state.value.filter((m) => !archived().includes(m.id) && (filter() === 'all' || !m.read))
    })

    return {
      inbox,
      filter,
      visible,
      setFilter: (f: 'all' | 'unread'): void => filter.set(f),
      archive: (id: number): void => archived.set([...archived(), id]),
    }
  })
</controller>

<column gap="4">
  <row align="center" justify="between">
    <h3>Support inbox</h3>
    <row gap="1">
      <button size="sm" tone="primary" variant={filter() === 'all' ? 'solid' : 'ghost'} on:click={() => setFilter('all')}>All</button>
      <button size="sm" tone="primary" variant={filter() === 'unread' ? 'solid' : 'ghost'} on:click={() => setFilter('unread')}>Unread</button>
    </row>
  </row>

  <match {inbox()}>
    {loading => }
    {error as err =>
      <alert tone="destructive">{err.message}</alert>
    }
    {ok =>
      <column gap="1">
        <for {visible() as m} key={m.topic}>
          <row
            gap="3"
            align="center"
            style(messageRow, { animate: { enter: play(slide-up) } })
          >
            <if {!m.read}><span style(unreadDot)></span></if>
            <column gap="0" style({ grow(1) })>
              <strong style(from)>{m.from}</strong>
              <small style(muted)>{m.subject}</small>
            </column>
            <badge tone={m.tone} variant="soft" size="sm">{m.topic}</badge>
            <button size="sm" variant="ghost" on:click={() => archive(m.id)}>Archive</button>
          </row>
        </for>
        <if {visible().length === 0}>
          <div style(emptyImprovised)>Nothing matches this filter.</div>
        </if>
      </column>
    }
  </match>
</column>
