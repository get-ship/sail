<controller>
// /data — a real async source with a visible delay, so the reader watches
// loading → ok and loading → error actually happen. `query()` derives its
// cache key from the client member path; the `attempt()`/`sabotage()`
// signals read in the call ARGS are tracked deps — bumping either mints a
// fresh key, so the state walks through loading again (that is the refetch).

import { controller, query, signal, type ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { usersDemo } from './users-demo'
import { code } from './snippets'

export default controller((_ctx: ViewScope<Scope>) => {
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

<column sx="max-w-breakpoint-tablet mx-auto" gap="5">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">Building</p>
  <h1>Live data</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
    Asynchronous data has three states — loading, error, and loaded — and
    forgetting one is where interfaces rot. SAIL makes all three explicit.
    <code>query()</code> turns a client call into a reactive state signal, and
    the template renders each state on its own terms.
  </p>

  <h2>The three states, live</h2>
  <grid cols="2" gap="4">
    <column>
      <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground mb-2">Live — a 900 ms fake API</p>
      <column sx="p-6 border border-border rounded-lg min-h-8 bg-surface-1" gap="3">
        <row gap="2">
          <button on:click={reload}>Reload</button>
          <button on:click={breakIt} tone="destructive" variant="outline">Fail the next one</button>
        </row>
        {#while-loading of users}
          <row gap="2" align="center"><spinner></spinner> <p sx="text-sm text-muted-foreground">loading…</p></row>
        {/while-loading}
        {#while-error err of users}
          <alert tone="destructive">{err.message}</alert>
        {/while-error}
        {#if users().kind === 'ok'}
          <row gap="2">
            {#for name of users().value key={name}}
              <badge>{name}</badge>
            {/for}
          </row>
        {/if}
      </column>
    </column>
    <column>
      <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground mb-2">Source</p>
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
    <code>&#123;#while-loading of x&#125;</code> and
    <code>&#123;#while-error err of x&#125;</code> render the pending and failed
    states; <code>&#123;#if x().kind === 'ok'&#125;</code> guards the loaded one,
    where <code>x().value</code> holds the data. The three cases are structural —
    you can't render the list without having handled loading and error first.
  </p>
  <code-block code={code.states}></code-block>

  <h2>Live server data</h2>
  <p>
    For data that lives on the server and changes over time,
    <code>data()</code> gives you a signal that's server-rendered at request
    time and then patched over a stream — no polling, no manual wiring. The
    server half — the endpoint, the patch feed, SSR — is <a href="/deck">deck</a>'s
    job; the view just reads the signal.
  </p>
  <code-block code={code.server}></code-block>

  <p sx="border border-border border-l-3 border-l-primary rounded-md p-3 px-4 text-sm text-muted-foreground">
    <strong>The state is the contract.</strong> Because the loaded value is only
    reachable through <code>kind === 'ok'</code>, a template physically can't
    render data it hasn't confirmed has arrived. The loading spinner and the
    error message aren't good manners — they're the only way through.
  </p>

  <row sx="mt-8 pt-4 border-t border-t-border">
    <a variant="default" href="/reactivity">← Reactivity</a>
    <a variant="default" href="/components">Component library →</a>
  </row>
</column>
