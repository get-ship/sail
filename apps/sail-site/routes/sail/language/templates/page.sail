<!--
  /sail/language/templates — chapter 3. The read surface of the language:
  expressions, branches, lists (and the keyed-for refusal, quoted verbatim),
  events, two-way form bindings, slots. Every claim here is checked against
  the compiler: the block set is tokenize.rs, the modifier list is attrs.rs,
  the bind: desugaring is runtime bind.ts, the diagnostic is real output.
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
  <h1>Templates</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
    A template reads state; handlers write it. The last chapter was the state
    side of that sentence. This one is the markup: expressions, branches,
    lists, events, form bindings, and slots.
  </p>

  <h2>How a template renders</h2>
  <p>
    The compiler turns the markup into real DOM the browser clones once at
    mount. Every dynamic part — a text expression, an attribute binding, a
    branch — becomes a subscription to the signals it reads. When a signal
    changes, only its subscribers run: the one text node updates, the one
    branch swaps. There is no re-render and no virtual DOM, because nothing
    needs diffing — the compiler already knows exactly which node each
    signal touches.
  </p>

  <h2>Reading</h2>
  <p>
    Call a signal anywhere an expression goes. That is the entire binding
    syntax — the same braces work in text and in attributes.
  </p>
  <code-block code={code.read}></code-block>

  <h2>Branching</h2>
  <p>
    Flow is spelled with blocks. Blocks splice into the surrounding markup
    without a wrapper element, so control flow never shows up in your DOM.
    <code>&#123;#if&#125;</code> takes <code>&#123;:else if&#125;</code> and
    <code>&#123;:else&#125;</code> arms:
  </p>
  <code-block code={code.ifBlock}></code-block>
  <p>
    When a value has more than two shapes, <code>&#123;#match&#125;</code>
    reads like the data it is matching on:
  </p>
  <code-block code={code.matchBlock}></code-block>

  <h2>Lists</h2>
  <p>
    <code>&#123;#for&#125;</code> renders a list, and <code>key</code> is
    required — it tells the reconciler which DOM row is which, so a reorder
    moves nodes instead of rewriting them, and the input you were typing in
    keeps its cursor. Leaving the key off is the classic lost-focus bug, in
    every framework. Here it does not compile:
  </p>
  <code-block code={code.forBlock}></code-block>
  <p>Compiling the same block without <code>key</code> stops with:</p>
  <code-block code={code.forKeyError}></code-block>

  <h2>Events</h2>
  <p>
    <code>on:</code> attaches any DOM event, and the handler is where state
    gets written. The handler must be a name the controller returned — the
    template cannot reach anything else.
  </p>
  <code-block code={code.events}></code-block>
  <p>
    Modifiers declare intent instead of boilerplate:
    <code>|prevent</code>, <code>|stop</code>, <code>|self</code>,
    <code>|trusted</code>, <code>|key:Escape</code>,
    <code>|debounce:200</code>, <code>|throttle:100</code>. They chain, and
    you never write <code>e.preventDefault()</code> by hand.
  </p>
  <code-block code={code.modifiers}></code-block>

  <h2>Two-way, exactly twice</h2>
  <p>
    Form controls are the one place a read and a write happen on the same
    attribute. <code>bind:value</code> keeps a string signal and an input in
    sync — it is precisely an effect that writes <code>.value</code> plus an
    <code>input</code> listener that writes the signal back.
    <code>bind:checked</code> does the same for checkboxes and radios, on the
    <code>change</code> event. Everything else in a template flows one way.
  </p>
  <code-block code={code.twoWay}></code-block>

  <h2>Slots</h2>
  <p>
    Components compose by projection: children you write inside a component's
    tag land on its <code>&lt;slot&gt;</code>. Named slots give a component
    more than one landing place — this is the stdlib
    <code>&lt;field&gt;</code>'s template:
  </p>
  <code-block code={code.slotDef}></code-block>
  <p>Which callers fill by name:</p>
  <code-block code={code.slotUse}></code-block>

  <p>
    Two more blocks — <code>&#123;#while-loading&#125;</code> and
    <code>&#123;#while-error&#125;</code> — exist for async state and belong
    to the <a href="/sail/data">data story</a>.
  </p>

  <row sx="mt-8 pt-4 border-t border-t-border" gap="2">
    <a variant="default" href="/sail/language/state">← State</a>
    <a variant="solid" tone="primary" href="/sail/styling">Styling →</a>
  </row>
</column>
