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
</scope>
<controller>
import { controller, signal, derived, query } from '@ship/sail-runtime'
import { inboxClient } from './tut-09-inbox-client'

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
      return state.value.filter(
        (m) => !archived().includes(m.id) && (filter() === 'all' || !m.read),
      )
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

  {#while-error err of inbox}
    <alert tone="destructive">{err.message}</alert>
  {/while-error}

  {#if inbox().kind === 'ok'}
    <column gap="1">
      {#for m of visible() key={m.topic}}
        <row gap="3" align="center" sx="px-3 py-2 bg-surface-1 border border-border-subtle rounded-md"
             motion:enter={{ opacity: 'fade', translate-y: 'spring:gentle', stagger: '40ms' }}>
          {#if !m.read}<span sx="inline-block w-1_5 h-1_5 rounded-full bg-primary"></span>{/if}
          <column gap="0" sx="grow-1">
            <strong sx="text-sm">{m.from}</strong>
            <small sx="text-muted-foreground">{m.subject}</small>
          </column>
          <badge tone={m.tone} variant="soft" size="sm">{m.topic}</badge>
          <button size="sm" variant="ghost" on:click={() => archive(m.id)}>Archive</button>
        </row>
      {/for}
      {#if visible().length === 0}
        <div sx="p-4 text-sm text-muted-foreground">Nothing matches this filter.</div>
      {/if}
    </column>
  {/if}
</column>
