<!--
  Actions: use: attach, author actions, L1 private vs public attach-anywhere.
-->

<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) }
  style demoField = { box: maxW(panel-max-w-sm) }
  style footNote = { box: mt(4), paint: fg(muted-foreground), text: fontSize(sm) }
  style footNav = { box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from '../../../../src/compositions/sail-language-actions-snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  return { code }
})
</controller>

<column gap="5" style(page)>
  <p style(eyebrow)>Concepts</p>
  <h1>Actions</h1>
  <p style(lead)>
    An action is a function that receives a DOM element, does something to
    it — attach a listener, start an observer, wire a third-party widget —
    and returns the cleanup that undoes it. <code>use:</code> attaches one
    to an element. This is how Sail code touches the DOM directly.
  </p>
  <code-block code={code.attach}></code-block>

  <h2>One working</h2>
  <p>
    <code>use:autosize</code> is a built-in. Type a few lines and the box
    grows to fit — the action measures the content and sets the height, and
    when the element leaves the page, the action’s cleanup runs with it:
  </p>
  <textarea use:autosize placeholder="Type a few lines here…" style(demoField) />

  <h2>Why there is no ref</h2>
  <p>
    Many frameworks give you a <em>ref</em>: an object you attach to an
    element, store somewhere, and read later when you want to touch the DOM.
    That splits one job across three places — the attachment, the storage,
    and the code that finally uses it — and the cleanup is yours to
    remember.
  </p>
  <p>
    An action keeps the job in one place. The element arrives as the
    function’s argument, the work happens right there, and the cleanup is
    the return value. There is nothing to store and nothing to forget to
    release.
  </p>

  <h2>Writing one</h2>
  <p>
    An action is a plain function — there is no registry to add it to and
    no interface to implement:
  </p>
  <code-block code={code.author}></code-block>
  <p>
    An action can also take a parameter. When the parameter is reactive,
    the action’s <code>update</code> runs each time the value changes —
    this is how a positioning action follows its anchor:
  </p>
  <code-block code={code.param}></code-block>

  <h2>Widgets own their private behavior</h2>
  <p>
    L1 widgets like <code>&lt;tabs&gt;</code>, <code>&lt;dropdown&gt;</code>,
    <code>&lt;combobox&gt;</code>, <code>&lt;pagination&gt;</code>, and
    <code>&lt;radio-group&gt;</code> keep their keyboard/ARIA wiring as
    <em>closed implementation</em> inside the widget — not as public
    <code>use:</code> actions you reattach by hand. Prefer the widget tag.
    Public <code>use:</code> stays for reusable, attach-anywhere capabilities
    (<code>use:anchor</code>, <code>use:clickOutside</code>,
    <code>use:autosize</code>, …). Author actions live under
    <code>src/actions/</code> and compile under a strict floor (no style, no
    state primitives, no network, host-only).
  </p>
  <code-block code={code.stdlib}></code-block>

  <p>
    The boundary with <a href="/docs/concepts/state">compositions</a> is
    worth restating: a composition builds state and has no access to the
    DOM; an action receives an element and works on it. If the reusable
    thing you are writing needs an element, it is an action.
  </p>

  <p style(footNote)>
    For the full surface: <a href="/docs/reference/actions">reference → Actions</a>.
  </p>

  <row justify="between" gap="2" style(footNav)>
    <a variant="default" href="/docs/concepts/animation">← Animation</a>
    <a variant="solid" tone="primary" href="/docs/concepts/data">Data →</a>
  </row>
</column>
