<!--
  Lesson 05 starter — the roster renders, but only in the happy path.

  A query() handle has THREE states, and this template handles one. Add the
  <match> loading arm and error arm boundaries, then wire the form.
-->

<controller>
import { query } from '@ship/sail-runtime'
import { directory } from './tut-05-directory-client'

export default controller(() => {
    const breakApi = signal(false)
    const name = signal('')
    const role = signal<'member' | 'admin'>('member')

    // TODO 1: this query passes constant args, so it keys once and never
    // re-fetches. Add an `attempt` signal and read attempt() (plus
    // breakApi()) INSIDE the call args — signals in the args are tracked
    // deps, so bumping either re-keys the query to a fresh cache entry
    // (a real refetch, back through 'loading').
    const team = query(() => directory.team.list({ attempt: 1, broken: false }))

    return {
      team,
      breakApi,
      name,
      role,
      // TODO 2: reload() should bump the attempt signal.
      reload: (): void => {},
      // TODO 3: add() — call directory.team.add({ name, role }), clear the
      // inputs, then bump attempt: the list refreshes through the same
      // query path as everything else.
      add: (): void => {},
    }
  })
</controller>

<scope>
  const muteLabel = style({ paint: fg(muted-foreground), text: fontSize(sm) })
  const memberRow = style({ box: padX(3).padY(2), paint: bg(surface-1).radius(md) })
  const memberName = style({ text: fontSize(sm) })
</scope>

<column gap="4">
  <row align="center" justify="between">
    <h3>Team</h3>
    <row gap="3" align="center">
      <label style(muteLabel)>
        <input type="checkbox" bind:checked={breakApi} /> break the API
      </label>
      <button size="sm" variant="outline" on:click={() => reload()}>Reload</button>
    </row>
  </row>

  <!-- TODO 4: <match {team()}> {loading => — a spinner row, so the first
       paint isn't blank. -->

  <!-- TODO 5: {error as err => — an <alert tone="destructive">
       showing err.message. -->

  <if {team().kind === 'ok'}>
    <column gap="1">
      <for {team().value as m} key={m.id}>
        <row gap="3" align="center" style(memberRow)>
          <strong style(memberName, { box: grow(1) })>{m.name}</strong>
          <if {m.role === 'admin'}><badge tone="primary" variant="soft" size="sm">admin</badge></if>
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
      <button type="submit" size="sm" variant="solid" tone="primary">Add</button>
    </row>
  </form>
</column>
