<!--
  /sail/language/sail-file — chapter 1: Components. Builds ONE bespoke
  component (<departures-board>, src/departures-board/) and walks its file
  slice by slice: template → scope → controller → flow → nesting. The live
  board leads; every snippet is a slice of the real file. No backlinks to
  other chapters' demos, no negative-space claims about other frameworks.
-->

<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  return { code }
})
</controller>

<column sx="max-w-breakpoint-tablet mx-auto" gap="5">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">The language</p>
  <h1>Components</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
    A component is one <code>.sail</code> file. This chapter builds this one —
    a departures board that runs itself: the boarding flight leaves, the next
    one starts boarding, a new city rolls onto the bottom.
  </p>

  <departures-board title="Departures — SFO">
    <p slot="footer" sx="mt-3 text-xs text-muted-foreground">
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
    component's contract. <code>&lt;controller&gt;</code> holds its behavior.
    Only the template is required — a static component is just markup.
  </p>
  <code-block code={code.shape}></code-block>

  <h2>The template</h2>
  <p>
    The board's template, in full. Braces are expressions:
    <code>&#123;departed()&#125;</code> reads a value, and when that value
    changes, only that text node updates. Lists are spelled
    <code>&#123;#for&#125;</code> with a required <code>key</code> — as rows
    shift up the board, the key tells the renderer which row is which, so DOM
    nodes move instead of being rewritten. <code>&#123;#match&#125;</code>
    picks a badge per status. The blocks get their full treatment in
    <a href="/sail/language/templates">Templates</a>; here, notice how little
    of this is not plain HTML.
  </p>
  <code-block code={code.template}></code-block>

  <h2>The scope</h2>
  <p>
    <code>&lt;scope&gt;</code> is the component's static surface — everything
    that is true about it before it runs. The board declares a type and one
    prop:
  </p>
  <code-block code={code.scope}></code-block>
  <p>
    Three kinds of member live here. <strong>Props</strong> are the
    component's inputs. A prop with a default is an optional attribute; a
    prop with a type and no default is a contract the parent must fill:
  </p>
  <code-block code={code.scopeTyped}></code-block>
  <p>
    <strong>Types</strong> — <code>Flight</code> is ordinary TypeScript, and
    because it is declared in scope the controller and the template's
    expressions can use it without an import. And <strong>variants</strong> —
    a component's named looks, covered with the rest of
    <a href="/sail/language/styling">styling</a>. Using a prop nobody
    declared, or passing the wrong shape, is a compile error, not a runtime
    surprise.
  </p>

  <h2>The controller</h2>
  <p>
    The controller owns the component's state and behavior. It runs once per
    mounted instance, and <strong>whatever it returns is the template's
    entire vocabulary</strong> — the template can read
    <code>flights()</code> and <code>departed()</code> because the controller
    returned them, and nothing it didn't return is reachable.
  </p>
  <p>
    The board keeps two signals and one effect. A <strong>signal</strong> is
    a value that knows who reads it. An <strong>effect</strong> is a
    computation that runs once, tracks every signal it read, and re-runs when
    any of them change; if it returns a function, that runs first as cleanup.
    The board's effect tracks nothing — it just starts the clock, and hands
    back the cleanup that stops it when the board unmounts:
  </p>
  <code-block code={code.controller}></code-block>

  <h2>The flow</h2>
  <p>
    Now trace one tick of the board through the file, because this loop is
    the whole model. The interval fires and writes
    <code>flights.set(...)</code> — state changes <em>only</em> because
    something wrote it. The template reads <code>flights()</code>, so the
    keyed <code>&#123;#for&#125;</code> reconciles: the departed row's DOM
    leaves, four rows shift up in place, one row enters. The
    <code>departed</code> write updates exactly one text node. Nothing else
    in the component runs — no re-render, no lifecycle to reason about.
  </p>
  <p>
    That is the direction of everything in SAIL: <strong>the template reads
    state; handlers and effects write it</strong>, and data a parent wants a
    child to have arrives as props. When you read an unfamiliar component,
    you always know where a value can change: find its writes in the
    controller, and you have the complete list.
  </p>

  <h2>Nested components</h2>
  <p>
    A compiled component is a custom element, so using one is writing its
    tag — this page's own source contains exactly what you'd expect:
  </p>
  <code-block code={code.usage}></code-block>
  <p>
    Children are the other half of nesting. The board places a named
    <code>&lt;slot&gt;</code> after its rows:
  </p>
  <code-block code={code.slotDef}></code-block>
  <p>
    …and whoever uses the board projects content into it. The footer line on
    the live board above is not the board's — it belongs to this page:
  </p>
  <code-block code={code.slotUse}></code-block>

  <row sx="mt-8 pt-4 border-t border-t-border" gap="2">
    <a variant="solid" tone="primary" href="/sail/language/state">State →</a>
    <a variant="default" href="/sail/language/templates">Templates</a>
  </row>
</column>
