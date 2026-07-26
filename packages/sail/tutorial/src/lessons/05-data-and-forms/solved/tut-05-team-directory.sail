<!--
  Lesson 05 solved — all three states of a query() handle are explicit.

  {#while-loading} covers the first paint and every refetch;
  {#while-error} covers rejection; the value is only reachable behind
  kind === 'ok'. The form writes through bind:value / bind:checked and
  refetches by bumping a tracked signal read in the query's call args.
-->

<controller>
import { query } from '@ship/sail-runtime'
import { directory } from './tut-05-directory-client'

export default controller(() => {
    const attempt = signal(1)
    const breakApi = signal(false)
    const name = signal('')
    const role = signal<'member' | 'admin'>('member')

    // query() reads from an imported client (directory.team.list) and
    // hands back a state signal:
    //   { kind: 'loading' } → { kind: 'ok', value } | { kind: 'error', error }
    // The signals read INSIDE the call ARGS — attempt() and breakApi() —
    // are the tracked deps: change either and query() re-keys to a fresh
    // cache entry (a real refetch), back through 'loading'. That is the
    // whole refetch story — no refetch() method, just a signal you bump.
    const team = query(() => directory.team.list({ attempt: attempt(), broken: breakApi() }))

    const canAdd = derived(() => name().trim().length > 0)

    return {
      team,
      breakApi,
      name,
      role,
      canAdd,
      reload: (): void => attempt.set((n) => n + 1),
      add: (): void => {
        if (!canAdd()) return
        directory.team.add({ name: name().trim(), role: role() })
        name.set('')
        role.set('member')
        attempt.set((n) => n + 1) // refetch: bumping attempt re-keys the query, same path
      },
    }
  })
</controller>

<scope>
  const muteLabel = style.fontSize(sm).fg(muted-foreground)
  const loadingRow = style.pad(4).bg(surface-1).radius(md)
  const muted = style.fg(muted-foreground)
  const memberRow = style.padX(3).padY(2).bg(surface-1).radius(md)
  const memberName = style.fontSize(sm)
</scope>

<column gap="4">
  <row align="center" justify="between">
    <h3>Team</h3>
    <row gap="3" align="center">
      <label sx={muteLabel}>
        <input type="checkbox" bind:checked={breakApi} /> break the API
      </label>
      <button size="sm" variant="outline" on:click={() => reload()}>Reload</button>
    </row>
  </row>

  <while-loading of="team">
    <row gap="2" align="center" sx={loadingRow}>
      <spinner size="sm"></spinner>
      <small sx={muted}>loading the roster…</small>
    </row>
  </while-loading>

  <while-error as="err" of="team">
    <alert tone="destructive">{err.message}</alert>
  </while-error>

  <if cond={team().kind === 'ok'}>
    <column gap="1">
      <for as="m" of={team().value} key={m.id}>
        <row gap="3" align="center" sx={memberRow}>
          <strong grow sx={memberName}>{m.name}</strong>
          <if cond={m.role === 'admin'}><badge tone="primary" variant="soft" size="sm">admin</badge></if>
        </row>
      </for>
    </column>
  </if>

  <form on:submit|prevent={add}>
    <row gap="2" align="center">
      <input placeholder="Add a teammate" bind:value={name} size="sm" />
      <select bind:value={role} size="sm">
        <option value="member">member</option>
        <option value="admin">admin</option>
      </select>
      <button type="submit" size="sm" variant="solid" tone="primary" disabled={!canAdd()}>Add</button>
    </row>
  </form>
</column>
