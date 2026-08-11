<!--
  Errors: throws vs async states. Async failures stay on Data’s match error arm.
-->

<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) }
  style monoLabel = { box: mb(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style livePanel = { box: pad(6).minH(8), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) }
  style callout = { box: pad(3).padX(4), paint: borderWidth(1).border(border).borderLWidth(3).borderL(primary).radius(md).fg(muted-foreground), text: fontSize(sm) }
  style footNav = { box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from '../../../../src/compositions/sail-language-errors-snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  const armed = signal(false)
  return {
    code,
    armed,
    arm: () => armed.set(true),
    disarm: () => armed.set(false),
    // Throws while the protected branch renders (demo for the boundary).
    boom: (): string => { throw new Error('the demo component threw') },
  }
})
</controller>

<column gap="5" style(page)>
  <p style(eyebrow)>Concepts</p>
  <h1>Errors</h1>
  <p style(lead)>
    Things fail two ways. Data can fail to arrive — that is a state, and the
    <a href="/docs/concepts/data">Data</a> chapter renders it with
    <code>&lt;match error arm&gt;</code>. Or code can throw — an exception in
    a component. This chapter is about the second kind: what catches a throw,
    and what the page shows instead of nothing.
  </p>

  <h2>When a component throws</h2>
  <p>
    A throw while a template branch renders (an <code>&lt;if&gt;</code>
    branch, a <code>&lt;for&gt;</code> row), while a controller sets up
    or mounts, or while an effect re-runs travels up the tree to the nearest
    <code>&lt;error-boundary&gt;</code>. The boundary swaps its content for a
    fallback; the rest of the page keeps running. A throw in one
    <code>&lt;for&gt;</code> row skips that row and renders the healthy
    ones.
  </p>
  <p>
    Two places a throw is <em>not</em> caught: event handlers (a throw in an
    <code>on:click</code> is an ordinary uncaught browser error — check the
    console) and async code, which never throws into the tree at all — a
    failed fetch becomes the error state below.
  </p>

  <h2>&lt;error-boundary&gt;, live</h2>
  <grid cols="2" gap="4">
    <column>
      <p style(monoLabel)>Live — throw, catch, retry</p>
      <column gap="3" style(livePanel)>
        <error-boundary on-error-reset={disarm}>
          <if {armed()}><p>{boom()}</p></if>
          <if {!armed()}>
            <p>This subtree is protected.</p>
            <button on:click={arm} tone="destructive" variant="outline">
              Throw during render
            </button>
          </if>
        </error-boundary>
      </column>
    </column>
    <column>
      <p style(monoLabel)>Source</p>
      <code-block code={code.demo}></code-block>
    </column>
  </grid>
  <p>
    Wrap the risky part, nothing else. On a catch the boundary swaps its
    content for the built-in fallback — a card with a Retry button. The
    <code>message</code> prop sets the fallback text when the built-in card is
    in use; a <code>slot="fallback"</code> replaces the whole card with your own
    markup.
  </p>
  <code-block code={code.fallback}></code-block>

  <h2>Without a boundary</h2>
  <p>
    An uncaught throw always lands in the console. In development the dev
    server also overlays it on the page — a fixed panel naming the component
    and the phase that threw — so a broken effect can’t fail invisibly; the
    overlay ships zero bytes in production. And every route in a deck app is
    wrapped in a default boundary of its own, so a page that throws degrades
    to an error card with a Retry — with the message and stack in
    development, redacted to a generic line in production.
  </p>

  <h2>Async errors are states, not throws</h2>
  <p>
    A rejected <code>query()</code> doesn’t trip a boundary — it sets the
    signal to <code>&#123; kind: 'error' &#125;</code>, and the template
    renders that state where it happens:
  </p>
  <code-block code={code.async}></code-block>
  <p style(callout)>
    The two systems don’t overlap: a failed fetch shows the
    <code>&lt;match error arm&gt;</code> branch, never the boundary
    fallback; a synchronous throw trips the boundary, never the error state.
    Expected failures get rendered as states; unexpected ones get contained
    by boundaries.
  </p>

  <row justify="between" gap="2" style(footNav)>
    <a variant="default" href="/docs/concepts/data">← Data</a>
    <a variant="solid" tone="primary" href="/docs/concepts/theming">Theming →</a>
  </row>
</column>
