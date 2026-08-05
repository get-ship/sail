<controller>
// /data — a real async source with a visible delay, so the reader watches
// loading → ok and loading → error actually happen. `query()` derives its
// cache key from the client member path; the `attempt()`/`sabotage()`
// signals read in the call ARGS are tracked deps — bumping either mints a
// fresh key, so the state walks through loading again (that is the refetch).

import { query, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { usersDemo } from './users-demo'
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

<column sx={style.maxW(breakpoint-tablet).mx(auto)} gap="5">
  <p sx={style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)}>Concepts</p>
  <h1>Data</h1>
  <p sx={style.fontSize(lg).fg(muted-foreground).leading(relaxed)}>
    Data that arrives over the network is in one of three states at any
    moment: still loading, failed, or loaded. <code>query()</code> turns an
    async call into a signal that carries which state it is in, and the
    template renders each state explicitly — including the two that are easy
    to forget.
  </p>

  <h2>The three states, live</h2>
  <grid cols="2" gap="4">
    <column>
      <p sx={style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(muted-foreground).mb(2)}>Live — a 900 ms fake API</p>
      <column sx={style.pad(6).borderWidth(1).border(border).radius(lg).minH(8).bg(surface-1)} gap="3">
        <row gap="2">
          <button on:click={reload}>Reload</button>
          <button on:click={breakIt} tone="destructive" variant="outline">Fail the next one</button>
        </row>
        <while-loading {users}>
          <row gap="2" align="center"><spinner></spinner> <p sx={style.fontSize(sm).fg(muted-foreground)}>loading…</p></row>
        </while-loading>
        <while-error {users}> as="err">
          <alert tone="destructive">{err.message}</alert>
        </while-error>
        <if {users().kind === 'ok'}>
          <row gap="2">
            <for {users().value} as="name" key={name}>
              <badge>{name}</badge>
            </for>
          </row>
        </if>
      </column>
    </column>
    <column>
      <p sx={style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(muted-foreground).mb(2)}>Source</p>
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
    <code>&lt;while-loading &#123;x&#125;&gt;</code> and
    <code>&lt;while-error &#123;x&#125; as="err"&gt;</code> render the pending and failed
    states; <code>&lt;if &#123;x().kind === 'ok'&#125;&gt;</code> guards the loaded one,
    where <code>x().value</code> holds the data. The three cases are structural —
    you can't render the list without having handled loading and error first.
  </p>
  <code-block code={code.states}></code-block>

  <p sx={style.borderWidth(1).border(border).borderLWidth(3).borderL(primary).radius(md).pad(3).padX(4).fontSize(sm).fg(muted-foreground)}>
    The loaded value is only reachable through <code>kind === 'ok'</code>,
    so a template cannot render data it hasn't confirmed has arrived —
    handling loading and error is built into the shape, not left to
    discipline.
  </p>

  <p>
    <code>query()</code> is the read side of the story; the write side —
    forms that validate, submit, and carry the same pending and failure
    states — is the <a href="/sail/guide/forms">Forms guide</a>.
  </p>
  <p sx={style.mt(4).fontSize(sm).fg(muted-foreground)}>
    For the author runtime (tier B): <a href="/sail/reference/runtime">reference → Author runtime</a>.
  </p>

  <row sx={style.mt(8).padT(4).borderTWidth(1).borderT(border)} justify="between" gap="2">
    <a variant="default" href="/sail/language/actions">← Actions</a>
    <a variant="solid" tone="primary" href="/sail/language/capabilities">Capabilities →</a>
  </row>
</column>
