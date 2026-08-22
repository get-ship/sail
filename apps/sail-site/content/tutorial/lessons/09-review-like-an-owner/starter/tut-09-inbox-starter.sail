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

  style messageRow = { pad(3).bg(surface).border(subtle).radius(md) }
  style unreadDot = { size(1_5).radius(full).bg(primary) }
  style from = { type(body-sm) }
  style muted = { fg(muted) }
  style emptyImprovised = { pad(4).fg(muted).type(body-sm) }
  style fill = { size(grow) }
  style rowEnter = { animate: { enter: play(slide-up) } }
</scope>
<controller>
import { inboxClient } from '@/compositions/tut-09-inbox-client'

// This controller is sound. Every flaw in this lesson lives in the
// TEMPLATE, and every one of them compiles — that is the point: the
// compiler owns correctness of the surface, the reviewer owns intent.
export default controller(() => {
    const inbox = query(() => inboxClient.messages.list())
    const view = signal<'all' | 'unread'>('all')
    const archived = signal<number[]>([])

    const visible = derived(() => {
      const state = inbox()
      if (state.kind !== 'ok') return []
      return state.value.filter((m) => !archived().includes(m.id) && (view() === 'all' || !m.read))
    })

    return {
      inbox,
      view,
      visible,
      setView: (f: 'all' | 'unread'): void => view.set(f),
      archive: (id: number): void => archived.set([...archived(), id]),
    }
  })
</controller>

<column gap="4">
  <row align="center" justify="between">
    <h3>Support inbox</h3>
    <row gap="1">
      <button size="sm" tone="primary" variant={view() === 'all' ? 'solid' : 'ghost'} on:click={() => setView('all')}>All</button>
      <button size="sm" tone="primary" variant={view() === 'unread' ? 'solid' : 'ghost'} on:click={() => setView('unread')}>Unread</button>
    </row>
  </row>

  <match {inbox()}>
    {loading => }
    {error as err =>
      <alert tone="danger">{err.message}</alert>
    }
    {ok =>
      <column gap="1">
        <for {visible() as m} key={m.topic}>
          <row
            gap="3"
            align="center"
            style(messageRow, rowEnter)
          >
            <if {!m.read}><strong style(unreadDot)></strong></if>
            <column gap="0" style(fill)>
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
