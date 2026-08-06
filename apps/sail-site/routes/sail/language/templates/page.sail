<!--
  /sail/language/templates — chapter 3, round 2 per owner review: teach, don't
  sell. Lede says what the chapter IS. "Control flow", not "Branching". The
  keyed-for requirement is mentioned, not staged. Events and bind: are taught
  neutrally. bind unification question filed as a claim
  (c-2026-07-18T22-42-14-627Z-dbd7ee55); dialog replaces the div example.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  return { code }
})
</controller>

<column gap="5">
  style({ box: maxW(breakpoint-tablet).mx(auto) })

  <p>
    style({ box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })
The language</p>
  <h1>Templates</h1>
  <p>
    style({ paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) })

    The template is the markup: expressions, control flow, events, form
    bindings, and slots. It reads state; handlers write it.
  </p>

  <h2>How a template renders</h2>
  <p>
    The compiler turns the markup into real DOM the browser clones once at
    mount, and every dynamic part — a text expression, an attribute binding,
    a branch — becomes a subscription to the signals it reads. The compiler
    knows exactly which node each signal touches, so when a signal changes,
    the update goes straight to that node.
  </p>

  <h2>Reading</h2>
  <p>
    Call a signal anywhere an expression goes — the same braces work in text
    and in attributes.
  </p>
  <code-block code={code.read}></code-block>

  <h2>Control flow</h2>
  <p>
    Flow is spelled with element blocks, and they splice into the surrounding
    markup without a wrapper element. <code>&lt;if &#123;…&#125;&gt;</code> takes
    <code>&lt;else-if&#gt;</code> and <code>&lt;else&gt;</code>
    arms (brace-primary predicate):
  </p>
  <code-block code={code.ifBlock}></code-block>
  <p>
    When a value has more than two shapes, <code>&lt;match &#123;…&#125;&gt;</code>
    reads like the data it is matching on:
  </p>
  <code-block code={code.matchBlock}></code-block>
  <p>
    <code>&lt;for &#123;items() as item&#125; key=&#123;…&#125;&gt;</code> renders a list. <code>key</code> identifies
    each row, so a reorder moves DOM nodes instead of rewriting them —
    focus, selection, and in-place edits survive. Leaving it off is a
    compile error.
  </p>
  <code-block code={code.forBlock}></code-block>

  <h2>Events</h2>
  <p>
    <code>on:</code> attaches any DOM event. The handler is an expression
    over the controller's returned scope:
  </p>
  <code-block code={code.events}></code-block>
  <p>
    Modifiers go after the event name, separated by <code>|</code>:
    <code>prevent</code> calls <code>preventDefault</code> before the
    handler, <code>stop</code> stops propagation, <code>self</code> fires
    only when the event target is the element itself,
    <code>key:Escape</code> filters a keyboard event to one key, and
    <code>debounce:200</code> / <code>throttle:100</code> rate-limit in
    milliseconds. They chain.
  </p>
  <code-block code={code.modifiers}></code-block>

  <h2>Form bindings</h2>
  <p>
    <code>bind:</code> connects a form control to a signal in both
    directions: the signal's value sets the control, and user input writes
    the signal back. <code>bind:value</code> is for text-valued controls —
    inputs, textareas, selects — and syncs on the <code>input</code> event;
    <code>bind:checked</code> is for checkboxes and radios, on
    <code>change</code>. Everything else in a template flows one way.
  </p>
  <code-block code={code.twoWay}></code-block>

  <h2>Children and slots</h2>
  <p>
    Whatever you write between a component's tags is its
    <strong>children</strong>, and the component decides where children land
    by placing <code>&lt;slot&gt;</code>. Named slots give it more than one
    landing place — this is the stdlib <code>&lt;field&gt;</code>'s
    template:
  </p>
  <code-block code={code.slotDef}></code-block>
  <p>Callers fill the named places with <code>slot="…"</code>:</p>
  <code-block code={code.slotUse}></code-block>

  <p>
    Two more blocks — <code>&lt;match&#gt;</code> and
    <code>&lt;match&#gt;</code> — exist for async state and belong
    to the <a href="/sail/language/data">data story</a>.
  </p>

  <p>
    style({ box: mt(4), paint: fg(muted-foreground), text: fontSize(sm) })

    For the full surface: <a href="/sail/reference/template">reference → Template directives</a>.
  </p>

  <row justify="between" gap="2">
    style({ box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) })

    <a variant="default" href="/sail/language/state">← State</a>
    <a variant="solid" tone="primary" href="/sail/language/styling">Styling →</a>
  </row>
</column>
