<!--
  /components — the component browser. A pack rail, category chips, and live
  search compose over one catalog (registry.ts); every tile below renders the
  REAL component — nothing on this page is a drawing of a component.

  The `lang` tiles are the complete 48-element L1 surface — the same list
  the l1-conformance gate renders on every commit. Each tile gates itself
  on `show.<key>()` (the zero-arg visibility read the controller derives
  from the three filters), so hiding a tile unmounts the live component
  and the grid packs itself. Every tile links to its /components/<slug>
  detail page.

  Overlay-family tiles (popover, dropdown, hover-card) show their TRIGGER —
  the panel is a native [popover] wired by popovertarget, so the tile stays
  honest: what you see closed is what a page shows closed.

  All 25 pack components are un-gated (their `sail.config.json` now grants
  the design-system capabilities — sx / raw_containers / variants — their
  sources use). Three components are still counted here but not mounted
  anywhere in the compiled app: the 7 canvas-pack tiles (sail-canvas /
  sail-board / sail-draw / sail-graph-canvas / sail-annotation-*) trip their
  world-layer actions when instantiated inside this page's already-large
  component tree (renders fine via direct /components/<slug> nav);
  sail-message-list and sail-media-csv both transitively import
  `@ship/sail-runtime/virtual` (message-list directly, media-csv via
  `createTable` from the `@ship/sail-primitives` barrel, which side-effect-
  imports sail-tree/sail-tree-grid) — a framework-singleton subpath with no
  deck-bundler importmap entry (see apps/sail-site/src/register-all.ts).
  That is a pre-existing bundler gap, not a floor issue; closing it is a
  separate job. Until then neither tag may be referenced by ANY compiled
  route, canvas tiles included, or the app build fails outright — so
  sail-message-list and sail-media-csv have no live route at all right now.
-->

<controller>
// /components — the browser. Three filters compose over one catalog (the
// full 48-element L1 surface): the pack rail, the category chips, and
// the search box (the runtime's filteredList composition). Their
// intersection is a set of visible tile keys; every tile in page.sail
// gates itself on `show.<key>()`, so the markup for each live component
// stays hand-written while the filtering stays data-driven.
//
// Every rendered tile is the REAL component. The 6 packs (charts, media,
// canvas, chat, mermaid, code-editor) are un-gated (granted the design-
// system capabilities via their `sail.config.json`); every pack component
// is in the catalog and counts. The canvas tiles are skipped in THIS grid
// (see the file banner) but still render via their own
// /components/<slug> route. sail-message-list and sail-media-csv are
// skipped everywhere (see the file banner) — no live route yet.

import { controller, signal, derived, filteredList, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { CATALOG, PACKS, CATEGORIES, packCount, type CatalogEntry } from './registry'

export default controller((_ctx: ControllerScope<Scope>) => {
    const { query, filtered } = filteredList(
      () => CATALOG,
      (c: CatalogEntry, q: string) => (c.tag + ' ' + c.cat + ' ' + c.pack).toLowerCase().includes(q),
    )
    // Default to L1 only — mounting charts+mermaid+code-editor+chat+all L1
    // at once OOMs tabs after Style cutover (each tile is a live component).
    // User still can pick "All packs" or a pack rail entry.
    const packSel = signal<string>('lang')
    const catSel = signal<string>('all')
    const pickPack = (id: string): void => packSel.set(id)
    const pickCat = (id: string): void => catSel.set(id)

    // Skipped in THIS grid — canvas's world-layer actions throw when
    // mounted in this already-large page (still renders fine via direct
    // /components/<slug> navigation). sail-message-list and sail-media-csv
    // are skipped EVERYWHERE — both transitively need
    // `@ship/sail-runtime/virtual`, a framework-singleton subpath deck's
    // importmap doesn't declare (see the file banner); referencing either
    // tag from ANY route fails the whole app build.
    const SKIP_TAGS = new Set([
      'sail-canvas', 'sail-board', 'sail-draw', 'sail-graph-canvas',
      'sail-annotation-layer', 'sail-annotation-list', 'sail-annotation-marker',
      'sail-message-list', 'sail-media-csv',
    ])
    const visible = derived(() => {
      const p = packSel()
      const c = catSel()
      const pass = new Set<string>()
      for (const e of filtered()) {
        if (SKIP_TAGS.has(e.tag)) continue
        if ((p === 'all' || e.pack === p) && (c === 'all' || e.cat === c)) pass.add(e.key)
      }
      return pass
    })

    // One zero-arg visibility read per tile — the only call shape a
    // renderable expression allows (`show.badge()`, never `isShown('badge')`).
    const show = Object.fromEntries(
      CATALOG.map((e) => [e.key, () => visible().has(e.key)]),
    ) as Record<string, () => boolean>

    const packs = () => PACKS.map((p) => ({ ...p, count: packCount(p.id) }))
    const cats = () => CATEGORIES.map((c) => c as string)
    // The headline number is the L1 surface — 48 elements, all live.
    const countLabel = derived(() => {
      const v = visible()
      const n = CATALOG.reduce((acc, e) => (v.has(e.key) ? acc + 1 : acc), 0)
      return n > 0 ? `${n} element${n === 1 ? '' : 's'} — all live` : ''
    })

    const none = derived(() => visible().size === 0)

    // Demo state the interactive tiles mutate — real signals, not props.
    const name = signal('')
    const agreed = signal(true)
    const notify = signal(true)
    const volume = signal(60)

    return {
      query, packSel, catSel, pickPack, pickCat,
      packs, cats, show, countLabel, none,
      name, agreed, notify, volume,
    }
})
</controller>

<scope>
  const tile = style.overflow(hidden)
  const tileCol = style.h(full)
  const preview = style.pad(3).minH(9).grow(1).borderBWidth(1).borderB(border-subtle).bg(surface-0)
  const tileMeta = style.padX(3).padY(2)
  const tileName = style.fontSize(sm).fontWeight(semibold).fontFamily(mono)
  const cat = style.fontSize(xs).fg(muted-foreground).fontFamily(mono)
  const mutedSm = style.fontSize(sm).fg(muted-foreground)
  const eyebrow = style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)
  const gridCol = style.grow(1).minW(0).basis(0)
  const rail = style.w(panel-min-w-md)
  const packBtn = style.radius(md).cursor(pointer).borderWidth(1).border(border).pad(0).overflow(hidden).bg(transparent).textAlign(start)
  const packLabel = style.fontSize(sm).grow(1)
  const packCount = style.fontSize(xs).fg(primary).fontFamily(mono)
  const railNote = style.fontSize(xs).fg(muted-foreground).leading(relaxed).padT(2)
  const search = style.maxW(panel-max-w-sm)
  const mutedXs = style.fontSize(xs).fg(muted-foreground)
</scope>

  <p sx={mutedSm}>
    Every tile is the running component, not a screenshot.
    The language's own elements need no import; the packs read the same tokens.
  </p>

  <row align="stretch" gap="4">
    <!-- ── Pack rail — a filter, not a hero ──────────────────────────── -->
    <column gap="1" sx={rail}>
      <p sx={eyebrow}>Packs</p>
      {#for p of packs() key={p.id}}
        <button sx={packBtn} on:click={() => pickPack(p.id)} aria-pressed={packSel() === p.id ? 'true' : 'false'}>
          <column gap="0" sx={tileMeta}>
            <row gap="2" align="center">
              <strong sx={packLabel}>{p.label}</strong>
              <span sx={packCount}>{p.count}</span>
            </row>
            {#if p.sub}
              <p sx={cat}>{p.sub}</p>
            {/if}
          </column>
        </button>
      {/for}
      <p sx={railNote}>
        Packs read the same tokens as the language — install one and it
        already matches your theme.
      </p>
    </column>

    <!-- ── Grid ──────────────────────────────────────────────────────── -->
    <!-- basis-0 keeps the fluid column from demanding its (large) content
         width, so the wrapping <row> floor keeps rail + grid on one line
         instead of stacking the grid below the rail. -->
    <column gap="3" sx={gridCol}>
      <row gap="2" align="center" justify="between">
        <row gap="2">
          {#for c of cats() key={c}}
            <button size="sm" tone="primary" variant={catSel() === c ? 'solid' : 'ghost'} on:click={() => pickCat(c)}>{c}</button>
          {/for}
        </row>
        <input placeholder="Search — try 'tab' or 'overlay'" bind:value={query} sx={search} />
      </row>

      <grid cols="4" gap="3">
        <!-- ══ Inputs ═══════════════════════════════════════════════════ -->
        {#if show.button()}
          <card padding="none" data-tile="button" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <row gap="2" align="center">
                  <button variant="solid" tone="primary" motion:press={{ scale: 'press' }}>Solid</button>
                  <button variant="outline" tone="primary">Outline</button>
                </row>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/button" sx={tileName}>button</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.input()}
          <card padding="none" data-tile="input" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <input placeholder="Type — it's real" bind:value={name} />
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/input" sx={tileName}>input</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.textarea()}
          <card padding="none" data-tile="textarea" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <textarea placeholder="A few lines…"></textarea>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/textarea" sx={tileName}>textarea</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.select()}
          <card padding="none" data-tile="select" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <select>
                  <option>Engineering</option>
                  <option>Design</option>
                  <option>Research</option>
                </select>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/select" sx={tileName}>select</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.combobox()}
          <card padding="none" data-tile="combobox" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <combobox placeholder="Search fruit — type">
                  <option value="apple">Apple</option>
                  <option value="banana">Banana</option>
                  <option value="cherry">Cherry</option>
                  <option value="kiwi">Kiwi</option>
                </combobox>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/combobox" sx={tileName}>combobox</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.checkbox()}
          <card padding="none" data-tile="checkbox" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <label><input type="checkbox" bind:checked={agreed} /> I agree</label>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/checkbox" sx={tileName}>checkbox</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.radioGroup()}
          <card padding="none" data-tile="radio-group" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <radio-group>
                  <label><input type="radio" name="plan" checked /> Monthly</label>
                  <label><input type="radio" name="plan" /> Yearly</label>
                </radio-group>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/radio-group" sx={tileName}>radio-group</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.switchToggle()}
          <card padding="none" data-tile="switch" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <row gap="2" align="center">
                  <switch bind:checked={notify}></switch>
                  <p sx={mutedXs}>{notify() ? 'notifying' : 'muted'}</p>
                </row>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/switch" sx={tileName}>switch</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.slider()}
          <card padding="none" data-tile="slider" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <row gap="2" align="center" sx={style.w(full)}>
                  <slider bind:value={volume} tone="primary" sx={style.grow(1)}></slider>
                  <span sx={cat}>{volume()}</span>
                </row>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/slider" sx={tileName}>slider</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.toggleGroup()}
          <card padding="none" data-tile="toggle-group" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <toggle-group>
                  <toggle value="left">Left</toggle>
                  <toggle value="center">Center</toggle>
                  <toggle value="right">Right</toggle>
                </toggle-group>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/toggle-group" sx={tileName}>toggle-group</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.field()}
          <card padding="none" data-tile="field" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <field>
                  <label for="tile-name">Name</label>
                  <input id="tile-name" placeholder="Ada Lovelace" />
                </field>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/field" sx={tileName}>field</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.label()}
          <card padding="none" data-tile="label" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview gap="1" sx={preview}>
                <label for="tile-email" required>Email</label>
                <input id="tile-email" placeholder="ada@lovelace.dev" />
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/label" sx={tileName}>label</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.fieldset()}
          <card padding="none" data-tile="fieldset" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <fieldset>
                  <legend>Account</legend>
                  <input placeholder="Name" sx={style.w(full)} />
                </fieldset>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/fieldset" sx={tileName}>fieldset</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.dropzone()}
          <card padding="none" data-tile="dropzone" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <card dropzone sx={style.pad(3)}>Drop a file, or click</card>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/dropzone" sx={tileName}>dropzone</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.themePicker()}
          <card padding="none" data-tile="theme-picker" sx={tile}>
            <column gap="0" sx={tileCol}>
              <!-- Do NOT mount a second live <theme-picker> here: hard-coded
                   panel id + full 9-theme previews next to the header instance
                   duplicates popover targets and was crashing this page. -->
              <center data-preview sx={preview}>
                <p sx={mutedXs}>Live control is in the site header (top-right).</p>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/theme-picker" sx={tileName}>theme-picker</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}

        <!-- ══ Feedback ═════════════════════════════════════════════════ -->
        {#if show.alert()}
          <card padding="none" data-tile="alert" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <alert tone="success"><strong>Saved.</strong> Changes are live.</alert>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/alert" sx={tileName}>alert</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>feedback</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.progressBar()}
          <card padding="none" data-tile="progress" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview gap="1" sx={preview}>
                <progress value={volume()} max="100"></progress>
                <p sx={mutedXs}>driven by the slider tile — one signal</p>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/progress" sx={tileName}>progress</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>feedback</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.meter()}
          <card padding="none" data-tile="meter" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <meter value="0.7"></meter>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/meter" sx={tileName}>meter</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>feedback</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.spinner()}
          <card padding="none" data-tile="spinner" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <row gap="3" align="center">
                  <spinner></spinner>
                  <spinner tone="primary"></spinner>
                  <spinner size="lg"></spinner>
                </row>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/spinner" sx={tileName}>spinner</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>feedback</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.skeleton()}
          <card padding="none" data-tile="skeleton" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <row gap="2" align="center" sx={style.w(full)}>
                  <span skeleton shape="circle" sx={style.w(7).h(7)}></span>
                  <column gap="1" sx={style.grow(1)}>
                    <span skeleton shape="text" size="sm"></span>
                    <span skeleton shape="text" size="sm"></span>
                  </column>
                </row>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/skeleton" sx={tileName}>skeleton</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>feedback</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.emptyState()}
          <card padding="none" data-tile="empty-state" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <empty-state variant="inline" size="sm">
                  <strong slot="title">No results</strong>
                  Try a different search.
                </empty-state>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/empty-state" sx={tileName}>empty-state</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>feedback</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.errorState()}
          <card padding="none" data-tile="error-state" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <error-state variant="inline" size="sm" code="ERR_TIMEOUT">The request timed out.</error-state>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/error-state" sx={tileName}>error-state</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>feedback</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.errorBoundary()}
          <card padding="none" data-tile="error-boundary" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview gap="1" sx={preview}>
                <error-boundary message="This panel could not load.">
                  <card variant="outline" sx={style.pad(2)}>
                    <p sx={mutedXs}>wrapped subtree — healthy, so the boundary is invisible</p>
                  </card>
                </error-boundary>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/error-boundary" sx={tileName}>error-boundary</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>feedback</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.toast()}
          <card padding="none" data-tile="toast" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <toast tone="success" duration="0">
                  <strong>Saved.</strong>
                  <button data-toast-close variant="ghost" size="sm">×</button>
                </toast>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/toast" sx={tileName}>toast</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>feedback</span>
                </row>
              </column>
            </column>
          </card>
        {/if}

        <!-- ══ Display ══════════════════════════════════════════════════ -->
        {#if show.badge()}
          <card padding="none" data-tile="badge" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <row gap="2" align="center">
                  <badge tone="primary">Primary</badge>
                  <badge tone="success">Active</badge>
                  <badge tone="warning" variant="outline">Pending</badge>
                </row>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/badge" sx={tileName}>badge</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>display</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.chip()}
          <card padding="none" data-tile="chip" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <row gap="2" align="center">
                  <chip selected>Selected</chip>
                  <chip removable>Removable</chip>
                </row>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/chip" sx={tileName}>chip</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>display</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.statusDot()}
          <card padding="none" data-tile="status-dot" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <row gap="2" align="center">
                  <status-dot tone="success"></status-dot>
                  <p sx={mutedXs}>all systems live</p>
                </row>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/status-dot" sx={tileName}>status-dot</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>display</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.card()}
          <card padding="none" data-tile="card" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <card interactive sx={style.pad(3)}>
                  <strong>A surface</strong>
                  <p sx={mutedXs}>the hover lift is one attribute</p>
                </card>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/card" sx={tileName}>card</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>display</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.avatar()}
          <card padding="none" data-tile="avatar" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <row gap="2" align="center">
                  <avatar name="Ada Lovelace"></avatar>
                  <avatar initials="GH" tone="primary"></avatar>
                  <avatar name="Alan Turing" size="lg" shape="square"></avatar>
                </row>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/avatar" sx={tileName}>avatar</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>display</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.icon()}
          <card padding="none" data-tile="icon" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <row gap="3" align="center">
                  <icon>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5" /></svg>
                  </icon>
                  <icon size="lg">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="11" cy="11" r="7" /><path d="m21 21-4.3-4.3" /></svg>
                  </icon>
                  <icon spin>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 12a9 9 0 1 1-6.2-8.56" /></svg>
                  </icon>
                </row>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/icon" sx={tileName}>icon</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>display</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.separator()}
          <card padding="none" data-tile="separator" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview gap="1" sx={preview}>
                <p sx={mutedXs}>above</p>
                <separator></separator>
                <p sx={mutedXs}>below</p>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/separator" sx={tileName}>separator</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>display</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.hrRule()}
          <card padding="none" data-tile="hr" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview gap="2" sx={preview}>
                <hr />
                <hr tone="primary" />
                <hr variant="dashed" />
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/hr" sx={tileName}>hr</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>display</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.detailsEl()}
          <card padding="none" data-tile="details" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <details>
                  <summary>Advanced options</summary>
                  <p sx={style.fontSize(xs)}>Native disclosure. Zero JavaScript.</p>
                </details>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/details" sx={tileName}>details</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>display</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.collapsible()}
          <card padding="none" data-tile="collapsible" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <collapsible>
                  <strong slot="trigger">Collapsible</strong>
                  <p sx={style.fontSize(xs)}>Animated, and it remembers.</p>
                </collapsible>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/collapsible" sx={tileName}>collapsible</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>display</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.prose()}
          <card padding="none" data-tile="prose" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <p sx={style.fontSize(xs)}>
                  <strong>strong</strong>, <em>emphasis</em>, <code>inline_code()</code>,
                  <mark>a highlight</mark> — plain tags, already styled.
                </p>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/prose" sx={tileName}>prose</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>display</span>
                </row>
              </column>
            </column>
          </card>
        {/if}

        <!-- ══ Overlay — the tile shows the TRIGGER; the panel rides the
             top layer via popovertarget, so a closed tile is honest ═════ -->
        {#if show.tooltip()}
          <card padding="none" data-tile="tooltip" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <tooltip content="Deletes the branch — cannot be undone">
                  <button variant="outline" tone="destructive">Delete branch</button>
                </tooltip>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/tooltip" sx={tileName}>tooltip</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>overlay</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.hoverCard()}
          <card padding="none" data-tile="hover-card" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <button variant="outline" popovertarget="tile-hc">@ada</button>
                <hover-card id="tile-hc">
                  <card sx={style.pad(3)}>Richer content on a native popover.</card>
                </hover-card>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/hover-card" sx={tileName}>hover-card</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>overlay</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.popover()}
          <card padding="none" data-tile="popover" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <button popovertarget="tile-pop">Open popover</button>
                <popover id="tile-pop">
                  <card sx={style.pad(3)}>Rides the top layer — no z-index war.</card>
                </popover>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/popover" sx={tileName}>popover</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>overlay</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.dropdown()}
          <card padding="none" data-tile="dropdown" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <button popovertarget="tile-menu">Menu ▾</button>
                <dropdown id="tile-menu">
                  <menu-item>Edit</menu-item>
                  <menu-item>Duplicate</menu-item>
                  <menu-item tone="destructive">Delete</menu-item>
                </dropdown>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/dropdown" sx={tileName}>dropdown</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>overlay</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.contextMenu()}
          <card padding="none" data-tile="context-menu" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <card id="tile-ctx" variant="outline" sx={style.pad(3)}>
                  <p sx={mutedXs}>right-click target</p>
                </card>
                <context-menu for="tile-ctx">
                  <menu-item>Cut</menu-item>
                  <menu-item>Copy</menu-item>
                  <menu-item>Paste</menu-item>
                </context-menu>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/context-menu" sx={tileName}>context-menu</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>overlay</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.dialog()}
          <card padding="none" data-tile="dialog" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <button command="show-modal" commandfor="tile-dialog" motion:press={{ scale: 'press' }}>Open modal</button>
                <dialog id="tile-dialog">
                  <header>A real dialog</header>
                  <section>Top layer, focus trap, Escape — all platform.</section>
                  <footer><button command="close" commandfor="tile-dialog">Close</button></footer>
                </dialog>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/dialog" sx={tileName}>dialog</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>overlay</span>
                </row>
              </column>
            </column>
          </card>
        {/if}

        <!-- ══ Navigation ═══════════════════════════════════════════════ -->
        {#if show.tabs()}
          <card padding="none" data-tile="tabs" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <tabs size="sm">
                  <tab-list>
                    <tab>Profile</tab>
                    <tab>Billing</tab>
                  </tab-list>
                  <tab-panel><p sx={mutedXs}>Profile settings.</p></tab-panel>
                  <tab-panel><p sx={mutedXs}>Billing settings.</p></tab-panel>
                </tabs>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/tabs" sx={tileName}>tabs</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>navigation</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.breadcrumb()}
          <card padding="none" data-tile="breadcrumb" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <breadcrumb>
                  <a href="/">Home</a>
                  <a href="/components">Components</a>
                  <small>Here</small>
                </breadcrumb>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/breadcrumb" sx={tileName}>breadcrumb</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>navigation</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.pagination()}
          <card padding="none" data-tile="pagination" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <pagination value="2" size="sm">
                  <page value="1" aria-label="Previous">‹</page>
                  <page value="1">1</page>
                  <page value="2">2</page>
                  <page value="3">3</page>
                  <page value="3" aria-label="Next">›</page>
                </pagination>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/pagination" sx={tileName}>pagination</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>navigation</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.navbar()}
          <card padding="none" data-tile="navbar" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <navbar bordered>
                  <strong slot="brand" sx={style.fontSize(sm)}>SHIP</strong>
                  <a href="/components" sx={style.fontSize(xs)}>Docs</a>
                  <a href="/components" sx={style.fontSize(xs)}>API</a>
                </navbar>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/navbar" sx={tileName}>navbar</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>navigation</span>
                </row>
              </column>
            </column>
          </card>
        {/if}

        <!-- ══ Data ═════════════════════════════════════════════════════ -->
        {#if show.table()}
          <card padding="none" data-tile="table" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={style.pad(3).minH(9).grow(1).borderBWidth(1).borderB(border-subtle).bg(surface-0).overflowX(auto)}>
                <table>
                  <thead>
                    <tr><th>Name</th><th>Status</th></tr>
                  </thead>
                  <tbody>
                    <tr><td>Ada Lovelace</td><td><badge tone="success" size="sm">Active</badge></td></tr>
                    <tr><td>Grace Hopper</td><td><badge tone="muted" size="sm">Away</badge></td></tr>
                  </tbody>
                </table>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/table" sx={tileName}>table</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>data</span>
                </row>
              </column>
            </column>
          </card>
        {/if}

        <!-- ══ Layout ═══════════════════════════════════════════════════ -->
        {#if show.layout()}
          <card padding="none" data-tile="layout" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview gap="2" sx={preview}>
                <row gap="2">
                  <div sx={style.grow(1).h(5).bg(muted).radius(sm)}></div>
                  <div sx={style.grow(1).h(5).bg(muted).radius(sm)}></div>
                  <div sx={style.grow(1).h(5).bg(muted).radius(sm)}></div>
                </row>
                <grid cols="2" gap="2">
                  <div sx={style.h(5).bg(muted).radius(sm)}></div>
                  <div sx={style.h(5).bg(muted).radius(sm)}></div>
                </grid>
                <p sx={mutedXs}>row · column · grid · center — keywords, not components</p>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/layout" sx={tileName}>layout</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>layout</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.scrollArea()}
          <card padding="none" data-tile="scroll-area" sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <scroll-area sx={style.h(9).w(full).padR(2)}>
                  <p sx={mutedXs}>line one</p>
                  <p sx={mutedXs}>line two</p>
                  <p sx={mutedXs}>line three</p>
                  <p sx={mutedXs}>line four</p>
                  <p sx={mutedXs}>line five</p>
                </scroll-area>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/scroll-area" sx={tileName}>scroll-area</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">language</badge>
                  <span sx={cat}>layout</span>
                </row>
              </column>
            </column>
          </card>
        {/if}

        <!-- ══ @ship/sail-charts ═════════════════════════════════════════ -->
        {#if show.sailSparkline()}
          <card padding="none" data-tile="sail-sparkline" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <sail-sparkline data="2,5,3,8,4,6,7" width="120" height="40"></sail-sparkline>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/sail-sparkline" sx={tileName}>sail-sparkline</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="accent" variant="soft">charts</badge>
                  <span sx={cat}>data</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.sailAreaChart()}
          <card padding="none" data-tile="sail-area-chart" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <sail-area-chart series='[{"name":"a","data":[2,4,3,5]}]' labels="Q1,Q2,Q3,Q4" width="160" height="60"></sail-area-chart>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/sail-area-chart" sx={tileName}>sail-area-chart</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="accent" variant="soft">charts</badge>
                  <span sx={cat}>data</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.sailBarChart()}
          <card padding="none" data-tile="sail-bar-chart" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <sail-bar-chart data="2,5,3,8,4" labels="A,B,C,D,E" width="160" height="60"></sail-bar-chart>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/sail-bar-chart" sx={tileName}>sail-bar-chart</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="accent" variant="soft">charts</badge>
                  <span sx={cat}>data</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.sailDonut()}
          <card padding="none" data-tile="sail-donut" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <sail-donut data='[{"label":"a","value":40},{"label":"b","value":30},{"label":"c","value":30}]' diameter="80" thickness="10"></sail-donut>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/sail-donut" sx={tileName}>sail-donut</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="accent" variant="soft">charts</badge>
                  <span sx={cat}>data</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.sailGauge()}
          <card padding="none" data-tile="sail-gauge" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <sail-gauge value="72" diameter="80" thickness="10"></sail-gauge>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/sail-gauge" sx={tileName}>sail-gauge</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="accent" variant="soft">charts</badge>
                  <span sx={cat}>data</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.sailHeatmap()}
          <card padding="none" data-tile="sail-heatmap" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <sail-heatmap rows="3" cols="5" values="1,2,3,4,3,4,5,6,5,4,3,2,1,2,3" width="160" height="60"></sail-heatmap>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/sail-heatmap" sx={tileName}>sail-heatmap</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="accent" variant="soft">charts</badge>
                  <span sx={cat}>data</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.sailLineChart()}
          <card padding="none" data-tile="sail-line-chart" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <sail-line-chart series='[{"name":"a","data":[2,4,3,5]}]' labels="Q1,Q2,Q3,Q4" width="160" height="60"></sail-line-chart>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/sail-line-chart" sx={tileName}>sail-line-chart</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="accent" variant="soft">charts</badge>
                  <span sx={cat}>data</span>
                </row>
              </column>
            </column>
          </card>
        {/if}

        <!-- ══ @ship/sail-media ══════════════════════════════════════════ -->
        {#if show.sailMediaImage()}
          <card padding="none" data-tile="sail-media-image" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <sail-media-image src="/sail-oss/sail-mark.svg" alt="Sail mark"></sail-media-image>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/sail-media-image" sx={tileName}>sail-media-image</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="info" variant="soft">media</badge>
                  <span sx={cat}>data</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.sailMediaAv()}
          <card padding="none" data-tile="sail-media-av" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <sail-media-av></sail-media-av>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/sail-media-av" sx={tileName}>sail-media-av</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="info" variant="soft">media</badge>
                  <span sx={cat}>data</span>
                </row>
              </column>
            </column>
          </card>
        {/if}

        <!-- ══ @ship/sail-code-editor + @ship/sail-mermaid ═══════════════════ -->
        {#if show.sailCodeEditor()}
          <card padding="none" data-tile="sail-code-editor" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <sail-code-editor value="const greeting = 'hi';" language="js" line-numbers size="sm"></sail-code-editor>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/sail-code-editor" sx={tileName}>sail-code-editor</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="muted" variant="soft">code-editor</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.sailMermaid()}
          <card padding="none" data-tile="sail-mermaid" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <sail-mermaid value="graph TD; A[Idea]-->B[Build]; B-->C[Ship]"></sail-mermaid>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/sail-mermaid" sx={tileName}>sail-mermaid</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="success" variant="soft">mermaid</badge>
                  <span sx={cat}>data</span>
                </row>
              </column>
            </column>
          </card>
        {/if}

        <!-- ══ @ship/sail-chat-elements — sail-message-list deferred (see banner) ═ -->
        {#if show.sailMessage()}
          <card padding="none" data-tile="sail-message" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <sail-message role="user" state="user">Hello — the live chat message primitive.</sail-message>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/sail-message" sx={tileName}>sail-message</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">chat</badge>
                  <span sx={cat}>display</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.sailCitation()}
          <card padding="none" data-tile="sail-citation" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <sail-citation source="docs.sail.dev" href="https://docs.sail.dev">A citation from the docs</sail-citation>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/sail-citation" sx={tileName}>sail-citation</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">chat</badge>
                  <span sx={cat}>display</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.sailComposer()}
          <card padding="none" data-tile="sail-composer" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <sail-composer placeholder="Ask anything…"></sail-composer>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/sail-composer" sx={tileName}>sail-composer</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">chat</badge>
                  <span sx={cat}>inputs</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.sailThinkingDots()}
          <card padding="none" data-tile="sail-thinking-dots" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <sail-thinking-dots></sail-thinking-dots>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/sail-thinking-dots" sx={tileName}>sail-thinking-dots</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">chat</badge>
                  <span sx={cat}>feedback</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
        {#if show.sailToolCall()}
          <card padding="none" data-tile="sail-tool-call" sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <sail-tool-call name="search" status="running">query: search</sail-tool-call>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/sail-tool-call" sx={tileName}>sail-tool-call</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone="primary" variant="soft">chat</badge>
                  <span sx={cat}>display</span>
                </row>
              </column>
            </column>
          </card>
        {/if}
      </grid>

      {#if none()}
        <empty-state>
          <strong slot="title">Nothing matches</strong>
          Clear a filter or two — or search for less.
        </empty-state>
      {/if}
    </column>
  </row>

  <row sx={style.mt(8).padT(4).borderTWidth(1).borderT(border)} gap="2">
    <a variant="default" href="/sail/overview">← Introduction</a>
    <a variant="solid" tone="primary" href="/sail/language/layout">Layout →</a>
  </row>
</column>
