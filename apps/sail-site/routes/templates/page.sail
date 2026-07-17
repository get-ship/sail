<!--
  /templates — control flow + bindings + events, which were three pages and are
  one idea: how the markup reads and writes state.
-->

<controller>
import { controller, type ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ViewScope<Scope>) => {
  return { code }
})
</controller>

<column sx="max-w-breakpoint-tablet mx-auto" gap="5">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">Building</p>
  <h1>Templates</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
    A template <strong>reads</strong> state; handlers <strong>write</strong> it.
    That separation is the whole mental model — and it is why you never have to
    wonder where a value changed.
  </p>

  <h2>One file, three parts</h2>
  <p>
    A <code>.sail</code> file has up to three sections, and each answers one
    question. <code>&lt;component&gt;</code> declares what comes <em>in</em> —
    the props, with types and defaults; using a prop off the declaration is a
    compile error. <code>&lt;controller&gt;</code> owns state and logic — its
    <code>scope()</code> returns exactly the names the template may bind, and
    nothing else is reachable. Everything outside those blocks is the
    <strong>template</strong>: markup that reads state and never computes it.
  </p>
  <code-block code={code.anatomy}></code-block>

  <h2>How a template renders</h2>
  <p>
    Once, into real DOM — then never again. The compiler turns the markup
    into a template the browser clones at mount, and every dynamic part —
    <code>{'{count()}'}</code>, an attribute binding, an <code>{'{#if}'}</code>
    branch — becomes a subscription to the signals it reads. When a signal
    changes, only its subscribers run: the text node updates, the branch
    swaps. There is no re-render, no virtual DOM to diff, and no component
    function that runs twice. What you wrote is what is in the page.
  </p>


  <h2>Reading</h2>
  <p>Call a signal anywhere an expression goes. That is the entire binding syntax.</p>
  <code-block code={code.read}></code-block>
  <p>
    A text binding is a compiled effect: when the signal changes, the compiler
    updates <em>the one text node that read it</em>. Nothing else in the tree
    re-runs. There is no virtual DOM because there is nothing to diff.
  </p>

  <h2>Writing</h2>
  <p>Handlers write. <code>on:</code> takes any DOM event.</p>
  <code-block code={code.write}></code-block>
  <p>
    Modifiers declare intent instead of boilerplate — <code>|prevent</code>,
    <code>|stop</code>, <code>|self</code>, <code>|key:Escape</code>,
    <code>|debounce:200</code>. You never write
    <code>e.preventDefault()</code> again, and you never forget it either.
  </p>
  <code-block code={code.modifiers}></code-block>

  <h2>Both at once</h2>
  <p>
    <code>bind:</code> is the one place a read and a write happen together — and
    it is exactly sugar for the read plus an <code>on:input</code> write. Two
    directives, because there are two kinds of control.
  </p>
  <code-block code={code.twoWay}></code-block>

  <h2>Shape</h2>
  <p>
    Three blocks. Each is a <em>partial</em> — it splices into the surrounding
    markup with no wrapper element, so control flow never shows up in your DOM.
  </p>

  <h3><code>&#123;#if&#125;</code></h3>
  <code-block code={code.ifBlock}></code-block>

  <h3><code>&#123;#for&#125;</code></h3>
  <p>
    <code>key</code> is <strong>required</strong>, and that is not pedantry:
    without it a reorder is a re-render, and the input you were typing in loses
    its cursor. Every framework has this bug. Here it will not compile.
  </p>
  <code-block code={code.forBlock}></code-block>

  <h3><code>&#123;#match&#125;</code></h3>
  <p>A discriminated match that reads like the data it is matching on.</p>
  <code-block code={code.matchBlock}></code-block>

  <row sx="mt-8 pt-4 border-t border-t-border" gap="2">
    <a variant="default" href="/reactivity">← State</a>
    <a variant="solid" tone="primary" href="/data">Data →</a>
  </row>
</column>
