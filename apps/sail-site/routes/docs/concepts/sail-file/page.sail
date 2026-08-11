<!--
  /docs/concepts/sail-file — components chapter via departures-board.sail:
  template → scope → controller → flow → nesting.
-->

<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) }
  style footerNote = { box: mt(3), paint: fg(muted-foreground), text: fontSize(xs) }
  style footNote = { box: mt(4), paint: fg(muted-foreground), text: fontSize(sm) }
  style footNav = { box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from '../../../../src/compositions/sail-language-sail-file-snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  return { code }
})
</controller>

<column gap="5" style(page)>
  <p style(eyebrow)>The language</p>
  <h1>Components</h1>
  <p style(lead)>
    A component is one <code>.sail</code> file. This chapter builds this one —
    a departures board that runs itself: the boarding flight leaves, the next
    one starts boarding, a new city rolls onto the bottom.
  </p>

  <departures-board title="Departures — SFO">
    <p slot="footer" style(footerNote)>
      Gate changes are announced. Allegedly.
    </p>
  </departures-board>

  <p>
    Everything the board does — the rotating list, the departed counter, the
    status badges, the footer line — lives in one file. The rest of this
    chapter is that file, part by part.
  </p>

  <h2>The shape</h2>
  <p>
    A <code>.sail</code> file has up to three parts, each with one job. The
    template is the markup. <code>&lt;scope&gt;</code> declares the
    component’s contract. <code>&lt;controller&gt;</code> holds its behavior.
    Only the template is required — a static component is just markup.
  </p>
  <code-block code={code.shape}></code-block>

  <h2>The template</h2>
  <p>
    The board’s template, in full. Braces are expressions:
    <code>&#123;departed()&#125;</code> reads a value, and when that value
    changes, only that text node updates. Lists are spelled
    <code>&lt;for&gt;</code> with a required <code>key</code> — as rows
    shift up the board, the key tells the renderer which row is which, so DOM
    nodes move instead of being rewritten. <code>&lt;match&gt;</code>
    picks a badge per status. Blocks are covered in depth in
    <a href="/docs/concepts/templates">Templates</a>. Everything else is
    plain HTML — elements, attributes, text.
  </p>
  <code-block code={code.template}></code-block>

  <h2>The scope</h2>
  <p>
    <code>&lt;scope&gt;</code> is the component’s static surface — everything
    that is true about it before it runs. The board’s scope, in full:
  </p>
  <code-block code={code.scope}></code-block>
  <p>
    Read it top to bottom: the <code>Flight</code> type is declared, then
    used two lines later — <code>seed</code> lets a parent hand the board its
    opening rows. Types live in scope because the contract needs them: a
    prop like <code>seed</code> is meaningless without <code>Flight</code>,
    and the controller’s state (next section) speaks the same type. Declared
    once here, it is usable everywhere in the file without an import.
  </p>
  <p>
    Both props have defaults, so both attributes are optional — the live
    board above sets <code>title</code> and skips <code>seed</code>. Leave
    the default off (<code>airport: string</code>) and the prop becomes
    required: a parent that doesn’t fill it does not compile. Scope has one
    more member kind, <strong>variants</strong> — a component’s named looks —
    covered with the rest of <a href="/docs/concepts/styling">styling</a>.
  </p>

  <h2>The controller</h2>
  <p>
    The controller owns the component’s state and behavior. It runs once per
    mounted instance, and <strong>whatever it returns is the template’s
    entire vocabulary</strong> — the template can read
    <code>flights()</code> and <code>departed()</code> because the controller
    returned them, and nothing it didn’t return is reachable.
  </p>
  <p>
    The board keeps two signals and one effect. A <strong>signal</strong> is
    a value that knows who reads it. An <strong>effect</strong> is a
    computation that runs once, tracks every signal it read, and re-runs when
    any of them change; if it returns a function, that runs first as cleanup.
    The board’s effect tracks nothing — it just starts the clock, and hands
    back the cleanup that stops it when the board unmounts:
  </p>
  <code-block code={code.controller}></code-block>
  <p>
    Two things to notice. Props arrive on <code>ctx</code> —
    <code>ctx.seed&#40;&#41;</code> is the prop the scope declared, readable like any
    signal. And <code>signal&lt;Flight[]&gt;</code> is the payoff of putting
    the type in scope: contract and state speak the same <code>Flight</code>,
    with no import in sight.
  </p>

  <h2>The flow</h2>
  <p>
    Here is one tick of the board, live, as a sequence. Watch what lights
    up — and what doesn’t:
  </p>
  <flow-trace></flow-trace>
  <p>
    A write, the reconcile it triggers, one repaint. There is no fourth
    step — no re-render, no lifecycle. That is the direction of everything
    in Sail: <strong>the template reads state; handlers and effects write
    it</strong>. When you read an unfamiliar component, find the writes in
    its controller and you have the complete list of ways it can change.
  </p>

  <h2>Nested components</h2>
  <p>
    Compiling a <code>.sail</code> file produces a <em>custom element</em> —
    a real HTML tag, registered with the browser itself, that works anywhere
    HTML works. There is no import to wire up and no wrapper to render
    through: using a component is writing its tag. This page’s own source
    contains exactly what you’d expect:
  </p>
  <code-block code={code.usage}></code-block>
  <p>
    Children are the other half of nesting. The board places a named
    <code>&lt;slot&gt;</code> after its rows:
  </p>
  <code-block code={code.slotDef}></code-block>
  <p>
    …and whoever uses the board projects content into it. The footer line on
    the live board above is not the board’s — it belongs to this page:
  </p>
  <code-block code={code.slotUse}></code-block>
  <p>
    To write one of these yourself, from an empty file to a running tag,
    follow <a href="/docs/guide/building-a-component">Building a
    component</a>.
  </p>

  <p style(footNote)>
    For the full surface: <a href="/docs/reference/scope">reference → Scope grammar</a>.
  </p>

  <row justify="end" gap="2" style(footNav)>
    <a variant="solid" tone="primary" href="/docs/concepts/state">State →</a>
  </row>
</column>
