<!--
  /elements — the component-library guide: what ships, the shared prop
  channels, composition, wired-in behavior, icons, prose. Every category and
  prop claim here is checked against the element registry and the stdlib
  sources; the snippets ARE the demo code.
-->

<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  return { code }
})
</controller>

<column sx={style.maxW(breakpoint-tablet).mx(auto)} gap="5">
  <p sx={style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)}>Guide</p>
  <h1>Using components</h1>
  <p sx={style.fontSize(lg).fg(muted-foreground).leading(relaxed)}>
    The component library is part of the language: buttons, dialogs, tables,
    comboboxes and the rest are tags, written the way you write HTML. Nothing
    to install or import — put the tag on the page and it works, themed and
    accessible.
  </p>

  <h2>Write them like HTML</h2>
  <code-block code={code.useThem}></code-block>
  <p>Which is exactly this:</p>
  <card sx={style.pad(4)}>
    <row gap="3" align="center">
      <avatar></avatar>
      <strong sx={style.grow(1)}>Ada Lovelace</strong>
      <badge tone="success">Active</badge>
    </row>
  </card>

  <h2>What ships</h2>
  <p>
    The library is seventy elements: forty-two components the compiler itself
    ships as <code>.sail</code> sources — the same language you write — plus
    twenty-eight bare HTML tags the framework restyles, so a plain
    <code>&lt;button&gt;</code> or <code>&lt;table&gt;</code> is already the
    design system's. The catalog groups them like this:
  </p>
  <column gap="2">
    <row gap="3" align="start" nowrap>
      <badge variant="outline" sx={style.shrink(0)}>Inputs</badge>
      <p sx={style.fontSize(sm).fg(muted-foreground)}>button, input, textarea, select,
      combobox, checkbox, radio-group, switch, slider, toggle-group, field,
      label, fieldset, dropzone, theme-picker</p>
    </row>
    <row gap="3" align="start" nowrap>
      <badge variant="outline" sx={style.shrink(0)}>Display</badge>
      <p sx={style.fontSize(sm).fg(muted-foreground)}>badge, chip, status-dot, card,
      avatar, icon, separator, hr, details, collapsible, prose</p>
    </row>
    <row gap="3" align="start" nowrap>
      <badge variant="outline" sx={style.shrink(0)}>Feedback</badge>
      <p sx={style.fontSize(sm).fg(muted-foreground)}>alert, progress, meter, spinner,
      skeleton, empty-state, error-state, error-boundary, toast</p>
    </row>
    <row gap="3" align="start" nowrap>
      <badge variant="outline" sx={style.shrink(0)}>Overlay</badge>
      <p sx={style.fontSize(sm).fg(muted-foreground)}>tooltip, hover-card, popover,
      dropdown, context-menu, dialog</p>
    </row>
    <row gap="3" align="start" nowrap>
      <badge variant="outline" sx={style.shrink(0)}>Navigation</badge>
      <p sx={style.fontSize(sm).fg(muted-foreground)}>tabs, breadcrumb, pagination,
      navbar</p>
    </row>
    <row gap="3" align="start" nowrap>
      <badge variant="outline" sx={style.shrink(0)}>Data</badge>
      <p sx={style.fontSize(sm).fg(muted-foreground)}>table</p>
    </row>
    <row gap="3" align="start" nowrap>
      <badge variant="outline" sx={style.shrink(0)}>Layout</badge>
      <p sx={style.fontSize(sm).fg(muted-foreground)}>row, column, grid, center,
      scroll-area</p>
    </row>
  </column>
  <p sx={style.fontSize(sm).fg(muted-foreground)}>
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
    <code>slot="trigger"</code> on a collapsible,
    <code>slot="brand"</code> and <code>slot="actions"</code> on a navbar,
    <code>slot="fallback"</code> on an error boundary.
  </p>
  <code-block code={code.compose}></code-block>
  <collapsible>
    <strong slot="trigger">Advanced options (click me)</strong>
    <p sx={style.padT(2)}>Hidden until asked for — and this one is running.</p>
  </collapsible>
  <p>
    Bigger components are small vocabularies of tags rather than
    configuration objects. A tab strip is written as the structure it is:
  </p>
  <code-block code={code.tabs}></code-block>
  <tabs active="overview">
    <tab-list>
      <tab name="overview">Overview</tab>
      <tab name="activity">Activity</tab>
    </tab-list>
    <tab-panel name="overview">The overview panel.</tab-panel>
    <tab-panel name="activity">The activity panel.</tab-panel>
  </tabs>

  <h2>The behavior is already wired</h2>
  <p>
    Keyboard traversal, focus management, filtering, ARIA — the parts every
    team gets subtly wrong ship inside the tag. The tabs above answer arrow
    keys, Home and End with a roving tabindex, and stamp
    <code>role="tab"</code> and <code>aria-selected</code> themselves. A
    dropdown closes on Escape and supports type-ahead. A switch is a real
    <code>&lt;input type="checkbox" role="switch"&gt;</code>, so Space, Tab
    and form submission are the platform's own. You write the options; the
    component does the rest. Try it:
  </p>
  <code-block code={code.behavior}></code-block>
  <combobox placeholder="Pick a framework" sx={style.maxW(panel-max-w-sm)}>
    <option value="sail">Sail</option>
    <option value="svelte">Svelte</option>
    <option value="solid">Solid</option>
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
    <icon name="git-branch"></icon>
    <icon name="compass"></icon>
    <icon name="zap"></icon>
    <icon name="rocket" size="lg"></icon>
    <icon name="alarm-clock" size="xl" label="Alarm"></icon>
    <icon name="refresh-cw" spin></icon>
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
      <icon name="trash-2"></icon> Delete
    </button>
  </row>
  <p sx={style.fontSize(sm).fg(muted-foreground)}>
    Accessibility is the <code>label</code> field — with it the icon is
    <code>role="img"</code> with an <code>aria-label</code>; without it the
    icon is marked <code>aria-hidden</code> — decoration, invisible to a
    screen reader. <code>size</code> runs <code>xs</code> to
    <code>xl</code>, <code>spin</code> rotates (and respects reduced-motion),
    and <code>src=</code> fetches any SVG URL when you need a glyph that
    isn't in the set.
  </p>

  <h2>Prose</h2>
  <p>
    Templates you author don't need typography classes — bare
    <code>h1</code>, <code>p</code> and <code>code</code> already take the
    design system's type scale. <code>&lt;prose&gt;</code> is for content you
    <em>render</em>: markdown output, a doc blob, anything that arrives as
    flowing text at runtime. Wrap it and the whole subtree gets reading
    typography — heading rhythm, list indentation, quote bars, inline code
    chips, table borders:
  </p>
  <code-block code={code.prose}></code-block>
  <card sx={style.pad(4)}>
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
  <p sx={style.fontSize(sm).fg(muted-foreground)}>
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

  <p sx={style.fontSize(sm).fg(muted-foreground)}>
    For the curious: most of these tags don't exist at runtime — the compiler
    lowers them to native markup with the behavior attached, which is why the
    library adds almost nothing to a page. A dialog ships as a real
    <code>&lt;dialog&gt;</code>. If that interests you, the
    <a href="/sail/reference">reference</a> covers the mechanics.
  </p>

  <row sx={style.mt(8).padT(4).borderTWidth(1).borderT(border)} justify="between" gap="2">
    <a variant="default" href="/sail/language/theming">← Theming</a>
    <a variant="solid" tone="primary" href="/sail/guide/building-a-component">Building a component →</a>
  </row>
</column>
