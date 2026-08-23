<!--
  /docs/guide/building-a-component — walkthrough of bill-split.sail in six
  steps. Snippets are slices of the real file; the finished tag runs below.
-->

<controller>
import { code } from '../../../../src/compositions/sail-guide-building-a-component-snippets'

export default controller(() => {
  return { code }
})
</controller>

<article>
  <column gap="5">
    <p>Guide</p>
    <h1>Building a component</h1>
    <p>
      We are going to write one component from an empty file: a bill splitter
      that divides a total between however many people are at the table. Six
      steps, each one a small change to the step before it. At the end it runs
      on this page.
    </p>

    <h2>Step 1 — a file with markup in it</h2>
    <p>
      A component is a <code>.sail</code> file. The convention is a directory
      named for the tag, holding a file of the same name — so
      <code>src/bill-split/bill-split.sail</code> becomes
      <code>&lt;bill-split&gt;</code>. Nothing registers it; the name of the
      file is the name of the tag.
    </p>
    <p>
      The smallest useful file is a template and nothing else. Layout is
      elements — <code>&lt;row&gt;</code> and <code>&lt;column&gt;</code> with
      <code>gap</code>, <code>align</code> and <code>justify</code> — and
      <code>style&#40;&#123; pad(4).fg(muted-foreground) &#125;&#41;</code> carries the finish.
    </p>
    <code-block code={code.markup} lang="sail"></code-block>
    <p>
      That is a working component. It is also a poster: everything in it is
      fixed.
    </p>

    <h2>Step 2 — the fixed parts become props</h2>
    <p>
      A <strong>prop</strong> is a value the caller passes in. Props are
      declared in a <code>&lt;scope&gt;</code> block, one per line, each with a
      default. The default is also the type — <code>total = 96</code> declares
      a number.
    </p>
    <p>
      In the template a prop is <em>called</em>, not read:
      <code>&#123;label()&#125;</code>. Props are signals, and calling one is how
      you read a signal’s current value. That is the same everywhere in the
      language, so there is only one rule to remember.
    </p>
    <code-block code={code.scope} lang="sail"></code-block>
    <p>From a page, the props are attributes:</p>
    <code-block code={code.usageProps} lang="sail"></code-block>

    <h2>Step 3 — state, in the controller</h2>
    <p>
      Props come from outside. Anything the component owns itself lives in a
      <code>&lt;controller&gt;</code> block, which is ordinary TypeScript.
      Whatever the controller returns becomes the template’s vocabulary —
      values and event handlers alike.
    </p>
    <p>
      Two primitives do the work here. <code>signal(2)</code> holds a value
      that changes; you read it by calling it and change it with
      <code>.set()</code>. <code>derived(…)</code> computes a value from other
      signals and recomputes itself when they move — nobody has to remember to
      update the share when the headcount changes.
    </p>
    <p>
      <code>ctx</code> is how the controller reads its own props, so the
      per-person share is <code>ctx.total&#40;&#41; / people()</code>. Formatting is
      ordinary code and belongs here rather than in the markup.
    </p>
    <code-block code={code.controller} lang="sail"></code-block>
    <p>
      The template spends what the controller returned.
      <code>on:click</code> attaches a DOM event listener; the handler is just
      the function.
    </p>
    <code-block code={code.counterRow} lang="sail"></code-block>

    <h2>Step 4 — styling, without leaving the tag</h2>
    <p>
      There is no stylesheet and no <code>&lt;style&gt;</code> block. Attach
      <strong>Style</strong> as a block statement on the host:
      <code>style&#40;&#123; pad(4).fg(muted-foreground).type(&#123; size: sm &#125;) &#125;&#41;</code>.
      Setters chain — <code>pad</code>, <code>fg</code>, <code>type</code>,
      <code>bg</code>, <code>size</code>, <code>border</code>.
      Values are design tokens: <code>type(&#123; size: sm &#125;)</code>,
      <code>fg(muted-foreground)</code>, <code>pad(4)</code>. Prefer
      <code>style&#32;name = &#123; … &#125;</code> in scope when the same paint
      repeats, then attach <code>style&#40;name&#41;</code>.
    </p>
    <p>
      The values are a closed set. <code>fg(muted-foreground)</code> compiles;
      a hex code or a pixel value does not, and the error names what you may
      use instead. That is what keeps a component readable under all nine
      themes: it never names a color, only a role. Utility-string hosts
      (<code>sx="…"</code>) are not the product — they refuse.
    </p>
    <p>
      Two setters above are worth naming.
      <code>type(&#123; family: mono &#125;)</code> with <code>type(&#123; nums: tabular &#125;)</code> gives
      digits a fixed width, so the total does not jitter as it recalculates.
      <code>size(&#123; max-w: panel-max-w-sm &#125;)</code> stops the card from stretching to the
      width of the page.
    </p>

    <h2>Step 5 — a slot, so callers can add their own content</h2>
    <p>
      A <strong>slot</strong> is a hole in the template that the caller fills.
      Give it a name and the caller targets it with a matching
      <code>slot=</code> attribute.
    </p>
    <code-block code={code.slotDef} lang="sail"></code-block>
    <p>
      A component with a slot stays useful in situations its author did not
      anticipate, without growing a prop for each one.
    </p>

    <h2>Step 6 — use it</h2>
    <p>
      Put the tag on a page, pass the props as attributes, and fill the slot.
      This is the whole integration step; there is no import and no
      registration call.
    </p>
    <code-block code={code.usage} lang="sail"></code-block>
    <p>And that is the component, running here:</p>
    <bill-split label="Dinner" total="96">
      <small slot="note">
        Tip included. Arguments not.
      </small>
    </bill-split>
    <p>
      Press the buttons. The share is a <code>derived</code> value, so it
      follows the headcount without anything being wired between them.
    </p>

    <h2>Where to go from here</h2>
    <p>
      The component above stays the same shape as it grows. Longer lists want
      <code>&lt;for&gt;</code>, covered in
      <a href="/docs/concepts/templates">Templates</a>; state shared between
      components is in <a href="/docs/concepts/state">State</a>; movement is in
      <a href="/docs/concepts/animation">Animation</a>.
    </p>

    <separator />
    <row gap="2">
      <a variant="default" href="/docs/guide/using-components">← Using components</a>
      <a variant="solid" tone="primary" href="/docs/guide/forms">Forms →</a>
    </row>
  </column>
</article>
