<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) }
  style monoLabel = { box: mb(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style livePanel = { box: pad(6), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) }
  style liveMuted = { paint: fg(muted-foreground), text: fontSize(sm) }
  style liveValue = { paint: fg(accent), text: fontFamily(mono).fontSize(lg) }
  style callout = { box: pad(3).padX(4), paint: borderWidth(1).border(border).borderLWidth(3).borderL(primary).radius(md).fg(muted-foreground), text: fontSize(sm) }
  style footNav = { box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) }
</scope>

<controller>
// Live demo: deck grants `route`, so `ctx.route.pathname()` is a real capability.

import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from '../../../../src/compositions/sail-language-capabilities-snippets'

export default controller((ctx: ControllerScope<Scope>) => {
  const here = derived(() => ctx.route.pathname())
  return { code, here }
})
</controller>

<column gap="5" style(page)>
  <p style(eyebrow)>Concepts</p>
  <h1>Capabilities</h1>
  <p style(lead)>
    A <strong>capability</strong> is a named, typed value that whatever runs your
    app puts on your scope. Props come from the parent that renders you;
    capabilities come from the <strong>host</strong> that mounts you. You read
    them the same way you read anything else in scope — and a name nobody
    granted is a compile error, not an undefined at runtime.
  </p>

  <h2>One, live</h2>
  <p>
    This documentation site is a <a href="/docs/app">deck</a> app, and deck grants
    every route a capability called <code>route</code>. Nothing on this page
    imported it or declared it. It is simply there, typed, because the host put
    it there:
  </p>
  <grid cols="2" gap="4">
    <column>
      <p style(monoLabel)>Live — read from the real host</p>
      <column gap="2" style(livePanel)>
        <p style(liveMuted)>ctx.route.pathname() says:</p>
        <p style(liveValue)>{here()}</p>
      </column>
    </column>
    <column>
      <p style(monoLabel)>Source</p>
      <code-block code={code.read}></code-block>
    </column>
  </grid>

  <h2>What is actually granted</h2>
  <p>
    A capability is three things: a <strong>name</strong>, the
    <strong>TypeScript type</strong> that name carries, and — when the type is
    not self-contained — the module that type is imported from. The compiler
    writes each one into the <code>Scope</code> it generates next to your
    <code>.sail</code> file, so your editor and <code>tsc</code> see the same
    members the runtime will hand you.
  </p>
  <p>
    Here is the generated <code>Scope</code> for a view in a deck app that
    declares one data client named <code>api</code>:
  </p>
  <code-block code={code.scope}></code-block>
  <p>
    Four members, none of them written by hand. <code>route</code> and
    <code>navigate</code> come from deck. <code>api</code> comes from the app’s
    own manifest — one flat <code>ctx</code> member per declared client, so a
    template reads <code>ctx.api.demo.clock()</code> directly.
  </p>
  <code-block code={code.manifest}></code-block>

  <h2>A name nobody granted</h2>
  <p>
    Reading a capability that was never granted does not fail at runtime, and it
    does not silently produce <code>undefined</code>. The compiler refuses, and
    the refusal lists what <em>is</em> in scope — so the fix is in the error:
  </p>
  <code-block code={code.refusal}></code-block>

  <h2>Who grants them</h2>
  <p>
    Sail grants nothing. It defines the mechanism and stays ignorant of what any
    particular name means — the compiler names no capability and no provider.
    The host resolves the list and hands it in. Two hosts you will meet:
  </p>
  <column gap="3">
    <card padding="md">
      <p><strong>deck</strong> grants <code>route</code> and
      <code>navigate</code> to every route. Their fields are signals, so a
      client-side navigation updates them in place. For the shape of those types
      and how routing works, see
      <a href="/docs/app/routing">deck · Routing</a> and
      <a href="/docs/app/reactivity">deck · Reactivity</a> — that is deck’s
      documentation to own, not Sail’s.</p>
    </card>
    <card padding="md">
      <p><strong>Your app</strong> grants one capability per client it declares
      in its <code>app.manifest.jsonc</code>. The manifest maps a name to a real
      ESM import; the compiler turns each entry into a typed scope member.</p>
    </card>
  </column>
  <p>
    Any host can do this — the mechanism is not tied to deck, and nothing about
    it is specific to routing or to data clients. A host that mounts Sail views
    inside some other application grants its own names the same way.
  </p>

  <h2>Why it works this way</h2>
  <p>
    One mechanism serves every host, so there is nothing new to learn per host —
    only new names. Everything is typed end to end: the same list that produces
    the generated <code>Scope</code> produces the runtime scope, so the type you
    code against and the value you receive cannot drift apart. Nothing is
    global, and nothing is ambient by magic — a name is in scope because a host
    granted it, and the error message names the grantor’s absence rather than
    leaving you to guess.
  </p>
  <p>
    Granting is also not the same as paying for it. The compiler imports only
    the capabilities a given template actually references, per view. Declaring
    five clients and using one costs you one import.
  </p>

  <p style(callout)>
    Today, a name your own controller or template declares takes precedence over
    a host capability of the same name — the author’s value shadows the granted
    one, silently.
  </p>

  <row justify="between" gap="2" style(footNav)>
    <a variant="default" href="/docs/concepts/data">← Data</a>
    <a variant="solid" tone="primary" href="/docs/concepts/theming">Theming →</a>
  </row>
</column>
