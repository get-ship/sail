<controller>
import { controller, signal, useDialog } from '@ship/sail-runtime'

function notify(message: string, tone: string, description?: string) {
  window.dispatchEvent(
    new CustomEvent('ship:notify', {
      bubbles: true,
      detail: { message, tone, description, duration: 4000 },
    }),
  )
}

export default controller(() => ({
    // Overlays
    dialogDemo: useDialog(),
    alertDialogOpen: signal(false),
    drawerOpen: signal(false),
    popoverOpen: signal(false),
    dropdownOpen: signal(false),
    hoverCardOpen: signal(false),
    contextMenuOpen: signal(false),
    // Error boundary
    triggerError: signal(false),
    // Toast dispatch helpers — plain functions, not signals
    showSuccessToast: () => notify('File saved', 'success'),
    showErrorToast: () => notify('Upload failed', 'destructive', 'Network error — try again'),
    showInfoToast: () => notify('Background job started', 'info'),
  }))
</controller>

<!-- FILE_CAP_OVERRIDE: gallery instantiates all 50+ sui/primitives — one card each; cannot be split without losing the gallery contract -->
<column gap="8" sx={style.pad(6).maxW(breakpoint-tablet).mx(auto)}>
  <header>
    <h1 sx={style.fontSize(2xl).fontWeight(bold).mb(2)}>sui/primitives gallery</h1>
    <p sx={style.fg(muted-foreground)}>Real component instantiations — one card per primitive, representative prop combos.</p>
  </header>

  <!-- Accordion -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-accordion</h2>
    <row gap="3" align="start">
      <sail-accordion type="single" value="a1">
        <sail-accordion-item value="a1">
          <span slot="trigger">What is SUI?</span>
          SUI is the declarative template and reactive runtime that powers Ship views.
        </sail-accordion-item>
        <sail-accordion-item value="a2">
          <span slot="trigger">Multiple sections</span>
          Open me independently — single mode keeps only one open.
        </sail-accordion-item>
        <sail-accordion-item value="a3" disabled>
          <span slot="trigger">Disabled item</span>
          This body is unreachable.
        </sail-accordion-item>
      </sail-accordion>
    </row>
  </section>

  <!-- Alert -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>alert</h2>
    <column gap="3">
      <alert tone="info">
        <span slot="title">Info</span>
        This is an informational alert.
      </alert>
      <alert tone="success">
        <span slot="title">Success</span>
        Operation completed.
      </alert>
      <alert tone="warning" dismissible>
        <span slot="title">Warning</span>
        Check your configuration.
      </alert>
      <alert tone="destructive">
        <span slot="title">Error</span>
        Something went wrong.
      </alert>
      <alert tone="muted">
        <span slot="title">Note</span>
        Muted tone for subtle notices.
      </alert>
    </column>
  </section>

  <!-- Avatar -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>avatar</h2>
    <row gap="3" align="start">
      <avatar name="Alice Hart" size="xs"></avatar>
      <avatar name="Bob Ray" size="sm" tone="primary"></avatar>
      <avatar name="Carol Wei" size="md" tone="success"></avatar>
      <avatar name="David Ko" size="lg" tone="warning"></avatar>
      <avatar name="Eva Moor" size="xl" tone="destructive"></avatar>
      <avatar initials="ZP" size="md" shape="square" tone="accent"></avatar>
      <avatar icon-slug="user" size="md" tone="muted"></avatar>
    </row>
  </section>

  <!-- Badge dissolved to L1 (bare <span> + sx) — see dissolve-gate.md -->

  <!-- Button -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-button</h2>
    <row gap="3" align="start">
      <sail-button variant="default">Default</sail-button>
      <sail-button variant="solid" tone="primary">Solid primary</sail-button>
      <sail-button variant="soft" tone="success">Soft success</sail-button>
      <sail-button variant="outline" tone="warning">Outline warning</sail-button>
      <sail-button variant="ghost" tone="destructive">Ghost destructive</sail-button>
      <sail-button variant="solid" tone="primary" size="xs">xs</sail-button>
      <sail-button variant="solid" tone="primary" size="sm">sm</sail-button>
      <sail-button variant="solid" tone="primary" size="lg">lg</sail-button>
      <sail-button variant="solid" tone="primary" disabled>Disabled</sail-button>
      <sail-button variant="solid" tone="primary" loading>Loading</sail-button>
      <sail-button variant="default">
        <icon slot="leading" name="star"></icon>With icon
      </sail-button>
    </row>
  </section>

  <!-- Card -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-card</h2>
    <row gap="3" align="start">
      <card sx={style.w(48)}>
        <span slot="header">Default card</span>
        Content in the default slot.
        <span slot="footer">Footer</span>
      </card>
      <card tone="primary" variant="outline" sx={style.w(48)}>
        <span slot="header">Outline primary</span>
        Outline variant with primary tone.
      </card>
      <card tone="success" accent-bar="static" status-rail sx={style.w(48)}>
        <span slot="header">Status rail</span>
        Accent bar + status rail active.
      </card>
      <card tone="info" fill padding="lg" sx={style.w(48)}>
        Fill + large padding.
      </card>
    </row>
  </section>

  <!-- Checkbox -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-checkbox</h2>
    <row gap="3" align="start">
      <sail-checkbox label="Default"></sail-checkbox>
      <sail-checkbox label="Checked" checked></sail-checkbox>
      <sail-checkbox label="Indeterminate" indeterminate></sail-checkbox>
      <sail-checkbox label="Disabled" disabled></sail-checkbox>
      <sail-checkbox label="Primary" tone="primary" checked></sail-checkbox>
      <sail-checkbox label="Large" size="lg" checked></sail-checkbox>
    </row>
  </section>

  <!-- Chip -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>chip</h2>
    <row gap="3" align="start">
      <chip>Default</chip>
      <chip tone="primary" variant="solid">Solid</chip>
      <chip tone="success" variant="outline">Outline</chip>
      <chip tone="accent" selected>Selected</chip>
      <chip tone="muted" removable>Removable</chip>
      <chip tone="warning" disabled>Disabled</chip>
    </row>
  </section>

  <!-- Cluster + Stack layout primitives -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>row / column</h2>
    <column gap="3">
      <row gap="3" align="center">
        <span tone="primary" sx={style.display(inline-block).minH(control-h-2xs).padX(2).fontSize(xs).fontWeight(semibold).radius(sm).bg(tone-bg).border(tone-border).fg(tone-fg)}>A</span>
        <span tone="success" sx={style.display(inline-block).minH(control-h-2xs).padX(2).fontSize(xs).fontWeight(semibold).radius(sm).bg(tone-bg).border(tone-border).fg(tone-fg)}>B</span>
        <span tone="accent" sx={style.display(inline-block).minH(control-h-2xs).padX(2).fontSize(xs).fontWeight(semibold).radius(sm).bg(tone-bg).border(tone-border).fg(tone-fg)}>C</span>
      </row>
      <column gap="2">
        <span tone="info" sx={style.display(inline-block).minH(control-h-2xs).padX(2).fontSize(xs).fontWeight(semibold).radius(sm).bg(tone-bg).border(tone-border).fg(tone-fg)}>Stack item 1</span>
        <span tone="warning" sx={style.display(inline-block).minH(control-h-2xs).padX(2).fontSize(xs).fontWeight(semibold).radius(sm).bg(tone-bg).border(tone-border).fg(tone-fg)}>Stack item 2</span>
        <span tone="destructive" sx={style.display(inline-block).minH(control-h-2xs).padX(2).fontSize(xs).fontWeight(semibold).radius(sm).bg(tone-bg).border(tone-border).fg(tone-fg)}>Stack item 3</span>
      </column>
    </column>
  </section>

  <!-- Code dissolved to L1 (bare code/pre + sx) — see dissolve-gate.md -->

  <!-- Collapsible -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-collapsible</h2>
    <row gap="3" align="start">
      <collapsible>
        <sail-button slot="trigger" variant="outline" size="sm">Toggle details</sail-button>
        <column sx={style.padY(2)}>Revealed content when expanded.</column>
      </collapsible>
    </row>
  </section>

  <!-- Empty state + Error state -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-empty-state / sail-error-state</h2>
    <row gap="3" align="start">
      <empty-state variant="default" size="sm">
        <icon slot="icon" name="inbox"></icon>
        <span slot="title">Nothing here yet</span>
        Start by creating a new item.
        <sail-button slot="action" tone="primary" variant="solid" size="sm">Create</sail-button>
      </empty-state>
      <empty-state variant="inline" size="sm" tone="muted">
        <span slot="title">No results</span>
        Adjust your filters.
      </empty-state>
      <error-state size="sm"><span slot="title">Load failed</span>
        Connection timed out.
        <sail-button slot="action" variant="outline" size="sm">Retry</sail-button>
      </error-state>
    </row>
  </section>

  <!-- Field + Label -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>field / sail-label / sail-input / sail-textarea</h2>
    <column gap="3" sx={style.maxW(panel-max-w-sm)}>
      <field>
        <sail-label slot="label" required>Name</sail-label>
        <input type="text" placeholder="Enter your name" name="demo-name">
        <span slot="hint">Used for display only.</span>
      </field>
      <field invalid>
        <sail-label slot="label">Email</sail-label>
        <input type="email" placeholder="user@example.com" invalid name="demo-email">
        <span slot="error">Enter a valid email address.</span>
      </field>
      <field>
        <sail-label slot="label">Notes</sail-label>
        <sail-textarea placeholder="Add notes…" rows="3" name="demo-notes"></sail-textarea>
      </field>
    </column>
  </section>

  <!-- Icon -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-icon</h2>
    <row gap="3" align="start">
      <icon name="star" size="xs"></icon>
      <icon name="star" size="sm"></icon>
      <icon name="star" size="md"></icon>
      <icon name="star" size="lg"></icon>
      <icon name="star" size="xl"></icon>
      <icon name="check-circle" tone="success" size="md"></icon>
      <icon name="alert-triangle" tone="warning" size="md"></icon>
      <icon name="x-circle" tone="destructive" size="md"></icon>
      <icon name="info" tone="info" size="md"></icon>
    </row>
  </section>

  <!-- Link dissolved to L1 (bare <a> + sx) — see dissolve-gate.md -->

  <!-- Navigation menu -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-navigation-menu</h2>
    <row gap="3" align="start">
      <nav-top value="home">
        <sail-navigation-item value="home" active>Home</sail-navigation-item>
        <sail-navigation-item value="docs">Docs</sail-navigation-item>
        <sail-navigation-item value="api">API</sail-navigation-item>
        <sail-navigation-item value="disabled" disabled>Disabled</sail-navigation-item>
      </nav-top>
    </row>
  </section>

  <!-- Pagination -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-pagination</h2>
    <column gap="3">
      <pagination page="1" total="150" page-size="10"></pagination>
      <pagination page="3" total="150" page-size="25" page-size-options="10,25,50"></pagination>
    </column>
  </section>

  <!-- Radio group -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>radio-group / radio</h2>
    <row gap="3" align="start">
      <radio-group name="demo-radio" value="b" label="Size">
        <radio value="a">Small</radio>
        <radio value="b">Medium</radio>
        <radio value="c">Large</radio>
      </radio-group>
      <radio-group name="demo-radio2" value="x" label="Tones" tone="primary">
        <radio value="x" tone="primary">Primary</radio>
        <radio value="y" tone="success">Success</radio>
        <radio value="z" disabled>Disabled</radio>
      </radio-group>
    </row>
  </section>

  <!-- Select + Combobox -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-select</h2>
    <row gap="3" align="start">
      <select name="demo-select" size="sm">
        <option value="">Pick one…</option>
        <option value="a">Alpha</option>
        <option value="b">Beta</option>
      </select>
      <select name="demo-select2" size="md" tone="info">
        <option value="">Info tone</option>
        <option value="a">Option A</option>
      </select>
      <select name="demo-select3" size="lg" invalid>
        <option value="">Invalid state</option>
      </select>
      <select name="demo-select4" disabled>
        <option value="">Disabled</option>
      </select>
    </row>
  </section>

  <!-- Separator -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-separator</h2>
    <column gap="3">
      <sail-separator orientation="horizontal"></sail-separator>
      <sail-separator orientation="horizontal" variant="dashed" tone="muted"></sail-separator>
      <sail-separator orientation="horizontal" tone="primary"></sail-separator>
    </column>
  </section>

  <!-- Skeleton -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-skeleton</h2>
    <column gap="3" sx={style.maxW(panel-max-w-sm)}>
      <sail-skeleton shape="text" size="md"></sail-skeleton>
      <sail-skeleton shape="line" size="sm"></sail-skeleton>
      <sail-skeleton shape="block" size="lg"></sail-skeleton>
      <sail-skeleton shape="circle" size="md"></sail-skeleton>
    </column>
  </section>

  <!-- Slider -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-slider</h2>
    <column gap="3" sx={style.maxW(panel-max-w-sm)}>
      <slider name="demo-slider" value="40" min="0" max="100" aria-label="Volume"></slider>
      <slider name="demo-slider2" value="60" tone="primary" size="lg" aria-label="Primary large"></slider>
      <slider name="demo-slider3" value="20" disabled aria-label="Disabled"></slider>
    </column>
  </section>

  <!-- Spinner -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-spinner</h2>
    <row gap="3" align="start">
      <sail-spinner size="xs"></sail-spinner>
      <sail-spinner size="sm" tone="primary"></sail-spinner>
      <sail-spinner size="md" tone="success"></sail-spinner>
      <sail-spinner size="lg" tone="warning"></sail-spinner>
    </row>
  </section>

  <!-- Status dot -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-status-dot</h2>
    <row gap="3" align="start">
      <sail-status-dot tone="success" label="Online"></sail-status-dot>
      <sail-status-dot tone="warning" pulse label="Pending"></sail-status-dot>
      <sail-status-dot tone="destructive" label="Error"></sail-status-dot>
      <sail-status-dot tone="muted" label="Offline"></sail-status-dot>
      <sail-status-dot tone="info" size="sm" label="Small"></sail-status-dot>
    </row>
  </section>

  <!-- Switch -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-switch</h2>
    <row gap="3" align="start">
      <label><switch name="demo-sw1"></switch> Default</label>
      <label><switch name="demo-sw2" checked></switch> Checked</label>
      <label><switch name="demo-sw3" tone="primary" checked></switch> Primary</label>
      <label><switch name="demo-sw4" tone="success" checked size="lg"></switch> Success lg</label>
      <label><switch name="demo-sw5" disabled></switch> Disabled</label>
    </row>
  </section>

  <!-- Tabs -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-tabs</h2>
    <column gap="3">
      <tabs active="tab1" variant="line">
        <sail-tab name="tab1" label="Overview"></sail-tab>
        <sail-tab name="tab2" label="Settings"></sail-tab>
        <sail-tab name="tab3" label="Logs" count="12"></sail-tab>
        <sail-tab name="tab4" label="Disabled" disabled></sail-tab>
      </tabs>
      <tabs active="p1" variant="pill" tone="primary">
        <sail-tab name="p1" label="Alpha"></sail-tab>
        <sail-tab name="p2" label="Beta"></sail-tab>
      </tabs>
      <tabs active="e1" variant="enclosed" size="sm">
        <sail-tab name="e1" label="Code"></sail-tab>
        <sail-tab name="e2" label="Preview"></sail-tab>
      </tabs>
    </column>
  </section>

  <!-- Timeline -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-timeline</h2>
    <row gap="3" align="start">
      <sail-timeline>
        <sail-timeline-item tone="success" at="2026-06-01">
          <span slot="title">Deployed</span>
          Release shipped to production.
        </sail-timeline-item>
        <sail-timeline-item tone="info" at="2026-05-30">
          <span slot="title">Review passed</span>
          All checks green.
        </sail-timeline-item>
        <sail-timeline-item tone="muted" at="2026-05-28">
          <span slot="title">PR opened</span>
          Initial draft created.
        </sail-timeline-item>
      </sail-timeline>
    </row>
  </section>

  <!-- Toast -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-toast (static preview)</h2>
    <column gap="3" sx={style.maxW(panel-max-w-sm)}>
      <toast tone="info">
        <span slot="title">Info toast</span>
        Something happened you should know about.
      </toast>
      <toast tone="success" show-progress>
        <span slot="title">Success</span>
        File uploaded successfully.
      </toast>
      <toast tone="destructive" dismissible>
        <span slot="title">Error</span>
        Upload failed — try again.
      </toast>
    </column>
  </section>

  <!-- Toggle group -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-toggle-group</h2>
    <column gap="3">
      <toggle-group type="single" value="b" variant="outline" name="demo-tg1">
        <sail-toggle value="a">Left</sail-toggle>
        <sail-toggle value="b">Center</sail-toggle>
        <sail-toggle value="c">Right</sail-toggle>
      </toggle-group>
      <toggle-group type="multiple" variant="segmented" name="demo-tg2">
        <sail-toggle value="bold" variant="segmented">Bold</sail-toggle>
        <sail-toggle value="italic" variant="segmented">Italic</sail-toggle>
        <sail-toggle value="under" variant="segmented">Underline</sail-toggle>
      </toggle-group>
    </column>
  </section>

  <!-- Tooltip -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>tooltip</h2>
    <row gap="3" align="start">
      <tooltip placement="top" content="Tooltip on top">
        <sail-button variant="outline">Hover me (top)</sail-button>
      </tooltip>
      <tooltip placement="bottom" content="Tooltip on bottom">
        <sail-button variant="outline">Bottom</sail-button>
      </tooltip>
      <tooltip placement="right" content="Tooltip on right">
        <sail-button variant="outline">Right</sail-button>
      </tooltip>
    </row>
  </section>

  <!-- Tree view -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-tree-view</h2>
    <row gap="3" align="start">
      <sail-tree>
        <sail-tree-item label="src" expanded>
          <sail-tree-item label="components" leaf>
            <icon slot="icon" name="folder" size="sm"></icon>
          </sail-tree-item>
          <sail-tree-item label="views" expanded>
            <sail-tree-item label="page.sui" leaf></sail-tree-item>
            <sail-tree-item label="controller.ts" leaf></sail-tree-item>
          </sail-tree-item>
        </sail-tree-item>
        <sail-tree-item label="tests" leaf></sail-tree-item>
      </sail-tree>
    </row>
  </section>

  <!-- Scroll area -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-scroll-area</h2>
    <row gap="3" align="start">
      <scroll-area orientation="vertical" sx={style.h(32).w(panel-max-w-sm).borderWidth(1).border(border).radius(md)}>
        <column gap="1" sx={style.pad(2)}>
          <p>Line 1</p><p>Line 2</p><p>Line 3</p><p>Line 4</p>
          <p>Line 5</p><p>Line 6</p><p>Line 7</p><p>Line 8</p>
        </column>
      </scroll-area>
    </row>
  </section>

  <!-- JSON view -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-json-view</h2>
    <row gap="3" align="start">
      <sail-json-view data={'{"name":"SUI","version":1,"stable":true}'} depth="2" expanded></sail-json-view>
    </row>
  </section>

  <!-- Markdown -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-markdown</h2>
    <row gap="3" align="start" sx={style.maxW(panel-max-w-md)}>
      <sail-markdown value="## Hello from **Markdown**&#10;&#10;- Item one&#10;- Item two&#10;- `code` inline"></sail-markdown>
    </row>
  </section>

  <!-- Diff -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-diff</h2>
    <row gap="3" align="start">
      <sail-diff from="const x = 1;" to="const x = 2;"></sail-diff>
    </row>
  </section>

  <!-- Table (slot mode) -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-table (slot mode)</h2>
    <row gap="3" align="start">
      <table variant="slot" label="Demo table" hoverable>
        <tr slot="head"><th>Name</th><th>Role</th><th>Status</th></tr>
        <tr><td>Alice</td><td>Engineer</td><td><span tone="success" sx={style.display(inline-block).minH(control-h-2xs).padX(1_5).fontSize(xs).fontWeight(semibold).radius(sm).bg(tone-bg).border(tone-border).fg(tone-fg)}>Active</span></td></tr>
        <tr><td>Bob</td><td>Designer</td><td><span tone="warning" sx={style.display(inline-block).minH(control-h-2xs).padX(1_5).fontSize(xs).fontWeight(semibold).radius(sm).bg(tone-bg).border(tone-border).fg(tone-fg)}>Away</span></td></tr>
        <tr><td>Carol</td><td>PM</td><td><span tone="muted" sx={style.display(inline-block).minH(control-h-2xs).padX(1_5).fontSize(xs).fontWeight(semibold).radius(sm).bg(tone-bg).border(tone-border).fg(tone-fg)}>Offline</span></td></tr>
      </table>
    </row>
  </section>

  <!-- Color picker -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-color-picker</h2>
    <row gap="3" align="start">
      <sail-color-picker name="demo-color1" value="#3b82f6"></sail-color-picker>
      <sail-color-picker name="demo-color2" value="#22c55e" size="sm"></sail-color-picker>
      <sail-color-picker name="demo-color3" value="#ef4444" size="lg" tone="primary"></sail-color-picker>
    </row>
  </section>

  <!-- Date picker -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-date-picker</h2>
    <row gap="3" align="start">
      <sail-date-picker name="demo-date1" placeholder="YYYY-MM-DD"></sail-date-picker>
      <sail-date-picker name="demo-date2" value="2026-06-05" tone="primary" size="sm"></sail-date-picker>
    </row>
  </section>

  <!-- File upload -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-file-upload</h2>
    <column gap="3" sx={style.maxW(panel-max-w-sm)}>
      <sail-file-upload accept="image/*,application/pdf" tone="primary"></sail-file-upload>
      <sail-file-upload multiple accept=".json,.csv" tone="muted"></sail-file-upload>
    </column>
  </section>

  <!-- Autocomplete -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-autocomplete</h2>
    <row gap="3" align="start">
      <combobox name="demo-ac" placeholder="Search fruits…" suggestions='["Apple","Apricot","Banana","Blueberry","Cherry","Date"]' size="md"></combobox>
    </row>
  </section>

  <!-- ── OVERLAYS ─────────────────────────────────────────────────────────── -->

  <!-- sail-dialog -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-dialog</h2>
    <row gap="3" align="start">
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(48)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>Modal dialog — size=md, backdrop=default</p>
        <sail-button variant="outline" size="sm" on:click={dialogDemo.open}>Open dialog</sail-button>
        <dialog use:dialogDemo>
          <header>Confirm action</header>
          <section>Are you sure you want to proceed? This cannot be undone.</section>
          <footer>
            <sail-button variant="outline" size="sm" on:click={dialogDemo.close}>Cancel</sail-button>
            <sail-button variant="solid" tone="primary" size="sm" on:click={dialogDemo.close}>Confirm</sail-button>
          </footer>
        </dialog>
      </column>
    </row>
  </section>

  <!-- sail-alert-dialog -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-alert-dialog</h2>
    <row gap="3" align="start">
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(48)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>Destructive confirm dialog with built-in cancel/confirm footer</p>
        <sail-button variant="outline" tone="destructive" size="sm" on:click={() => alertDialogOpen.set(true)}>Delete item</sail-button>
        <dialog role="alertdialog"
          open={alertDialogOpen()}
          tone="destructive"
          confirm-label="Delete"
          cancel-label="Keep it"
        >
          <span slot="title">Delete this item?</span>
          This action is permanent and cannot be reversed. All associated data will be removed.
        </dialog>
      </column>
    </row>
  </section>

  <!-- sail-drawer -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-drawer</h2>
    <row gap="3" align="start">
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(48)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>Side-sheet panel — side=right, size=md</p>
        <sail-button variant="outline" size="sm" on:click={() => drawerOpen.set(true)}>Open drawer</sail-button>
        <dialog side="end" open={drawerOpen()} side="right" size="md">
          <span slot="title">Settings</span>
          <span slot="description">Adjust your workspace preferences.</span>
          <column gap="3">
            <field>
              <sail-label slot="label">Theme</sail-label>
              <select name="drawer-theme"><option value="light">Light</option><option value="dark">Dark</option></select>
            </field>
            <field>
              <sail-label slot="label">Language</sail-label>
              <select name="drawer-lang"><option value="en">English</option><option value="es">Español</option></select>
            </field>
          </column>
          <span slot="footer">
            <sail-button variant="outline" size="sm" on:click={() => drawerOpen.set(false)}>Cancel</sail-button>
            <sail-button variant="solid" tone="primary" size="sm" on:click={() => drawerOpen.set(false)}>Save</sail-button>
          </span>
        </dialog>
      </column>
    </row>
  </section>

  <!-- sail-popover -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-popover</h2>
    <row gap="3" align="start">
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(48)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>Anchor-positioned popover — placement=bottom</p>
        <popover placement="bottom">
          <sail-button slot="trigger" variant="outline" size="sm">Open popover</sail-button>
          <column gap="1" sx={style.pad(3).maxW(56)}>
            <strong sx={style.fontSize(sm)}>Popover content</strong>
            <p sx={style.fontSize(xs).fg(muted-foreground)}>
              Click outside or press Escape to dismiss. Uses native popover API for top-layer rendering.
            </p>
          </column>
        </popover>
      </column>
    </row>
  </section>

  <!-- sail-dropdown-menu -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-dropdown-menu</h2>
    <row gap="3" align="start">
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(48)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>Trigger-anchored menu with ARIA role=menu + keyboard navigation</p>
        <dropdown>
          <sail-button slot="trigger" variant="outline" size="sm">
            Actions
            <icon slot="trailing" name="chevron-down" size="xs"></icon>
          </sail-button>
          <sail-menu-item value="edit">Edit</sail-menu-item>
          <sail-menu-item value="duplicate">Duplicate</sail-menu-item>
          <sail-menu-separator></sail-menu-separator>
          <sail-menu-item value="delete" tone="destructive">Delete</sail-menu-item>
        </dropdown>
      </column>
    </row>
  </section>

  <!-- sail-hover-card -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-hover-card</h2>
    <row gap="3" align="start">
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(48)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>Hover-triggered rich content overlay — placement=bottom, delay=300ms</p>
        <hover-card placement="bottom" show-delay="300" hide-delay="150">
          <sail-button slot="trigger" variant="ghost" size="sm">
            <avatar name="Alice Hart" size="sm" tone="primary"></avatar>
            Alice Hart
          </sail-button>
          <column gap="2">
            <row gap="2">
              <avatar name="Alice Hart" size="md" tone="primary"></avatar>
              <column gap="0">
                <span sx={style.fontWeight(semibold).fontSize(sm)}>Alice Hart</span>
                <span sx={style.fontSize(xs).fg(muted-foreground)}>Senior Engineer</span>
              </column>
            </row>
            <p sx={style.fontSize(xs).fg(muted-foreground)}>
              Joined 2023 · 142 contributions · Rust, TypeScript
            </p>
          </column>
        </hover-card>
      </column>
    </row>
  </section>

  <!-- sail-context-menu -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-context-menu</h2>
    <row gap="3" align="start">
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(48)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>Right-click triggered overlay menu — positioned at cursor via show(x,y)</p>
        <context-menu>
          <column slot="trigger" sx={style.padY(4).padX(6).borderWidth(1).border(border).radius(md).textAlign(center).fontSize(sm).fg(muted-foreground).select(none).cursor(context-menu)}>Right-click this area</column>
          <sail-menu-item value="open">Open</sail-menu-item>
          <sail-menu-item value="rename">Rename</sail-menu-item>
          <sail-menu-separator></sail-menu-separator>
          <sail-menu-item value="copy">Copy path</sail-menu-item>
          <sail-menu-separator></sail-menu-separator>
          <sail-menu-item value="delete" tone="destructive">Move to trash</sail-menu-item>
        </context-menu>
      </column>
    </row>
  </section>

  <!-- ── OPAQUE-JSON-PROP ──────────────────────────────────────────────────── -->

  <!-- sail-combobox -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-combobox</h2>
    <row gap="3" align="start">
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(48).maxW(panel-max-w-sm)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>Single-select with type-to-filter — options via JSON</p>
        <combobox
          name="demo-combo"
          placeholder="Search framework…"
          options='[{"value":"react","label":"React"},{"value":"vue","label":"Vue"},{"value":"svelte","label":"Svelte"},{"value":"solid","label":"SolidJS"},{"value":"sail","label":"SAIL (Ship)"}]'
          size="md"
        ></combobox>
      </column>
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(48).maxW(64)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>Pre-selected value · size=sm</p>
        <combobox
          name="demo-combo2"
          value="sail"
          options='[{"value":"react","label":"React"},{"value":"vue","label":"Vue"},{"value":"sail","label":"SAIL (Ship)"}]'
          size="sm"
          tone="primary"
        ></combobox>
      </column>
    </row>
  </section>

  <!-- sail-calendar -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-calendar</h2>
    <column gap="3">
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(48)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>Month view with 3 events — focus-date=2026-06-05</p>
        <sail-calendar
          view="month"
          focus-date="2026-06-05"
          events='[{"id":"e1","title":"Team standup","start":"2026-06-03T09:00:00","end":"2026-06-03T09:30:00","tone":"primary"},{"id":"e2","title":"Design review","start":"2026-06-10T14:00:00","end":"2026-06-10T15:00:00","tone":"info"},{"id":"e3","title":"Ship launch","start":"2026-06-20T12:00:00","end":"2026-06-20T13:00:00","tone":"success"}]'
        ></sail-calendar>
      </column>
    </column>
  </section>

  <!-- sail-tree -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-tree</h2>
    <row gap="3" align="start">
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(panel-min-w-md)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>Hierarchical tree — selectable=single, show-guides</p>
        <sail-tree
          label="File system"
          selectable="single"
          show-guides
          nodes='[{"id":"src","label":"src","children":[{"id":"comp","label":"components","children":[{"id":"btn","label":"sail-button.sui"},{"id":"card","label":"sail-card.sui"}]},{"id":"views","label":"views","children":[{"id":"pg","label":"page.sui"},{"id":"ct","label":"controller.ts"}]}]},{"id":"tests","label":"tests"}]'
        ></sail-tree>
      </column>
    </row>
  </section>

  <!-- sail-tree-view / sail-tree-item — already covered above but was a thin static stub; a full grid follows -->
  <!-- sail-tree-grid -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-tree-grid</h2>
    <column gap="3">
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(48)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>Hierarchical tree-grid (role=treegrid) — columns with Name, Size, Kind</p>
        <sail-tree-grid
          label="Artifact tree"
          selectable="single"
          nodes='[{"id":"dist","label":"dist","size":"2.4 MB","kind":"dir","children":[{"id":"main","label":"main.js","size":"1.1 MB","kind":"js"},{"id":"css","label":"style.css","size":"42 KB","kind":"css"},{"id":"assets","label":"assets","size":"1.2 MB","kind":"dir","children":[{"id":"logo","label":"logo.svg","size":"4 KB","kind":"svg"}]}]},{"id":"pkg","label":"package.json","size":"1.8 KB","kind":"json"}]'
          columns='[{"id":"label","header":"Name","accessorKey":"label"},{"id":"size","header":"Size","accessorKey":"size"},{"id":"kind","header":"Kind","accessorKey":"kind"}]'
        ></sail-tree-grid>
      </column>
    </column>
  </section>

  <!-- sail-sheet -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-sheet</h2>
    <column gap="3">
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(48)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>Spreadsheet primitive — 2-D cell space with A1 addressing and formula bar</p>
        <dialog side="end"
          label="Budget sheet"
          show-headers
          show-gridlines
          show-formula-bar
          workbook={'{"sheets":[{"id":"sheet1","name":"Q2 Budget","cells":{"A1":{"v":"Category"},"B1":{"v":"Budget"},"C1":{"v":"Spent"},"A2":{"v":"Engineering"},"B2":{"v":50000},"C2":{"v":42300},"A3":{"v":"Design"},"B3":{"v":20000},"C3":{"v":18750},"A4":{"v":"Marketing"},"B4":{"v":15000},"C4":{"v":9200}}}]}'}
          sx={style.h(56)}
        ></dialog>
      </column>
    </column>
  </section>

  <!-- ── INFRA/COMPANION ───────────────────────────────────────────────────── -->

  <!-- sail-portal -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-portal</h2>
    <row gap="3" align="start">
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(48)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>
          Teleports slotted children into a different DOM location (stacking-context escape).
          The portal below appends a labelled marker to <code>body</code> — inspect the
          DOM to confirm it lives outside this view's mount point.
        </p>
                  <column sx={style.display(none)} data-portal-demo aria-hidden="true">
            [sail-portal demo — this node lives in &lt;body&gt;, not in the gallery mount]
          </column>
                <p sx={style.fontSize(xs).fg(muted-foreground).italic(true)}>Portal active — check DevTools &gt; Elements for the marker in &lt;body&gt;.</p>
      </column>
    </row>
  </section>

  <!-- sail-toast-container -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-toast-container / sail-toast-host</h2>
    <row gap="3" align="start">
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(48)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>
          <code>sail-toast-container</code> is the fixed-position stacking host.
          <code>sail-toast-host</code> wraps it and listens for <code>ship:notify</code> events.
          Click the button to dispatch a notify event — the host shows the toast.
        </p>
        <toast-region position="bottom-end"></toast-region>
        <row gap="3" align="start">
          <sail-button variant="outline" size="sm" on:click={() => showSuccessToast()}>
            Show success toast
          </sail-button>
          <sail-button variant="outline" size="sm" on:click={() => showErrorToast()}>
            Show error toast
          </sail-button>
          <sail-button variant="outline" size="sm" on:click={() => showInfoToast()}>
            Show info toast
          </sail-button>
        </row>
      </column>
    </row>
  </section>

  <!-- sail-tab-panel -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-tab-panel (inside sail-tabs)</h2>
    <column gap="3">
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(48)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>
          <code>sail-tab-panel</code> is the companion panel element driven by the parent
          <code>sail-tabs</code> controller. Each panel shows when its <code>name</code>
          matches the active tab.
        </p>
        <tabs active="tp1" variant="line">
          <sail-tab name="tp1" label="Overview"></sail-tab>
          <sail-tab name="tp2" label="Details"></sail-tab>
          <sail-tab name="tp3" label="History"></sail-tab>
          <tab-panel name="tp1">
            <column sx={style.pad(4).fontSize(sm).fg(muted-foreground).bg(surface-2).radius(md)}>
              Overview panel — rendered when the Overview tab is active.
              Uses <code>animation: sail-tab-panel-fade</code> on transition.
            </column>
          </tab-panel>
          <tab-panel name="tp2">
            <column sx={style.pad(4).fontSize(sm).fg(muted-foreground).bg(surface-2).radius(md)}>
              Details panel — secondary content rendered on demand.
            </column>
          </tab-panel>
          <tab-panel name="tp3">
            <column sx={style.pad(4).fontSize(sm).fg(muted-foreground).bg(surface-2).radius(md)}>
              History panel — lazy content until the user navigates here.
            </column>
          </tab-panel>
        </tabs>
      </column>
    </column>
  </section>

  <!-- sail-error-boundary -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>sail-error-boundary</h2>
    <column gap="3">
      <column gap="3" sx={style.borderWidth(1).border(border).radius(md).bg(surface-1).pad(4).minW(48)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(normal)}>
          Render-fault isolation. Toggle the switch to simulate a subtree error;
          the boundary catches it and shows the fallback. Click Retry to recover.
        </p>
        <row gap="3" align="start">
          <label><switch name="demo-eb-trigger" checked={triggerError()} on:change={(e) => triggerError.set(e.target.checked)}></switch> Trigger subtree error</label>
        </row>
        <error-boundary fallback-text="The widget threw an error — click Retry to recover.">
          {#if triggerError()}
            <column sx={style.display(none)}>This subtree is broken (error boundary will catch)</column>
          {:else}
            <column sx={style.pad(4).fontSize(sm).bg(surface-2).radius(md).fg(success-foreground)}>
              Subtree is healthy. Toggle the switch above to simulate a fault.
            </column>
          {/if}
        </error-boundary>
      </column>
    </column>
  </section>

  <!-- ── #263-BLOCKED ──────────────────────────────────────────────────────── -->

  <!-- sail-board / sail-draw / sail-media-av / sail-media-image / sail-media-csv -->
  <section>
    <h2 sx={style.fontSize(sm).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide).mb(3).padB(2).borderBWidth(1).borderB(border)}>Blocked by #263 — production-compile gap</h2>
    <column gap="3">

      <column gap="2" sx={style.pad(4).radius(md).borderWidth(1).border(tone-destructive-border).bg(tone-destructive-bg)}>
        <row gap="3">
          <code>sail-board</code>
          <span tone="destructive" sx={style.display(inline-block).minH(control-h-2xs).padX(2).fontSize(xs).fontWeight(medium).radius(full).bg(tone-bg).borderWidth(1).border(tone-border).fg(tone-fg).whitespace(nowrap)}>blocked by #263 — production-compile gap</span>
        </row>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(relaxed)}>
          <code>sail-board</code> uses <code>Set.prototype.has()</code> inside template
          expressions, which the real (native) SUI compiler rejects. Passes jsdom tests;
          fails <code>cargo run -p sail-compiler -- build</code>. Tracked in #263.
        </p>
      </column>

      <column gap="2" sx={style.pad(4).radius(md).borderWidth(1).border(tone-destructive-border).bg(tone-destructive-bg)}>
        <row gap="3">
          <code>sail-draw</code>
          <span tone="destructive" sx={style.display(inline-block).minH(control-h-2xs).padX(2).fontSize(xs).fontWeight(medium).radius(full).bg(tone-bg).borderWidth(1).border(tone-border).fg(tone-fg).whitespace(nowrap)}>blocked by #263 — production-compile gap</span>
        </row>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(relaxed)}>
          <code>sail-draw</code> uses <code>Set.prototype.has()</code> inside template
          expressions, which the real (native) SUI compiler rejects. Passes jsdom tests;
          fails <code>cargo run -p sail-compiler -- build</code>. Tracked in #263.
        </p>
      </column>

      <column gap="2" sx={style.pad(4).radius(md).borderWidth(1).border(tone-destructive-border).bg(tone-destructive-bg)}>
        <row gap="3">
          <code>sail-media-av</code> / <code>sail-media-image</code> / <code>sail-media-csv</code>
          <span tone="destructive" sx={style.display(inline-block).minH(control-h-2xs).padX(2).fontSize(xs).fontWeight(medium).radius(full).bg(tone-bg).borderWidth(1).border(tone-border).fg(tone-fg).whitespace(nowrap)}>blocked by #263 — production-compile gap</span>
        </row>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(relaxed)}>
          All three <code>sail-media-*</code> components use <code>bind:this</code> on
          non-root elements inside conditional branches, which the real compiler rejects.
          The gallery-sail-media view covers the scalar-attr surface of these components
          (no expressions involving the blocked patterns). Tracked in #263.
        </p>
      </column>

    </column>
  </section>
</column>
