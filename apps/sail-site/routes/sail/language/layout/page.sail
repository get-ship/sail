<!--
  /sail/language/layout — chapter 5. Five keywords, the axis is the name.

  Claims checked against the compiler: the element set + axis props are the
  sail-layout lint (unknown attributes refuse, with the axis list in the
  error), rows-wrap-by-default and nowrap are preprocess.rs, min-width:0 on
  every layout child is in the emitted base rules, span collapse below 720px
  is layout.rs. The quoted redirect is captured output.
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
  <h1>Layout</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
    Five keywords, and <strong>the axis is the name</strong>. There is no
    <code>flex-direction</code> to get backwards, no debate about whether a
    stack is vertical, and no CSS to write.
  </p>

  <code-block code={code.axes}></code-block>

  <h2>Why this is a keyword and not a style</h2>
  <p>
    Flexbox is four decisions before you have laid anything out: which axis,
    how the main axis distributes, how the cross axis aligns, whether it
    wraps. Every one is a place two authors diverge. So <code>sx</code> has
    none of them — write layout as a style and the compiler hands back the
    element you meant:
  </p>
  <code-block code={code.layoutSxError}></code-block>

  <h2><code>&lt;row&gt;</code> — the x axis</h2>
  <p>
    <code>gap</code> · <code>align</code> (cross axis) · <code>justify</code>
    (main axis).
  </p>
  <row gap="3" align="center" justify="between" sx="p-4 bg-muted rounded-md">
    <strong>Ada Lovelace</strong>
    <badge tone="success">Active</badge>
  </row>
  <code-block code={code.row}></code-block>
  <p>
    A row <strong>wraps by default</strong> — it is a flow, not a clamp. When
    content cannot fit, it breaks to the next line instead of overflowing.
    Most rows want exactly that; the one that does not is the shell whose
    panes must never stack — a toolbar, a rail beside a content pane. That
    row says so:
  </p>
  <code-block code={code.nowrap}></code-block>

  <h2><code>&lt;column&gt;</code> — the y axis</h2>
  <p><code>gap</code> · <code>align</code>.</p>
  <code-block code={code.column}></code-block>

  <h2><code>&lt;grid&gt;</code> — two dimensions</h2>
  <p>
    <code>cols</code> (and <code>rows</code>) fix a track count;
    <code>min</code> makes a responsive track that fills as many columns of
    at least that width as will fit. You do not write a media query.
  </p>
  <grid cols="3" gap="3">
    <card sx="p-4"><strong>One</strong></card>
    <card sx="p-4"><strong>Two</strong></card>
    <card sx="p-4"><strong>Three</strong></card>
  </grid>
  <code-block code={code.grid}></code-block>
  <p>
    A child spans tracks with <code>data-col-span</code> /
    <code>data-row-span</code> — and below 720px every span collapses to a
    single track on its own, so the wide card never forces a phone to scroll
    sideways.
  </p>
  <code-block code={code.spans}></code-block>

  <h2><code>&lt;stack&gt;</code> — the z axis</h2>
  <p>
    Children occupy the same cell, one over another: a badge on an avatar, an
    overlay on an image. It is the third axis, and it has a name for the same
    reason the other two do.
  </p>

  <h2><code>&lt;center&gt;</code></h2>
  <p>
    Centres its child on both axes. The single most-Googled question in CSS
    is a keyword here.
  </p>

  <h2>What children get</h2>
  <p>
    Every child of a layout element is born with <code>min-width: 0</code>.
    Flexbox's default — an item refuses to shrink below its content — is the
    reason truncation "doesn't work" and long words blow layouts open in
    every flex codebase. Here the default is corrected at the language level:
    <code>truncate</code> works the first time.
  </p>
  <p>
    That is the contract in one sentence: <strong>the parent decides how its
    children sit; a child decides only how it fills the space it was
    given</strong> — <code>grow-1</code>, <code>basis-*</code>,
    <code>min-w-0</code> are <code>sx</code> because they describe the
    element, not its siblings.
  </p>

  <h2>That is the whole vocabulary</h2>
  <p>
    Rows of columns of rows. Every layout you have ever built is those
    three, nested — and because they are keywords, two agents nest them the
    same way.
  </p>
  <code-block code={code.nesting}></code-block>

  <row sx="mt-8 pt-4 border-t border-t-border" justify="between" gap="2">
    <a variant="default" href="/sail/language/styling">← Styling</a>
    <a variant="solid" tone="primary" href="/sail/language/motion">Motion →</a>
  </row>
</column>
