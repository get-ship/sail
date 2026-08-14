<!--
  Lesson 09 solved — the screen as an owner would ship it.

  Compare with the starter: the title flows from the prop; all three
  query() states render; the {#for} keys on identity (m.id); rows exit as
  well as enter; and the empty case is the semantic <empty-state>, not a
  hand-rolled div.
-->
<scope>
  props: {
    title = 'Support inbox'
  }

  style loadingRow = { pad(4).bg(surface-1).radius(md) }
  style muted = { fg(muted-foreground) }
  style messageRow = { pad({ x: 3, y: 2 }).bg(surface-1).border({ color: border-subtle, width: 1 }).radius(md) }
  style unreadDot = { display(inline-block).size(1_5).radius(full).bg(primary) }
  style from = { type({ size: sm }) }
</scope>
<controller>
import { inboxClient } from '@/compositions/tut-09-inbox-client'

export default controller(() => {
    const inbox = query(() => inboxClient.messages.list())
    const filter = signal<'all' | 'unread'>('all')
    const archived = signal<number[]>([])

    // Visible list composes the live value with local state: archive hides
    // rows client-side so exit animation can play; filter narrows to unread.
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
    <h3>{title()}</h3>
    <row gap="1">
      <button size="sm" tone="primary" variant={filter() === 'all' ? 'solid' : 'ghost'} on:click={() => setFilter('all')}>All</button>
      <button size="sm" tone="primary" variant={filter() === 'unread' ? 'solid' : 'ghost'} on:click={() => setFilter('unread')}>Unread</button>
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
            style(messageRow, { animate: enter({ opacity: 'fade', translate-y: 'spring:gentle', stagger: '40ms' }).exit({ opacity: 'fade-out' }) })
          >
            <if {!m.read}>
              <span style(unreadDot)></span>
            </if>
            <column gap="0" style({ grow(1) })>
              <strong style(from)>{m.from}</strong>
              <small style(muted)>{m.subject}</small>
            </column>
            <badge tone={m.tone} variant="soft" size="sm">{m.topic}</badge>
            <button size="sm" variant="ghost" on:click={() => archive(m.id)}>Archive</button>
          </row>
        </for>
        <if {visible().length === 0}>
          <empty-state variant="inline" tone="muted">
            <span slot="title">Inbox zero</span>
            Nothing matches this filter.
          </empty-state>
        </if>
      </column>
    }
  </match>
</column>
