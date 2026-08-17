<!--
  /elements — the component-library guide: what ships, the shared prop
  channels, composition, wired-in behavior, icons, prose. Every category and
  prop claim here is checked against the element registry and the stdlib
  sources; the snippets ARE the demo code.
-->

<scope>
  type CatLabelProps = {
    label: string
  }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from '../../../../src/compositions/sail-elements-snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  return { code }
})
</controller>

<partial cat-label props: CatLabelProps>
  <badge variant="outline" tone="muted">{label}</badge>
</partial>

<article>
  <column gap="5">
    <p>Guide</p>
    <h1>Using components</h1>
    <p>
      The component library is part of the language: buttons, dialogs, tables,
      comboboxes and the rest are tags, written the way you write HTML. Nothing
      to install or import — put the tag on the page and it works, themed and
      accessible.
    </p>

    <h2>Write them like HTML</h2>
    <code-block code={code.useThem}></code-block>
    <p>Which is exactly this:</p>
    <card>
      <row gap="3" align="center">
        <avatar></avatar>
        <strong>Ada Lovelace</strong>
        <badge tone="success">Active</badge>
      </row>
    </card>

    <h2>What ships</h2>
    <p>
      The library is seventy elements: forty-two components the compiler itself
      ships as <code>.sail</code> sources — the same language you write — plus
      twenty-eight bare HTML tags the framework restyles, so a plain
      <code>&lt;button&gt;</code> or <code>&lt;table&gt;</code> is already the
      design system’s. The catalog groups them like this:
    </p>
    <column gap="4">
      <column gap="2">
        <cat-label label="Inputs"></cat-label>
        <row gap="1" align="center">
          <badge size="sm" variant="soft" tone="muted">button</badge>
          <badge size="sm" variant="soft" tone="muted">input</badge>
          <badge size="sm" variant="soft" tone="muted">textarea</badge>
          <badge size="sm" variant="soft" tone="muted">select</badge>
          <badge size="sm" variant="soft" tone="muted">combobox</badge>
          <badge size="sm" variant="soft" tone="muted">checkbox</badge>
          <badge size="sm" variant="soft" tone="muted">radio-group</badge>
          <badge size="sm" variant="soft" tone="muted">switch</badge>
          <badge size="sm" variant="soft" tone="muted">slider</badge>
          <badge size="sm" variant="soft" tone="muted">toggle-group</badge>
          <badge size="sm" variant="soft" tone="muted">field</badge>
          <badge size="sm" variant="soft" tone="muted">label</badge>
          <badge size="sm" variant="soft" tone="muted">fieldset</badge>
          <badge size="sm" variant="soft" tone="muted">theme-picker</badge>
        </row>
      </column>
      <column gap="2">
        <cat-label label="Display"></cat-label>
        <row gap="1" align="center">
          <badge size="sm" variant="soft" tone="muted">badge</badge>
          <badge size="sm" variant="soft" tone="muted">chip</badge>
          <badge size="sm" variant="soft" tone="muted">card</badge>
          <badge size="sm" variant="soft" tone="muted">avatar</badge>
          <badge size="sm" variant="soft" tone="muted">icon</badge>
          <badge size="sm" variant="soft" tone="muted">separator</badge>
          <badge size="sm" variant="soft" tone="muted">kbd</badge>
          <badge size="sm" variant="soft" tone="muted">accordion</badge>
          <badge size="sm" variant="soft" tone="muted">prose</badge>
        </row>
      </column>
      <column gap="2">
        <cat-label label="Feedback"></cat-label>
        <row gap="1" align="center">
          <badge size="sm" variant="soft" tone="muted">alert</badge>
          <badge size="sm" variant="soft" tone="muted">progress</badge>
          <badge size="sm" variant="soft" tone="muted">meter</badge>
          <badge size="sm" variant="soft" tone="muted">spinner</badge>
          <badge size="sm" variant="soft" tone="muted">skeleton</badge>
          <badge size="sm" variant="soft" tone="muted">empty-state</badge>
          <badge size="sm" variant="soft" tone="muted">error-state</badge>
          <badge size="sm" variant="soft" tone="muted">error-boundary</badge>
          <badge size="sm" variant="soft" tone="muted">toast</badge>
        </row>
      </column>
      <column gap="2">
        <cat-label label="Overlay"></cat-label>
        <row gap="1" align="center">
          <badge size="sm" variant="soft" tone="muted">tooltip</badge>
          <badge size="sm" variant="soft" tone="muted">hover-card</badge>
          <badge size="sm" variant="soft" tone="muted">popover</badge>
          <badge size="sm" variant="soft" tone="muted">dropdown</badge>
          <badge size="sm" variant="soft" tone="muted">context-menu</badge>
          <badge size="sm" variant="soft" tone="muted">dialog</badge>
        </row>
      </column>
      <column gap="2">
        <cat-label label="Navigation"></cat-label>
        <row gap="1" align="center">
          <badge size="sm" variant="soft" tone="muted">tabs</badge>
          <badge size="sm" variant="soft" tone="muted">breadcrumb</badge>
          <badge size="sm" variant="soft" tone="muted">pagination</badge>
          <badge size="sm" variant="soft" tone="muted">navbar</badge>
        </row>
      </column>
      <column gap="2">
        <cat-label label="Data"></cat-label>
        <row gap="1" align="center">
          <badge size="sm" variant="soft" tone="muted">table</badge>
        </row>
      </column>
      <column gap="2">
        <cat-label label="Layout"></cat-label>
        <row gap="1" align="center">
          <badge size="sm" variant="soft" tone="muted">row</badge>
          <badge size="sm" variant="soft" tone="muted">column</badge>
          <badge size="sm" variant="soft" tone="muted">grid</badge>
          <badge size="sm" variant="soft" tone="muted">center</badge>
          <badge size="sm" variant="soft" tone="muted">scroll-area</badge>
        </row>
      </column>
    </column>
    <p>
      A few tags only make sense inside a parent — <code>&lt;tab&gt;</code>
      inside <code>&lt;tabs&gt;</code>, <code>&lt;menu-item&gt;</code> inside
      <code>&lt;dropdown&gt;</code>, <code>&lt;page&gt;</code> inside
      <code>&lt;pagination&gt;</code> — so they appear under their parent in
      the catalog rather than on their own.
    </p>

    <h2>Props are the whole API</h2>
    <p>
      Three props cover most needs, and they are channels — the same prop
      means the same thing on every element that takes it.
    </p>
    <p>
      <code>tone</code> picks the color from the design system. The full set is
      <code>default</code>, <code>primary</code>, <code>info</code>,
      <code>success</code>, <code>warning</code>, <code>destructive</code>,
      <code>accent</code> and <code>muted</code>; a few elements accept only the
      subset that makes sense for them (an alert has no <code>primary</code>),
      but <code>tone="destructive"</code> is the same red promise on a button,
      a badge, a switch or a slider.
    </p>
    <p>
      <code>variant</code> picks how the element draws itself. On tone-colored
      elements that means fill strength — <code>solid</code>,
      <code>soft</code>, <code>outline</code>, <code>ghost</code> — while a few
      elements have looks of their own (tabs come in <code>line</code>,
      <code>pill</code> and <code>enclosed</code>). <code>size</code> scales
      controls: <code>sm</code>, <code>md</code>, <code>lg</code>, with
      <code>xs</code> and <code>xl</code> on the small stuff like icons and
      avatars.
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
      children where the component expects them —
      <code>slot="trigger"</code> on an accordion,
      <code>slot="brand"</code> and <code>slot="actions"</code> on a navbar,
      <code>slot="fallback"</code> on an error boundary.
    </p>
    <code-block code={code.compose}></code-block>
    <accordion>
      <strong slot="trigger">Advanced options (click me)</strong>
      <p>Hidden until asked for — and this one is running.</p>
    </accordion>
    <p>
      Bigger components are small vocabularies of tags rather than
      configuration objects. A tab is its title plus the panel it owns:
    </p>
    <code-block code={code.tabs}></code-block>
    <tabs>
      <tab title="Overview">The overview panel.</tab>
      <tab title="Activity">The activity panel.</tab>
    </tabs>

    <h2>The behavior is already wired</h2>
    <p>
      Keyboard traversal, focus management, filtering, ARIA — the parts every
      team gets subtly wrong ship inside the tag. The tabs above answer arrow
      keys, Home and End with a roving tabindex, and stamp
      semantic <code>selected</code> / tab anatomy themselves. A
      dropdown closes on Escape and supports type-ahead. A switch is a real
      <code>&lt;switch&gt;</code> (L1 anatomy stamps role=switch), so Space, Tab
      and form submission are the platform’s own. You write the options; the
      component does the rest. Try it:
    </p>
    <code-block code={code.behavior}></code-block>
    <combobox placeholder="Pick a fruit">
      <option value="apple">Apple</option>
      <option value="mango">Mango</option>
      <option value="plum">Plum</option>
    </combobox>

    <h2>Icons</h2>
    <p>
      The icon system is one element and one attribute:
      <code>&lt;icon name="…"&gt;</code>, where the name is any of the 1,952
      glyphs in the bundled <a href="https://lucide.dev">Lucide</a> set,
      kebab-case, exactly as Lucide names them. The dev server serves each
      glyph as a tiny SVG; the element fetches it once and caches it.
    </p>
    <code-block code={code.icons}></code-block>
    <row gap="3" align="center">
      <icon name="search"></icon>
      <icon name="git"></icon>
      <icon name="compass"></icon>
      <icon name="zap"></icon>
      <icon name="rocket" size="lg"></icon>
      <icon name="alarm-clock" size="xl" label="Alarm"></icon>
      <icon name="refresh" spin></icon>
    </row>
    <p>
      An icon has no color prop — it draws in <code>currentColor</code>, the
      text color of wherever it sits, which is why it follows tone for free:
    </p>
    <code-block code={code.iconButton}></code-block>
    <row gap="2" align="center">
      <button variant="solid" tone="primary">
        <icon name="rocket"></icon> Launch
      </button>
      <button variant="outline" tone="destructive">
        <icon name="trash"></icon> Delete
      </button>
    </row>
    <p>
      Accessibility is the <code>label</code> field — with it the icon is
      accessible name via semantic prop <code>label</code>; without it the
      icon is marked <code>hidden</code> — decoration, invisible to a
      screen reader. <code>size</code> runs <code>xs</code> to
      <code>xl</code>, <code>spin</code> rotates (and respects reduced-motion),
      and <code>src=</code> fetches any SVG URL when you need a glyph that
      isn’t in the set.
    </p>

    <h2>Prose</h2>
    <p>
      Templates you author don’t need typography classes — bare
      <code>h1</code>, <code>p</code> and <code>code</code> already take the
      design system’s type scale. <code>&lt;prose&gt;</code> is for content you
      <em>render</em>: markdown output, a doc blob, anything that arrives as
      flowing text at runtime. Wrap it and the whole subtree gets reading
      typography — heading rhythm, list indentation, quote bars, inline code
      chips, table borders:
    </p>
    <code-block code={code.prose}></code-block>
    <card>
      <prose>
        <h3>Release notes</h3>
        <p>The build went <strong>green</strong> at noon — see
           <code>deploy.log</code> for the timings.</p>
        <blockquote>Ship early. Ship boring.</blockquote>
        <ul>
          <li>tokens are the only styling currency</li>
          <li>elements carry their own behavior</li>
        </ul>
      </prose>
    </card>
    <p>
      It takes no props. This site used to carry 515 lines of hand-written
      reading-column CSS; the framework stylesheet and
      <code>&lt;prose&gt;</code> replaced all of it.
    </p>

    <h2>Find the one you need</h2>
    <p>
      <a href="/components">The component browser</a> runs every element live —
      search it, filter by category, and open any component for its variants,
      usage and props. The props tables there are generated from the same
      declarations the compiler checks, so they cannot drift from the source.
    </p>

    <p>
      For the curious: most of these tags don’t exist at runtime — the compiler
      lowers them to native markup with the behavior attached, which is why the
      library adds almost nothing to a page. A dialog ships as a real
      <code>&lt;dialog&gt;</code>. If that interests you, the
      <a href="/docs/reference">reference</a> covers the mechanics.
    </p>

    <separator />
    <row gap="2">
      <a variant="default" href="/docs/concepts/theming">← Theming</a>
      <a variant="solid" tone="primary" href="/docs/guide/building-a-component">Building a component →</a>
    </row>
  </column>
</article>
