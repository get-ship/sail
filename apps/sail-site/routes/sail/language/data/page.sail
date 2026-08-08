<controller>
// /data — a real async source with a visible delay, so the reader watches
// loading → ok and loading → error actually happen. `query()` derives its
// cache key from the client member path; the `attempt()`/`sabotage()`
// signals read in the call ARGS are tracked deps — bumping either mints a
// fresh key, so the state walks through loading again (that is the refetch).

import { query, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { usersDemo } from '../../../../src/compositions/users-demo'
import { code } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
    const attempt = signal(1)
    const sabotage = signal(false)
    const users = query(() => usersDemo.users.list({ attempt: attempt(), fail: sabotage() }))
    return {
      code,
      users,
      reload: () => {
        sabotage.set(false)
        attempt.set((a) => a + 1)
      },
      breakIt: () => {
        sabotage.set(true)
        attempt.set((a) => a + 1)
      },
    }
})
</controller>

<column gap="5" style({ box: maxW(breakpoint-tablet).mx(auto) })>
  <p style({ box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })>Concepts</p>
  <h1>Data</h1>
  <p style({ paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) })>
    Data that arrives over the network is in one of three states at any
    moment: still loading, failed, or loaded. <code>query()</code> turns an
    async call into a signal that carries which state it is in, and the
    template renders each state explicitly — including the two that are easy
    to forget.
  </p>

  <h2>The three states, live</h2>
  <grid cols="2" gap="4">
    <column>
      <p style({ box: mb(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })>Live — a 900 ms fake API</p>
      <column gap="3" style({ box: pad(6).minH(8), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) })>
        <row gap="2">
          <button on:click={reload}>Reload</button>
          <button on:click={breakIt} tone="destructive" variant="outline">Fail the next one</button>
        </row>
        <match {users()}>
  {loading => <row gap="2" align="center"><spinner></spinner> <p style({ paint: fg(muted-foreground), text: fontSize(sm) })>loading…</p></row>}
  {error as err =>
          <alert tone="destructive">{err.message}</alert>}
  {ok as value => <row gap="2">
            <for {value as name} key={name}>
              <badge>{name}</badge>
            </for>
          </row>}
</match>
      </column>
    </column>
    <column>
      <p style({ box: mb(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })>Source</p>
      <code-block code={code.demo}></code-block>
    </column>
  </row>

  <h2>Wrap an async source</h2>
  <p>
    <code>query()</code> takes a thunk that calls a client method and hands
    back a signal of a tagged state:
    <code>&#123; kind: 'loading' &#125;</code>, then either
    <code>&#123; kind: 'ok', value &#125;</code> or
    <code>&#123; kind: 'error', error &#125;</code>.
    It starts loading immediately and settles when the source resolves.
  </p>
  <code-block code={code.source}></code-block>

  <h2>Render the three states</h2>
  <p>
    One <code>&lt;match &#123;users()&#125;&gt;</code> carries the three arms:
    <code>&#123;loading =&gt; …&#125;</code>,
    <code>&#123;error as err =&gt; …&#125;</code>, and
    <code>&#123;ok as value =&gt; …&#125;</code>. Arms are structural — you
    cannot render the list without having handled loading and error first.
    There is no <code>while-loading</code> / sibling-case form; those are dead.
  </p>
  <code-block code={code.states}></code-block>

  <p style({ box: pad(3).padX(4), paint: borderWidth(1).border(border).borderLWidth(3).borderL(primary).radius(md).fg(muted-foreground), text: fontSize(sm) })>
    The loaded value is only reachable on the <code>ok</code> arm
    (<code>as value</code>), so a template cannot render data it hasn’t
    confirmed has arrived — handling loading and error is built into the
    shape, not left to discipline.
  </p>

  <p>
    <code>query()</code> is the read side of the story; the write side —
    forms that validate, submit, and carry the same pending and failure
    states — is the <a href="/sail/guide/forms">Forms guide</a>.
  </p>
  <p style({ box: mt(4), paint: fg(muted-foreground), text: fontSize(sm) })>
    For the author runtime (tier B): <a href="/sail/reference/runtime">reference → Author runtime</a>.
  </p>

  <row justify="between" gap="2" style({ box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) })>
    <a variant="default" href="/sail/language/actions">← Actions</a>
    <a variant="solid" tone="primary" href="/sail/language/capabilities">Capabilities →</a>
  </row>
</column>
