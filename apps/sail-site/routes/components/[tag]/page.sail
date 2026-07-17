<!--
  /components/[tag] — the detail view. The live component holds center stage
  (press it, type in it, pick from it); the variant chips drive a REAL
  attribute on it; usage is a complete compiling program; props are the
  curated truth from the stdlib declarations.

  route.params.tag() picks the entry; ./registry.ts covers the FULL L1
  surface (all 48 elements), so the empty state below is for typos, not
  for real components. Each stage mounts through one zero-arg gate
  (`stage.<key>()`) — the only call shape a renderable expression
  allows.
-->

<controller>
// /components/[tag] — the detail view. route.params.tag() picks the entry
// out of the DETAILS registry (the full L1 surface); one variant signal
// drives the live stage (each entry maps its chips onto whichever
// attribute its variants mean — button:variant, slider:tone, switch:size,
// tooltip:placement).
//
// Every read the template does is a zero-arg call (`entry()`, `stage.button()`),
// because that is the only call shape a renderable expression allows — the
// stage map mirrors the browser page's `show` map.

import { controller, signal, derived, effect, type ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { DETAILS, type DetailEntry } from './registry'
export default controller((ctx: ViewScope<Scope>) => {
    const tag = derived(() => ctx.route.params.tag?.() ?? '')
    const entry = derived<DetailEntry | null>(() => DETAILS[tag()] ?? null)
    const known = derived(() => entry() !== null)
    // Pack-component tags don't have DETAILS entries (that registry is the
    // L1 surface only) — mark them known too so this route mounts them
    // instead of reporting a typo. Includes the 7 canvas-pack tags, which
    // page.sail skips in its own grid but which still render fully here via
    // direct navigation. sail-message-list and sail-media-csv are NOT here:
    // both transitively import `@ship/sail-runtime/virtual` — message-list
    // directly, media-csv via `createTable` from the `@ship/sail-primitives`
    // barrel, whose index.ts side-effect-imports sail-tree/sail-tree-grid — a
    // framework-singleton subpath with no deck-bundler importmap entry
    // (apps/sail-site/src/register-all.ts). Pre-existing bundler gap, not a
    // floor issue. Both still count in the registry catalog, but no
    // template may reference either tag anywhere until that importmap gap
    // is closed (separate job), or the app build fails outright.
    const PACK_TAGS = new Set([
      'sail-sparkline', 'sail-area-chart', 'sail-bar-chart', 'sail-donut', 'sail-gauge',
      'sail-heatmap', 'sail-line-chart', 'sail-media-image', 'sail-media-av',
      'sail-code-editor', 'sail-mermaid', 'sail-message',
      'sail-citation', 'sail-composer', 'sail-thinking-dots',
      'sail-tool-call', 'sail-canvas', 'sail-board', 'sail-draw', 'sail-graph-canvas',
      'sail-annotation-layer', 'sail-annotation-list', 'sail-annotation-marker',
    ])
    const knownPack = derived(() => PACK_TAGS.has(tag()))
    const showViewer = derived(() => known() || knownPack())

    const variant = signal('')
    // A navigation between two detail pages reuses this scope — re-seed the
    // variant whenever the tag changes.
    effect(() => {
      variant.set(entry()?.variants[0] ?? '')
    })
    const pick = (v: string): void => variant.set(v)
    // The shared viewer owns the chips; the page keeps its own variant signal
    // because every specimen stage reads it.
    const onVariantChange = (e: CustomEvent<{ variant: string }>): void =>
      variant.set(e.detail?.variant ?? '')

    const variants = () => entry()?.variants ?? []
    const hasVariants = derived(() => (entry()?.variants.length ?? 0) > 0)
    const variantAttr = () => entry()?.variantAttr ?? ''
    const blurb = () => entry()?.blurb ?? ''
    const props = () => entry()?.props ?? []
    const usageCode = () => entry()?.usage ?? ''
    // Slot tagline — the page subtitle under the specimen. For pack
    // components the L1 "nothing here is a screenshot" copy undersells the
    // point (the pack IS the live component) and some packs have their own
    // interaction model (world layer, code editor, mermaid SVG).
    const slotCopy = (() => {
      if (!knownPack()) return 'click it · type in it · pick from it — nothing here is a screenshot'
      const t = tag()
      if (t === 'sail-canvas' || t === 'sail-board' || t === 'sail-draw' || t === 'sail-graph-canvas' || t.startsWith('sail-annotation-')) {
        return 'use the world layer — pan/zoom · drop annotations on click'
      }
      if (t === 'sail-code-editor') return 'click into the editor · edit and watch the value'
      if (t === 'sail-mermaid') return 'mermaid rendered the diagram · pan the SVG to inspect'
      return 'interact with the live component — type, click, toggle'
    })()
    const heading = () => entry()?.heading ?? tag()

    // Pack blurb — packs without a DETAILS entry get a hand-written
    // one-liner keyed off the tag.
    const packBlurb = () => {
      const t = tag()
      if (t === 'sail-area-chart') return 'Multi-series area chart with stacked fills. Series JSON + label list.'
      if (t === 'sail-bar-chart') return 'Vertical bars, one per label. CSV data string + label list.'
      if (t === 'sail-donut') return 'Stroked-arc segments, sized by value. JSON label/value/tone.'
      if (t === 'sail-gauge') return 'Single-arc gauge, value 0..max. Auto-centers the indicator.'
      if (t === 'sail-heatmap') return 'Grid of cells, colored by value. rows × cols × values.'
      if (t === 'sail-line-chart') return 'Multi-series line chart with markers and grid. Series JSON + label list.'
      if (t === 'sail-sparkline') return 'Compact single-line trend. CSV data, width × height.'
      if (t === 'sail-media-image') return 'Image primitive with src/alt + framework theming.'
      if (t === 'sail-media-av') return 'Audio/video primitive with platform controls + framework skin.'
      if (t === 'sail-code-editor') return 'CodeMirror 6 wrapped as a primitive; controlled value + language prop.'
      if (t === 'sail-mermaid') return 'Mermaid-rendered SVG; pass the source as the value prop.'
      if (t === 'sail-message') return 'A single chat message; role, state, content, timestamp props.'
      if (t === 'sail-citation') return 'Citation pill with source/label; click to follow href.'
      if (t === 'sail-composer') return 'Chat-composer input with submit + cancel events.'
      if (t === 'sail-thinking-dots') return 'Three bouncing dots with polite live status.'
      if (t === 'sail-tool-call') return 'Tool-call disclosure with name + status (running|ok|error).'
      if (t === 'sail-canvas') return 'The pannable/zoomable world layer canvas-pack components mount on.'
      if (t === 'sail-board') return 'A bounded canvas surface hosting a world layer.'
      if (t === 'sail-draw') return 'Freehand drawing surface over the world layer.'
      if (t === 'sail-graph-canvas') return 'Node/edge graph canvas; JSON nodes + edges.'
      if (t === 'sail-annotation-layer') return 'Click-to-annotate overlay for the world layer.'
      if (t === 'sail-annotation-list') return 'List view of the annotations dropped on a layer.'
      if (t === 'sail-annotation-marker') return 'A single point annotation marker at x/y.'
      return 'Pack component.'
    }
    // Pack props table — hand-curated per the pack's authored prop surface.
    const PACK_PROPS: Record<string, { name: string; type: string; desc: string }[]> = {
      'sail-sparkline':   [{ name: 'data',   type: 'string',  desc: 'CSV numbers, e.g. "2,5,3,8,4,6,7".' }, { name: 'width',  type: 'number',  desc: 'SVG width in px.' }, { name: 'height', type: 'number',  desc: 'SVG height in px.' }],
      'sail-area-chart':  [{ name: 'series', type: 'string (JSON)', desc: '[{name, data, tone?}] — multi-series.' }, { name: 'labels', type: 'string (CSV)', desc: 'X-axis labels.' }],
      'sail-bar-chart':   [{ name: 'data',   type: 'string (CSV)', desc: 'Bar values, e.g. "2,5,3,8,4".' }, { name: 'labels', type: 'string (CSV)', desc: 'Bar labels.' }],
      'sail-donut':       [{ name: 'data',   type: 'string (JSON)', desc: '[{label,value,tone?}] segments.' }, { name: 'diameter', type: 'number', desc: 'Outer diameter in px.' }],
      'sail-gauge':       [{ name: 'value',  type: 'number', desc: 'Indicator position (0..max).' }, { name: 'diameter', type: 'number', desc: 'Outer diameter in px.' }],
      'sail-heatmap':     [{ name: 'rows',  type: 'number', desc: 'Cell rows.' }, { name: 'cols', type: 'number', desc: 'Cell columns.' }, { name: 'values', type: 'string (CSV)', desc: 'Row-major cell values.' }],
      'sail-line-chart':  [{ name: 'series', type: 'string (JSON)', desc: '[{name, data, tone?}].' }, { name: 'labels', type: 'string (CSV)', desc: 'X-axis labels.' }],
      'sail-media-image': [{ name: 'src',   type: 'string', desc: 'Image URL.' }, { name: 'alt',  type: 'string', desc: 'Alt text.' }],
      'sail-media-av':    [{ name: 'src',   type: 'string', desc: 'Audio/video URL.' }, { name: 'mime', type: 'string', desc: 'Override mime type.' }],
      'sail-code-editor': [{ name: 'value', type: 'string', desc: 'Editor text.' }, { name: 'language', type: 'string', desc: 'CodeMirror language.' }, { name: 'line-numbers', type: 'bool', desc: 'Show gutter.' }],
      'sail-mermaid':     [{ name: 'value', type: 'string', desc: 'Mermaid source text.' }],
      'sail-message':     [{ name: 'role',  type: 'string', desc: 'Message author role.' }, { name: 'state', type: 'string', desc: 'Message state (user|assistant|...).' }, { name: 'content', type: 'string', desc: 'Body text.' }],
      'sail-citation':    [{ name: 'source', type: 'string', desc: 'Citation source label.' }, { name: 'href',  type: 'string', desc: 'Click target.' }],
      'sail-composer':    [{ name: 'placeholder', type: 'string', desc: 'Empty-state copy.' }],
      'sail-thinking-dots': [{ name: 'aria-label', type: 'string', desc: 'Live-region label.' }],
      'sail-tool-call':   [{ name: 'name',   type: 'string', desc: 'Tool name.' }, { name: 'status', type: 'string', desc: 'running|ok|error.' }],
      'sail-canvas':      [{ name: 'width',  type: 'number', desc: 'Viewport width in px.' }, { name: 'height', type: 'number', desc: 'Viewport height in px.' }],
      'sail-graph-canvas': [{ name: 'nodes', type: 'string (JSON)', desc: '[{id, kind, title, x, y}].' }, { name: 'edges', type: 'string (JSON)', desc: '[{from, to}].' }],
      'sail-annotation-marker': [{ name: 'x', type: 'number', desc: 'X position.' }, { name: 'y', type: 'number', desc: 'Y position.' }],
    }
    const packProps = () => PACK_PROPS[tag()] ?? []

    // One zero-arg gate per stage — `stage.<key>()`, derived from the entry.
    // Pack components (the 6 un-gated packs, 25 components) are added here
    // as a side channel — they don't have DETAILS entries, so their
    // `entry()` is null; we gate on tag() (kebab-case) directly.
    const stage = Object.fromEntries([
      ...Object.values(DETAILS).map((e) => [e.key, () => entry()?.key === e.key]),
      ...[...PACK_TAGS].map((k) => [k.replace(/-([a-z])/g, (_, c) => c.toUpperCase()), () => tag() === k]),
    ]) as Record<string, () => boolean>

    // Stage state — real signals the stages mutate.
    const presses = signal(0)
    const press = (): void => presses.set((n) => n + 1)
    const notify = signal(true)
    const agreed = signal(false)
    const load = signal(62)
    const name = signal('')
    const bio = signal('')
    const team = signal('Design')
    const fw = signal('')
    const pickFw = (e: CustomEvent<{ value: string }>): void => fw.set(e.detail?.value ?? '')
    const pageNo = signal('2')
    const goPage = (e: CustomEvent<{ value: string }>): void => pageNo.set(e.detail?.value ?? '1')
    const kept = signal(true)
    const dropChip = (): void => kept.set(false)
    const restoreChip = (): void => kept.set(true)
    const sheetSide = () => (variant() === 'sheet' ? 'end' : null)
    const people = () => [
      { id: 1, name: 'Ada Lovelace', team: 'Engines', status: 'active' },
      { id: 2, name: 'Grace Hopper', team: 'Compilers', status: 'active' },
      { id: 3, name: 'Alan Turing', team: 'Research', status: 'away' },
    ]

    return {
      tag, entry, known, knownPack, showViewer, heading,
      variant, pick, onVariantChange, variants, hasVariants, variantAttr,
      blurb, props, packBlurb, packProps, usageCode, stage, slotCopy,
      presses, press, notify, agreed, load, name, bio, team,
      fw, pickFw, pageNo, goPage, kept, dropChip, restoreChip,
      sheetSide, people,
    }
})
</controller>

<column sx="max-w-breakpoint-tablet mx-auto" gap="4">
  {#if showViewer()}
    <breadcrumb>
      <a href="/components">components</a>
      <small>{tag()}</small>
    </breadcrumb>

    <sail-component-viewer
      name={heading()}
      blurb={knownPack() ? packBlurb() : blurb()}
      pack={knownPack() ? (
        tag().startsWith('sail-area-chart') || tag().startsWith('sail-bar-chart') || tag().startsWith('sail-line-chart') || tag() === 'sail-donut' || tag() === 'sail-gauge' || tag() === 'sail-heatmap' || tag() === 'sail-sparkline' ? 'Pack: charts' :
        tag().startsWith('sail-media-') ? 'Pack: media' :
        tag() === 'sail-canvas' || tag() === 'sail-board' || tag() === 'sail-draw' || tag() === 'sail-graph-canvas' || tag().startsWith('sail-annotation-') ? 'Pack: canvas' :
        tag().startsWith('sail-message') || tag() === 'sail-citation' || tag() === 'sail-composer' || tag() === 'sail-thinking-dots' || tag() === 'sail-tool-call' ? 'Pack: chat' :
        tag() === 'sail-mermaid' ? 'Pack: mermaid' :
        tag() === 'sail-code-editor' ? 'Pack: code-editor' : 'Pack'
      ) : 'In the language'}
      variant-attr={variantAttr()}
      usage={usageCode()}
      prop:variants={variants()}
      prop:rows={knownPack() ? packProps() : props()}
      on:sail-variant-change={onVariantChange}
    >
      <column slot="stage" gap="4" align="center">
            <!-- ══ Inputs ══════════════════════════════════════════════ -->
            {#if stage.button()}
              <column gap="3" align="center">
                <button variant={variant()} tone="primary" size="lg" on:click={press} motion:press={{ scale: 'press' }}>
                  Press me
                </button>
                <p sx="text-xs text-muted-foreground font-mono">motion:press fired ×{presses()}</p>
              </column>
            {/if}
            {#if stage.input()}
              <column gap="3" sx="w-full max-w-panel-max-w-sm">
                <input placeholder="Ada Lovelace" bind:value={name} />
                <p sx="text-xs text-muted-foreground font-mono">hello, {name() || 'stranger'}</p>
              </column>
            {/if}
            {#if stage.textarea()}
              <column gap="3" sx="w-full max-w-panel-max-w-sm">
                <textarea placeholder="Tell us about yourself" bind:value={bio}></textarea>
                <p sx="text-xs text-muted-foreground font-mono">{bio().length} characters</p>
              </column>
            {/if}
            {#if stage.select()}
              <column gap="3" sx="w-full max-w-panel-max-w-sm">
                <select bind:value={team}>
                  <option>Engineering</option>
                  <option>Design</option>
                  <option>Research</option>
                </select>
                <p sx="text-xs text-muted-foreground font-mono">team: {team()}</p>
              </column>
            {/if}
            {#if stage.combobox()}
              <column gap="3" sx="w-full max-w-panel-max-w-sm">
                <combobox placeholder="Pick a framework — type to filter" on:sail-combobox-select={(e) => pickFw(e)}>
                  <option value="sail">SAIL</option>
                  <option value="react">React</option>
                  <option value="svelte">Svelte</option>
                  <option value="solid">Solid</option>
                  <option value="vue">Vue</option>
                  <option value="qwik">Qwik</option>
                </combobox>
                <p sx="text-xs text-muted-foreground font-mono">picked: {fw() || 'nothing yet'}</p>
              </column>
            {/if}
            {#if stage.checkbox()}
              <column gap="3" align="center">
                <label><input type="checkbox" bind:checked={agreed} /> I agree to the terms</label>
                <p sx="text-xs text-muted-foreground font-mono">{agreed() ? 'thank you' : 'the platform checkbox, skinned'}</p>
              </column>
            {/if}
            {#if stage.radioGroup()}
              <radio-group label="Billing period">
                <label><input type="radio" name="stage-plan" checked /> Monthly</label>
                <label><input type="radio" name="stage-plan" /> Yearly</label>
              </radio-group>
            {/if}
            {#if stage.switchToggle()}
              <row gap="3" align="center">
                <switch bind:checked={notify} size={variant()} tone="primary"></switch>
                <p sx="text-xs text-muted-foreground font-mono">{notify() ? 'on' : 'off'} · size {variant()}</p>
              </row>
            {/if}
            {#if stage.slider()}
              <column gap="3" sx="w-full max-w-panel-max-w-sm">
                <slider bind:value={load} min="0" max="100" tone={variant()}></slider>
                <progress value={load()} max="100"></progress>
                <p sx="text-xs text-muted-foreground font-mono">one signal, two components — {load()}%</p>
              </column>
            {/if}
            {#if stage.toggleGroup()}
              <toggle-group variant={variant()}>
                <toggle value="left">Left</toggle>
                <toggle value="center">Center</toggle>
                <toggle value="right">Right</toggle>
              </toggle-group>
            {/if}
            {#if stage.field()}
              <column gap="4" sx="w-full max-w-panel-max-w-sm">
                <field>
                  <label for="stage-name">Name</label>
                  <input id="stage-name" placeholder="Ada Lovelace" />
                </field>
                <field invalid>
                  <label for="stage-bad-email">Email</label>
                  <input id="stage-bad-email" value="not-an-email" />
                </field>
              </column>
            {/if}
            {#if stage.label()}
              <column gap="1" sx="w-full max-w-panel-max-w-sm">
                <label for="stage-email" required size={variant()}>Email</label>
                <input id="stage-email" placeholder="ada@lovelace.dev" />
              </column>
            {/if}
            {#if stage.fieldset()}
              <fieldset sx="w-full max-w-panel-max-w-sm">
                <legend>Account details</legend>
                <column gap="2">
                  <input placeholder="Name" />
                  <input placeholder="Email" />
                </column>
              </fieldset>
            {/if}
            {#if stage.dropzone()}
              <card dropzone sx="p-6 w-full max-w-panel-max-w-sm">Drop a file, or click to browse</card>
            {/if}
            {#if stage.themePicker()}
              <column gap="3" align="center">
                <theme-picker></theme-picker>
                <p sx="text-xs text-muted-foreground font-mono">open it — it themes this whole site, live</p>
              </column>
            {/if}

            <!-- ══ Feedback ════════════════════════════════════════════ -->
            {#if stage.alert()}
              <alert tone={variant()} sx="w-full">
                <strong>Heads up.</strong> The tone chips above restyle me — colour and icon both.
              </alert>
            {/if}
            {#if stage.progressBar()}
              <column gap="3" sx="w-full max-w-panel-max-w-sm">
                <slider bind:value={load} min="0" max="100" tone="primary"></slider>
                <progress value={load()} max="100"></progress>
                <p sx="text-xs text-muted-foreground font-mono">drag the slider — same signal, {load()}%</p>
              </column>
            {/if}
            {#if stage.meter()}
              <!-- Fixed width: meter has no intrinsic width, and a percentage
                   can't resolve inside the content-sized <center> stage. -->
              <column gap="2" sx="w-panel-max-w-sm">
                <meter value="0.3"></meter>
                <meter value="0.7"></meter>
                <meter value="0.95"></meter>
              </column>
            {/if}
            {#if stage.spinner()}
              <row gap="4" align="center">
                <spinner size={variant()}></spinner>
                <spinner size={variant()} tone="primary"></spinner>
                <spinner size={variant()} tone="destructive"></spinner>
              </row>
            {/if}
            {#if stage.skeleton()}
              <row gap="3" align="center" sx="w-full max-w-panel-max-w-sm">
                <span skeleton shape={variant()} sx="w-7 h-7"></span>
                <column gap="2" sx="grow-1">
                  <span skeleton shape={variant()} size="sm"></span>
                  <span skeleton shape={variant()} size="sm"></span>
                </column>
              </row>
            {/if}
            {#if stage.emptyState()}
              <empty-state variant={variant()} sx="w-full">
                <strong slot="title">No results</strong>
                Try a different search, or clear a filter.
              </empty-state>
            {/if}
            {#if stage.errorState()}
              <error-state variant={variant()} code="ERR_TIMEOUT" sx="w-full">
                The request timed out. Retry, or check the connection.
              </error-state>
            {/if}
            {#if stage.errorBoundary()}
              <column gap="3" sx="w-full max-w-panel-max-w-sm">
                <error-boundary message="This panel could not load.">
                  <card variant="outline" sx="p-4">
                    <p sx="text-sm text-muted-foreground">
                      A healthy boundary is invisible — this card is the wrapped
                      subtree. A render throw inside would swap in the fallback.
                    </p>
                  </card>
                </error-boundary>
              </column>
            {/if}
            {#if stage.toast()}
              <toast tone={variant()} duration="0" sx="w-full max-w-panel-max-w-sm">
                <strong>Saved.</strong> Sticky at duration 0 — hover pauses real ones.
                <button data-toast-close variant="ghost" size="sm">×</button>
              </toast>
            {/if}

            <!-- ══ Display ═════════════════════════════════════════════ -->
            {#if stage.badge()}
              <row gap="2" align="center">
                <badge variant={variant()} tone="primary">primary</badge>
                <badge variant={variant()} tone="success">success</badge>
                <badge variant={variant()} tone="warning">warning</badge>
                <badge variant={variant()} tone="destructive">destructive</badge>
                <badge variant={variant()} tone="muted">muted</badge>
              </row>
            {/if}
            {#if stage.chip()}
              <column gap="3" align="center">
                <row gap="2" align="center">
                  <chip variant={variant()} selected>Selected</chip>
                  {#if kept()}
                    <chip variant={variant()} removable on:sail-remove={dropChip}>Removable</chip>
                  {/if}
                  {#if !kept()}
                    <button variant="ghost" size="sm" on:click={restoreChip}>bring it back</button>
                  {/if}
                </row>
                <p sx="text-xs text-muted-foreground font-mono">× or Backspace fires sail-remove</p>
              </column>
            {/if}
            {#if stage.statusDot()}
              <row gap="2" align="center">
                <status-dot tone={variant()}></status-dot>
                <p sx="text-sm text-muted-foreground">the tone is the message</p>
              </row>
            {/if}
            {#if stage.card()}
              <card variant={variant()} sx="p-5" motion:hover={{ scale: 'spring:gentle' }}>
                <strong>A surface</strong>
                <p sx="text-sm text-muted-foreground">hover me — the lift is one attribute</p>
              </card>
            {/if}
            {#if stage.avatar()}
              <row gap="3" align="center">
                <avatar name="Ada Lovelace" size={variant()}></avatar>
                <avatar initials="GH" tone="primary" size={variant()}></avatar>
                <avatar name="Alan Turing" size={variant()} shape="square"></avatar>
              </row>
            {/if}
            {#if stage.icon()}
              <row gap="4" align="center">
                <icon size={variant()}>
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5" /></svg>
                </icon>
                <icon size={variant()}>
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="11" cy="11" r="7" /><path d="m21 21-4.3-4.3" /></svg>
                </icon>
                <icon size={variant()} spin>
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 12a9 9 0 1 1-6.2-8.56" /></svg>
                </icon>
              </row>
            {/if}
            {#if stage.separator()}
              <column gap="2" sx="w-full max-w-panel-max-w-sm">
                <p sx="text-sm text-muted-foreground">above</p>
                <separator variant={variant()}></separator>
                <p sx="text-sm text-muted-foreground">below — and vertical:</p>
                <row gap="2" align="center">
                  <span>A</span>
                  <separator orientation="vertical" variant={variant()}></separator>
                  <span>B</span>
                </row>
              </column>
            {/if}
            {#if stage.hrRule()}
              <!-- Fixed width: an <hr> is a 1px line with no intrinsic width. -->
              <column gap="3" sx="w-panel-max-w-sm">
                <hr tone={variant()} />
                <hr tone={variant()} variant="dashed" />
              </column>
            {/if}
            {#if stage.detailsEl()}
              <details sx="w-full max-w-panel-max-w-sm">
                <summary>Advanced options</summary>
                <p>Native disclosure. Zero JavaScript — open/close is the platform.</p>
              </details>
            {/if}
            {#if stage.collapsible()}
              <collapsible sx="w-full max-w-panel-max-w-sm">
                <strong slot="trigger">Show the details</strong>
                <p>Animated open and close — and it remembers its state.</p>
              </collapsible>
            {/if}
            {#if stage.prose()}
              <column gap="2" sx="w-full max-w-panel-max-w-md">
                <h3>Plain tags, already styled</h3>
                <p>Body copy sets the baseline — <strong>strong</strong>, <em>emphasis</em>,
                   <code>inline_code()</code> and <mark>a highlight</mark> need no classes.</p>
                <blockquote>A quotation carries its own rule.</blockquote>
              </column>
            {/if}

            <!-- ══ Overlay ═════════════════════════════════════════════ -->
            {#if stage.tooltip()}
              <tooltip content="Deletes the branch — cannot be undone" placement={variant()}>
                <button tone="destructive" variant="outline">Delete branch</button>
              </tooltip>
            {/if}
            {#if stage.hoverCard()}
              <column gap="3" align="center">
                <button variant="ghost" popovertarget="stage-hc">@ada</button>
                <hover-card id="stage-hc">
                  <card sx="p-4">
                    <strong>Ada Lovelace</strong>
                    <p sx="text-sm text-muted-foreground">wrote the first program.</p>
                  </card>
                </hover-card>
                <p sx="text-xs text-muted-foreground font-mono">hover or focus the trigger</p>
              </column>
            {/if}
            {#if stage.popover()}
              <column gap="3" align="center">
                <button popovertarget="stage-pop">Open popover</button>
                <popover id="stage-pop">
                  <card sx="p-4">Rides the platform top layer — Escape or click outside closes it.</card>
                </popover>
                <p sx="text-xs text-muted-foreground font-mono">top layer · light dismiss · zero z-index</p>
              </column>
            {/if}
            {#if stage.dropdown()}
              <column gap="3" align="center">
                <button popovertarget="stage-menu">Actions ▾</button>
                <dropdown id="stage-menu">
                  <menu-item>Edit</menu-item>
                  <menu-item>Duplicate</menu-item>
                  <menu-item tone="destructive">Delete</menu-item>
                </dropdown>
                <p sx="text-xs text-muted-foreground font-mono">arrows rove · type-ahead · Escape closes</p>
              </column>
            {/if}
            {#if stage.contextMenu()}
              <column gap="3" align="center" sx="w-full max-w-panel-max-w-sm">
                <card id="stage-ctx" variant="outline" sx="p-6 w-full">
                  <center><p sx="text-sm text-muted-foreground">right-click this card</p></center>
                </card>
                <context-menu for="stage-ctx">
                  <menu-item value="cut">Cut</menu-item>
                  <menu-item value="copy">Copy</menu-item>
                  <menu-item value="paste">Paste</menu-item>
                </context-menu>
              </column>
            {/if}
            {#if stage.dialog()}
              <column gap="3" align="center">
                <button command="show-modal" commandfor="stage-dialog" motion:press={{ scale: 'press' }}>
                  Open the {variant()}
                </button>
                <p sx="text-xs text-muted-foreground font-mono">Escape and the backdrop close it — that's the platform</p>
                <dialog id="stage-dialog" side={sheetSide()}>
                  <header>Delete project</header>
                  <section>This cannot be undone. Top layer, focus trap, Escape — zero JavaScript.</section>
                  <footer>
                    <button command="close" commandfor="stage-dialog" variant="ghost">Cancel</button>
                    <button command="close" commandfor="stage-dialog" tone="destructive" variant="solid">Delete</button>
                  </footer>
                </dialog>
              </column>
            {/if}

            <!-- ══ Navigation ══════════════════════════════════════════ -->
            {#if stage.tabs()}
              <tabs variant={variant()} tone="primary" sx="w-full">
                <tab-list>
                  <tab>Overview</tab>
                  <tab>Activity</tab>
                  <tab>Settings</tab>
                </tab-list>
                <tab-panel><p sx="text-sm text-muted-foreground">Arrows rove, Home and End jump — wired for you.</p></tab-panel>
                <tab-panel><p sx="text-sm text-muted-foreground">The activity feed panel.</p></tab-panel>
                <tab-panel><p sx="text-sm text-muted-foreground">The settings panel.</p></tab-panel>
              </tabs>
            {/if}
            {#if stage.breadcrumb()}
              <breadcrumb>
                <a href="/">Home</a>
                <a href="/components">Components</a>
                <small>breadcrumb</small>
              </breadcrumb>
            {/if}
            {#if stage.pagination()}
              <column gap="3" align="center">
                <pagination value={pageNo()} size={variant()} on:sail-page-change={(e) => goPage(e)}>
                  <page value="1">1</page>
                  <page value="2">2</page>
                  <page value="3">3</page>
                  <page value="4">4</page>
                </pagination>
                <p sx="text-xs text-muted-foreground font-mono">on page {pageNo()} — click one</p>
              </column>
            {/if}
            {#if stage.navbar()}
              <navbar bordered sx="w-full">
                <strong slot="brand">SHIP</strong>
                <a href="/components">Docs</a>
                <a href="/components">Components</a>
                <button slot="actions" size="sm" variant="outline">Sign in</button>
              </navbar>
            {/if}

            <!-- ══ Data ════════════════════════════════════════════════ -->
            {#if stage.table()}
              <table sx="w-full">
                <thead>
                  <tr><th>Name</th><th>Team</th><th>Status</th></tr>
                </thead>
                <tbody>
                  {#for p of people() key={p.id}}
                    <tr>
                      <td>{p.name}</td>
                      <td>{p.team}</td>
                      <td><badge tone={p.status === 'active' ? 'success' : 'muted'} size="sm">{p.status}</badge></td>
                    </tr>
                  {/for}
                </tbody>
              </table>
            {/if}

            <!-- ══ Layout ══════════════════════════════════════════════ -->
            {#if stage.layout()}
              <column gap="3" sx="w-full max-w-panel-max-w-md">
                <row gap="2" align="center" justify="between">
                  <strong>row — the x axis</strong>
                  <badge tone="primary">end</badge>
                </row>
                <grid cols="3" gap="2">
                  <card sx="p-3"><center>1</center></card>
                  <card sx="p-3"><center>2</center></card>
                  <card sx="p-3"><center>3</center></card>
                </grid>
                <center sx="p-4 border border-border-subtle rounded-md">
                  <p sx="text-sm text-muted-foreground">center — both axes at once</p>
                </center>
              </column>
            {/if}
            {#if stage.scrollArea()}
              <scroll-area sx="max-h-panel-max-h-sm w-full max-w-panel-max-w-sm p-3 border border-border-subtle rounded-md">
                <p>Constrain it, and the overflow scrolls behind thin, themed scrollbars.</p>
                <p>Line two.</p>
                <p>Line three.</p>
                <p>Line four.</p>
                <p>Line five.</p>
                <p>Line six.</p>
                <p>Line seven.</p>
                <p>Line eight.</p>
                <p>Line nine.</p>
                <p>Line ten — still scrolling.</p>
              </scroll-area>
            {/if}

            <!-- ══ Pack components ═══════════════════════════════════════ -->
            {#if stage.suiSparkline()}
              <column gap="3" align="center" data-tile-stage="sail-pack-sparkline">
                <sail-sparkline data="2,5,3,8,4,6,7" width="200" height="60"></sail-sparkline>
              </column>
            {/if}
            {#if stage.suiAreaChart()}
              <column gap="3" align="center" data-tile-stage="sail-pack-area-chart">
                <sail-area-chart series='[{"name":"a","data":[2,4,3,5]}]' labels="Q1,Q2,Q3,Q4" width="200" height="120"></sail-area-chart>
              </column>
            {/if}
            {#if stage.suiBarChart()}
              <column gap="3" align="center" data-tile-stage="sail-pack-bar-chart">
                <sail-bar-chart data="2,5,3,8,4" labels="A,B,C,D,E" width="200" height="120"></sail-bar-chart>
              </column>
            {/if}
            {#if stage.suiDonut()}
              <column gap="3" align="center" data-tile-stage="sail-pack-donut">
                <sail-donut data='[{"label":"a","value":40},{"label":"b","value":30},{"label":"c","value":30}]' diameter="120" thickness="12"></sail-donut>
              </column>
            {/if}
            {#if stage.suiGauge()}
              <column gap="3" align="center" data-tile-stage="sail-pack-gauge">
                <sail-gauge value="72" diameter="120" thickness="12"></sail-gauge>
              </column>
            {/if}
            {#if stage.suiHeatmap()}
              <column gap="3" align="center" data-tile-stage="sail-pack-heatmap">
                <sail-heatmap rows="4" cols="6" values="1,2,3,4,3,2,4,5,6,5,4,3,2,1,2,3,4,3,2,1,2,3,4,5" width="200" height="120"></sail-heatmap>
              </column>
            {/if}
            {#if stage.suiLineChart()}
              <column gap="3" align="center" data-tile-stage="sail-pack-line-chart">
                <sail-line-chart series='[{"name":"a","data":[2,4,3,5]}]' labels="Q1,Q2,Q3,Q4" width="200" height="120"></sail-line-chart>
              </column>
            {/if}
            {#if stage.suiMediaImage()}
              <column gap="3" align="center" data-tile-stage="sail-pack-media-image">
                <sail-media-image src="/sail-oss/sail-mark.svg" alt="SUI mark"></sail-media-image>
              </column>
            {/if}
            {#if stage.suiMediaAv()}
              <column gap="3" align="center" data-tile-stage="sail-pack-media-av">
                <sail-media-av></sail-media-av>
              </column>
            {/if}
            {#if stage.suiCodeEditor()}
              <column gap="3" align="center" data-tile-stage="sail-pack-code-editor">
                <sail-code-editor value="const greeting = 'hi';" language="js" line-numbers size="md"></sail-code-editor>
              </column>
            {/if}
            {#if stage.suiMermaid()}
              <column gap="3" align="center" data-tile-stage="sail-pack-mermaid">
                <sail-mermaid value="graph TD; A[Idea]-->B[Build]; B-->C[Ship]"></sail-mermaid>
              </column>
            {/if}
            {#if stage.suiMessage()}
              <column gap="3" align="center" data-tile-stage="sail-pack-message">
                <sail-message role="user" state="user">Hello — the live chat message primitive.</sail-message>
              </column>
            {/if}
            {#if stage.suiCitation()}
              <column gap="3" align="center" data-tile-stage="sail-pack-citation">
                <sail-citation source="docs.sail.dev" href="https://docs.sail.dev">A citation from the docs</sail-citation>
              </column>
            {/if}
            {#if stage.suiComposer()}
              <column gap="3" align="center" data-tile-stage="sail-pack-composer">
                <sail-composer placeholder="Ask anything…"></sail-composer>
              </column>
            {/if}
            {#if stage.suiThinkingDots()}
              <column gap="3" align="center" data-tile-stage="sail-pack-thinking-dots">
                <sail-thinking-dots></sail-thinking-dots>
              </column>
            {/if}
            {#if stage.suiToolCall()}
              <column gap="3" align="center" data-tile-stage="sail-pack-tool-call">
                <sail-tool-call name="search" status="running">query: search</sail-tool-call>
              </column>
            {/if}
            {#if stage.suiCanvas()}
              <column gap="3" align="center" data-tile-stage="sail-pack-canvas">
                <sail-canvas width="200" height="120"></sail-canvas>
              </column>
            {/if}
            {#if stage.suiBoard()}
              <column gap="3" align="center" data-tile-stage="sail-pack-board">
                <sail-board width="200" height="120">
                  <sail-canvas width="200" height="120"></sail-canvas>
                </sail-board>
              </column>
            {/if}
            {#if stage.suiDraw()}
              <column gap="3" align="center" data-tile-stage="sail-pack-draw">
                <sail-draw width="200" height="120"></sail-draw>
              </column>
            {/if}
            {#if stage.suiGraphCanvas()}
              <column gap="3" align="center" data-tile-stage="sail-pack-graph-canvas">
                <sail-graph-canvas width="200" height="120" nodes='[{"id":"a","kind":"agent","title":"A","x":80,"y":60},{"id":"b","kind":"skill","title":"B","x":120,"y":60}]' edges="[]"></sail-graph-canvas>
              </column>
            {/if}
            {#if stage.suiAnnotationLayer()}
              <column gap="3" align="center" data-tile-stage="sail-pack-annotation-layer">
                <sail-annotation-layer></sail-annotation-layer>
              </column>
            {/if}
            {#if stage.suiAnnotationList()}
              <column gap="3" align="center" data-tile-stage="sail-pack-annotation-list">
                <sail-annotation-list></sail-annotation-list>
              </column>
            {/if}
            {#if stage.suiAnnotationMarker()}
              <column gap="3" align="center" data-tile-stage="sail-pack-annotation-marker">
                <sail-annotation-marker x="50" y="30"></sail-annotation-marker>
              </column>
            {/if}

      </column>
      <p slot="stage" sx="text-xs text-muted-foreground font-mono">{slotCopy}</p>
    </sail-component-viewer>

    <row gap="2">
      <a variant="default" href="/components">← All components</a>
      <a variant="solid" tone="primary" href="/reference">API reference</a>
    </row>
  {:else}
    <breadcrumb>
      <a href="/components">components</a>
      <small>{tag()}</small>
    </breadcrumb>
    <empty-state motion:enter={{ opacity: 'fade', translate-y: 'spring:snappy' }}>
      <strong slot="title">Nothing named “{tag()}” here</strong>
      Every element on the L1 surface has a detail page. Pack components
      (the 6 un-gated packs) render here too — this looks like a typo.
    </empty-state>
    <row gap="2">
      <a variant="solid" tone="primary" href="/components">← All components</a>
    </row>
  {/if}
</column>
