<!--
  /elements — the first detailed guide: USING the components. Props, tones,
  composition, behavior — the reader's concerns, in the reader's order. The
  compile-away story is one closing note, not the curriculum.
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
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">Components</p>
  <h1>Using components</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
    The component library is part of the language: buttons, dialogs, tables,
    comboboxes and the rest are tags, written the way you write HTML. Nothing
    to install or import — put the tag on the page and it works, themed and
    accessible.
  </p>

  <h2>Write them like HTML</h2>
  <code-block code={code.useThem}></code-block>
  <p>Which is exactly this:</p>
  <card sx="p-4">
    <row gap="3" align="center">
      <avatar></avatar>
      <strong sx="grow-1">Ada Lovelace</strong>
      <badge tone="success">Active</badge>
    </row>
  </card>

  <h2>Props are the whole API</h2>
  <p>
    Three props cover most needs. <code>tone</code> picks the color from the
    design system — <code>primary</code>, <code>destructive</code>,
    <code>success</code>, <code>warning</code>, <code>info</code>,
    <code>muted</code> — and means the same thing on every component.
    <code>variant</code> picks how much of that color you get
    (<code>solid</code>, <code>soft</code>, <code>outline</code>,
    <code>ghost</code>). <code>size</code> scales controls.
  </p>
  <code-block code={code.props}></code-block>
  <row gap="2" align="center">
    <button variant="solid" tone="primary">Save</button>
    <button variant="outline" tone="destructive">Delete</button>
    <button variant="ghost">Cancel</button>
    <badge tone="warning">rolling</badge>
  </row>
  <p>
    There is no <code>color=</code> anywhere. A tone is a promise the theme
    keeps — restyle the site and every tone follows.
  </p>

  <h2>Compose by nesting</h2>
  <p>
    Components contain whatever you put inside them; named slots place
    children where the component expects them.
  </p>
  <code-block code={code.compose}></code-block>
  <collapsible>
    <strong slot="trigger">Advanced options (click me)</strong>
    <p sx="pt-2">Hidden until asked for — and this one is running.</p>
  </collapsible>

  <h2>The behavior is already wired</h2>
  <p>
    Keyboard traversal, focus management, filtering, ARIA — the parts every
    team gets subtly wrong ship inside the tag. You write the options; the
    component does the rest. Try it:
  </p>
  <code-block code={code.behavior}></code-block>
  <combobox placeholder="Pick a framework" sx="max-w-panel-max-w-sm">
    <option value="sail">Sail</option>
    <option value="svelte">Svelte</option>
    <option value="solid">Solid</option>
  </combobox>

  <h2>Find the one you need</h2>
  <p>
    <a href="/components">The component browser</a> runs every element live —
    search it, filter by category, and open any component for its variants,
    usage and props.
  </p>

  <p sx="text-sm text-muted-foreground">
    For the curious: most of these tags don't exist at runtime — the compiler
    lowers them to native markup with the behavior attached, which is why the
    library adds almost nothing to a page. A dialog ships as a real
    <code>&lt;dialog&gt;</code>. If that interests you, the
    <a href="/sail/reference">reference</a> covers the mechanics.
  </p>

  <row sx="mt-8 pt-4 border-t border-t-border" gap="2">
    <a variant="default" href="/components">← Component library</a>
    <a variant="solid" tone="primary" href="/sail/language/layout">Layout →</a>
  </row>
</column>
