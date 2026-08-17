<!--
  Lesson 09 solved — the screen as an owner would ship it.

  Compare with the starter: the title flows from the prop; all three
  query() states render; the {#for} keys on identity (m.id); rows exit as
  well as enter; and the empty case is the semantic <empty-state>, not a
  hand-rolled box.
-->
<scope>
  props: {
    title = 'Support inbox'
  }

  style loadingRow = { pad(4).bg(surface).radius(md) }
  style muted = { fg(muted) }
  style messageRow = { pad(3).bg(surface).border(subtle).radius(md) }
  style unreadDot = { size(1_5).radius(full).bg(primary) }
  style from = { type(body-sm) }
  style fill = { size(grow) }
  style rowMotion = { animate: { enter: play(slide-up), exit: play(fade) } }
</scope>
<controller>
import { inboxClient } from '@/compositions/tut-09-inbox-client'

export default controller(() => {
    const inbox = query(() => inboxClient.messages.list())
    const view = signal<'all' | 'unread'>('all')
    const archived = signal<number[]>([])

    // Visible list composes the live value with local state: archive hides
    // rows client-side so exit animation can play; view narrows to unread.
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
    <h3>{title()}</h3>
    <row gap="1">
      <button size="sm" tone="primary" variant={view() === 'all' ? 'solid' : 'ghost'} on:click={() => setView('all')}>All</button>
      <button size="sm" tone="primary" variant={view() === 'unread' ? 'solid' : 'ghost'} on:click={() => setView('unread')}>Unread</button>
    </row>
  </row>

  <match {inbox()}>
    {loading =>
      <row gap="2" align="center" style(loadingRow)>
        <spinner size="sm"></spinner>
        <small style(muted)>loading messages…</small>
      </row>
    }
    {error as err =>
      <alert tone="destructive">{err.message}</alert>
    }
    {ok =>
      <column gap="1">
        <for {visible() as m} key={m.id}>
          <row
            gap="3"
            align="center"
            style(messageRow, rowMotion)
          >
            <if {!m.read}>
              <strong style(unreadDot)></strong>
            </if>
            <column gap="0" style(fill)>
              <strong style(from)>{m.from}</strong>
              <small style(muted)>{m.subject}</small>
            </column>
            <badge tone={m.tone} variant="soft" size="sm">{m.topic}</badge>
            <button size="sm" variant="ghost" on:click={() => archive(m.id)}>Archive</button>
          </row>
        </for>
        <if {visible().length === 0}>
          <empty-state variant="inline" tone="muted">
            <strong slot="title">Inbox zero</strong>
            Nothing matches this filter.
          </empty-state>
        </if>
      </column>
    }
  </match>
</column>
