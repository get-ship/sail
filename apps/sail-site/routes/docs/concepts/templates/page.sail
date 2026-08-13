<!--
  /docs/concepts/templates — expressions, control flow, events, bind:, slots,
  in-file partials. Teach, don’t sell.
-->

<scope>
  style page = { box: size({ max-w: breakpoint-tablet }).space({ mx: auto }) }
  style eyebrow = { box: space({ mb: 2 }), paint: ink({ color: accent }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { paint: ink({ color: muted-foreground }), text: type({ size: lg, leading: relaxed }) }
  style footNote = { box: space({ mt: 4 }), paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style footNav = { box: space({ pad-t: 4, mt: 8 }), paint: border({ color: border, width: 1, edge: t }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from '../../../../src/compositions/sail-language-templates-snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  return { code }
})
</controller>

<column gap="5" style(page)>
  <p style(eyebrow)>The language</p>
  <h1>Templates</h1>
  <p style(lead)>
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
    <code>&lt;else-if&gt;</code> and <code>&lt;else&gt;</code>
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

  <h2>In-file partials</h2>
  <p>
    When the same markup repeats in <em>one</em> file, declare an in-file
    <code>&lt;partial&gt;</code> — a same-file functional component. Params
    are a plain type in <code>&lt;scope&gt;</code>; the body reads them as bare
    names (<code>&#123;title&#125;</code>), and you invoke it as
    <code>&lt;kebab-name …&gt;</code> in this file only.
  </p>
  <code-block code={code.partial}></code-block>
  <p>
    Partials expand at preprocess: the runtime never sees them, and the
    lowered tree is the same as hand-written markup. Rules that keep them
    small and honest: top-level declaration only; no recursion; no
    tag-position <code>style&#40;…&#41;</code> on the invocation (paint inside
    the body, or pass a Style value as a param). Cross-file reuse is still a
    component file — <code>src/&lt;tag&gt;/&lt;tag&gt;.sail</code>.
  </p>
  <p>
    This is not Style composition. Named Style pieces composed with
    <code>style&#40;base, emphasis&#41;</code> or <code>match:</code> arms are
    paint data on a host node; <code>&lt;partial&gt;</code> is template
    structure. The
    <a href="/docs/concepts/styling">styling chapter</a> covers Style
    composition.
  </p>

  <h2>Events and callbacks</h2>
  <p>
    Props flow down. Callbacks report up. <code>bind:</code> syncs a value.
    That is the entire communication model — there is no event bus.
  </p>
  <p>
    <code>on:</code> attaches a real DOM event (click, input, submit, …).
    The handler is an expression over the controller’s returned scope:
  </p>
  <code-block code={code.events}></code-block>
  <p>
    Component moments use typed callback props — kebab
    <code>on-*</code> attrs such as <code>on-select=&#123;fn&#125;</code> or
    <code>on-page-change=&#123;go&#125;</code>. The payload is typed on the
    owning element; hover shows it. Form mutation moments are
    <code>on-success</code> / <code>on-failure</code> on
    <code>&lt;form mutation=…&gt;</code> (do not react to moments via the
    MutationState signal — it persists and re-fires).
  </p>
  <p>
    Modifiers go after a DOM event name, separated by <code>|</code>:
    <code>prevent</code> calls <code>preventDefault</code> before the
    handler, <code>stop</code> stops propagation, <code>self</code> fires
    only when the event target is the element itself,
    <code>key:Escape</code> filters a keyboard event to one key, and
    <code>debounce:200</code> / <code>throttle:100</code> rate-limit in
    milliseconds. They chain. Modifiers do not apply to callback props —
    use the runtime debounce helper when needed.
  </p>
  <code-block code={code.modifiers}></code-block>

  <h2>Form bindings</h2>
  <p>
    <code>bind:</code> connects a form control to a signal in both
    directions: the signal’s value sets the control, and user input writes
    the signal back. <code>bind:value</code> is for text-valued controls —
    inputs, textareas, selects — and syncs on the <code>input</code> event;
    <code>bind:checked</code> is for checkboxes and radios, on
    <code>change</code>. Everything else in a template flows one way.
  </p>
  <code-block code={code.twoWay}></code-block>

  <h2>Children and slots</h2>
  <p>
    Whatever you write between a component’s tags is its
    <strong>children</strong>, and the component decides where children land
    by placing <code>&lt;slot&gt;</code>. Named slots give it more than one
    landing place — this is the stdlib <code>&lt;field&gt;</code>’s
    template:
  </p>
  <code-block code={code.slotDef}></code-block>
  <p>Callers fill the named places with <code>slot="…"</code>:</p>
  <code-block code={code.slotUse}></code-block>

  <p>
    The same <code>&lt;match&gt;</code> block also covers async
    <code>query()</code> state with <code>loading</code> /
    <code>error</code> / <code>ok</code> arms — that shape belongs to the
    <a href="/docs/concepts/data">data story</a>.
  </p>

  <p style(footNote)>
    For the full surface: <a href="/docs/reference/template">reference → Template directives</a>.
  </p>

  <row justify="between" gap="2" style(footNav)>
    <a variant="default" href="/docs/concepts/state">← State</a>
    <a variant="solid" tone="primary" href="/docs/concepts/styling">Styling →</a>
  </row>
</column>
