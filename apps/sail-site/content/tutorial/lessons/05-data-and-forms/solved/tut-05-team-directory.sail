<!--
  Lesson 05 solved — all three states of a query() handle are explicit.

  <match {team()}> covers loading / error / ok arms; the value is only
  reachable behind `{ok as …}`. The form writes through bind:value /
  bind:checked and refetches by bumping a tracked signal read in the
  query's call args.
-->

<scope>
  style muteLabel = { fg(muted).type(body-sm) }
  style loadingRow = { pad(4).bg(surface).radius(md) }
  style muted = { fg(muted) }
  style memberRow = { pad(3).bg(surface).radius(md) }
  style memberName = { type(body-sm).size(grow) }
</scope>

<controller>
import { directory } from '@/compositions/tut-05-directory-client'

export default controller(() => {
    const attempt = signal(1)
    const breakApi = signal(false)
    const name = signal('')
    const role = signal<'member' | 'admin'>('member')

    // query() reads an imported client (directory.team.list) and returns a
    // state signal:
    //   { kind: 'loading' } → { kind: 'ok', value } | { kind: 'error', error }
    // Signals read INSIDE the call ARGS — attempt() and breakApi() — are the
    // tracked deps: change either and query() re-keys to a fresh cache entry
    // (a real refetch), back through 'loading'.
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
        attempt.set((n) => n + 1)
      },
    }
  })
</controller>

<column gap="4">
  <row align="center" justify="between">
    <h3>Team</h3>
    <row gap="3" align="center">
      <label style(muteLabel)>
        <checkbox bind:checked={breakApi} /> break the API
      </label>
      <button size="sm" variant="outline" on:click={() => reload()}>Reload</button>
    </row>
  </row>

  <match {team()}>
    {loading =>
      <row gap="2" align="center" style(loadingRow)>
        <spinner size="sm"></spinner>
        <small style(muted)>loading the roster…</small>
      </row>
    }
    {error as err =>
      <alert tone="danger">{err.message}</alert>
    }
    {ok as value =>
      <column gap="1">
        <for {value as m} key={m.id}>
          <row gap="3" align="center" style(memberRow)>
            <strong style(memberName)>{m.name}</strong>
            <if {m.role === 'admin'}>
              <badge tone="primary" variant="soft" size="sm">admin</badge>
            </if>
          </row>
        </for>
      </column>
    }
  </match>

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
