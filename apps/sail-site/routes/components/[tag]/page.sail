<!--
  /components/[tag] — detail route. Live stage + variant chips + usage + props.
  L1 from component-details; pack tags from component-pack-meta. Template reads
  only zero-arg calls (`entry()`, `stage.button()`).
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { DETAILS, type DetailEntry } from '../../../src/compositions/component-details'
import {
  DEMO_PHOTO,
  PACK_PROPS,
  PACK_TAGS,
  packBlurbFor,
  packLabelFor,
  packSlotCopy,
} from '../../../src/compositions/component-pack-meta'

export default controller((ctx: ControllerScope<Scope>) => {
    const tag = derived(() => ctx.route.params.tag?.() ?? '')
    const entry = derived<DetailEntry | null>(() => DETAILS[tag()] ?? null)
    const known = derived(() => entry() !== null)
    // Pack tags lack DETAILS rows; still mount via knownPack (catalog skips
    // message-list/media-csv — importmap gap, not this route).
    const knownPack = derived(() => PACK_TAGS.has(tag()))
    const showViewer = derived(() => known() || knownPack())

    const variant = signal('')
    // Button owns its own axis toolbar (variant/size/tone) — not the viewer's single chip row.
    const btnSize = signal('md')
    const btnTone = signal('primary')
    // Scope is reused across detail navigations — re-seed on tag change.
    effect(() => {
      variant.set(entry()?.variants[0] ?? '')
      if (tag() === 'button') {
        btnSize.set('md')
        btnTone.set('primary')
      }
    })
    const pick = (v: string): void => variant.set(v)
    const onVariantChange = (detail: { variant: string }): void =>
      variant.set(detail?.variant ?? '')
    const pickBtnVariant = (v: string): void => variant.set(v)
    const pickBtnSize = (s: string): void => btnSize.set(s)
    const pickBtnTone = (t: string): void => btnTone.set(t)
    // Derived chip rows — templates only allow zero-arg reads.
    const variantChips = derived(() =>
      (['default', 'solid', 'soft', 'outline', 'ghost'] as const).map((v) => ({
        v,
        chip: variant() === v ? 'solid' : 'ghost',
      })),
    )
    const sizeChips = derived(() =>
      (['xs', 'sm', 'md', 'lg', 'icon', 'iconSm', 'iconLg'] as const).map((s) => ({
        s,
        chip: btnSize() === s ? 'solid' : 'ghost',
      })),
    )
    const toneChips = derived(() =>
      (['primary', 'success', 'warning', 'destructive', 'accent', 'muted'] as const).map((t) => ({
        t,
        chip: btnTone() === t ? 'solid' : 'ghost',
      })),
    )
    const btnMeta = derived(() => `${variant()} · ${btnSize()} · ${btnTone()} · press ×${presses()}`)
    const isIconSize = derived(() => btnSize().startsWith('icon'))
    // Hide viewer variant strip for button — we own a unified axis toolbar.
    const viewerVariants = () => (tag() === 'button' ? [] : entry()?.variants ?? [])
    const viewerVariantAttr = () => (tag() === 'button' ? '' : entry()?.variantAttr ?? '')
    // Example walkthrough snippets (code + prose below the stage — not a second matrix).
    const btnExWithIcon = () =>
      `<button variant="solid" tone="primary">\n  <icon name="check" size="sm"></icon>\n  Save\n</button>`
    const btnExIconOnly = () =>
      `<button variant="solid" tone="primary" size="icon" label="Settings">\n  <icon name="settings"></icon>\n</button>\n<button variant="ghost" size="iconSm" label="Search">\n  <icon name="search" size="sm"></icon>\n</button>`
    const btnExSizes = () =>
      `<button variant="solid" tone="primary" size="xs">xs</button>\n<button variant="solid" tone="primary" size="lg">lg</button>`

    // sail-media-image src is a JS identity property, not a reflect attr.
    effect(() => {
      void tag()
      requestAnimationFrame(() => {
        document.querySelectorAll('sail-media-image').forEach((el) => {
          const host = el as HTMLElement & { src?: string }
          if (host.src !== DEMO_PHOTO) host.src = DEMO_PHOTO
        })
      })
    })

    const variants = () => entry()?.variants ?? []
    const hasVariants = derived(() => (entry()?.variants.length ?? 0) > 0)
    const variantAttr = () => entry()?.variantAttr ?? ''
    const blurb = () => entry()?.blurb ?? ''
    const props = () => entry()?.props ?? []
    const usageCode = () => entry()?.usage ?? ''
    const slotCopy = () => (knownPack() ? packSlotCopy(tag()) : '')
    const heading = () => entry()?.heading ?? tag()
    const packBlurb = () => packBlurbFor(tag())
    const packProps = () => PACK_PROPS[tag()] ?? []
    const packLabel = () => packLabelFor(tag())

    const stage = Object.fromEntries([
      ...Object.values(DETAILS).map((e) => [e.key, () => entry()?.key === e.key]),
      ...[...PACK_TAGS].map((k) => [k.replace(/-([a-z])/g, (_, c) => c.toUpperCase()), () => tag() === k]),
    ]) as Record<string, () => boolean>

    const presses = signal(0)
    const press = (): void => presses.set((n) => n + 1)
    const notify = signal(true)
    const agreed = signal(false)
    const load = signal(62)
    const name = signal('')
    const bio = signal('')
    const team = signal('Design')
    const fw = signal('')
    const pickFw = (detail: { value: string }): void => fw.set(detail?.value ?? '')
    const pageNo = signal('2')
    const goPage = (detail: { page?: string; value?: string }): void => pageNo.set(detail?.page ?? detail?.value ?? '1')
    const kept = signal(true)
    const dropChip = (): void => kept.set(false)
    const restoreChip = (): void => kept.set(true)
    // Toast stage owns instances (push → dismiss → re-push). L1 remove() is
    // correct; the stage must remount via signal, not treat one node as forever.
    let toastSeq = 0
    const toastItems = signal<{ key: string }[]>([{ key: 'toast-0' }])
    const pushToast = (): void => {
      toastSeq += 1
      toastItems.set([{ key: `toast-${toastSeq}` }])
    }
    const onToastDismiss = (): void => {
      toastItems.set([])
    }
    // Re-seed when navigating onto the toast detail (scope is reused).
    effect(() => {
      if (tag() === 'toast') {
        toastSeq = 0
        toastItems.set([{ key: 'toast-0' }])
      }
    })
    const sheetSide = () => (variant() === 'sheet' ? 'end' : null)
    const people = () => [
      { id: 1, name: 'Ada Lovelace', team: 'Engines', status: 'active' },
      { id: 2, name: 'Grace Hopper', team: 'Compilers', status: 'active' },
      { id: 3, name: 'Alan Turing', team: 'Research', status: 'away' },
    ]

    return {
      tag, entry, known, knownPack, showViewer, heading,
      variant, pick, onVariantChange, variants, hasVariants, variantAttr,
      viewerVariants, viewerVariantAttr,
      blurb, props, packBlurb, packProps, packLabel, usageCode, stage, slotCopy,
      presses, press, notify, agreed, load, name, bio, team,
      fw, pickFw, pageNo, goPage, kept, dropChip, restoreChip,
      toastItems, pushToast, onToastDismiss,
      sheetSide, people,
      btnSize, btnTone, pickBtnVariant, pickBtnSize, pickBtnTone,
      variantChips, sizeChips, toneChips, btnMeta, isIconSize,
      btnExWithIcon, btnExIconOnly, btnExSizes,
    }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  // Explicit width (not minW) — `.sail-col > * { min-width: 0 }` zeros any
  // min-width on children, so minW(panel-*) never lands. width: token wins.
  style stageCol = { box: w(panel-max-w-sm).maxW(full) }
  style stageColMd = { box: w(panel-max-w-md).maxW(full) }
  style meterCol = { box: w(panel-max-w-sm) }
  style grow1 = { box: grow(1) }
  style cardPad = { box: pad(4) }
  style cardPad5 = { box: pad(5) }
  style mediaStage = { box: w(full).maxW(panel-max-w-sm).h(48).minH(48).overflow(hidden), paint: radius(md) }
  style fullMedia = { box: w(full).h(full) }
  style fullW = { box: w(full) }
  style ticker = { box: w(full).maxW(panel-max-w-md).overflow(hidden).padY(3), paint: bg(muted).radius(md) }
  style tickerText = { box: padX(4), paint: fg(muted-foreground), text: fontSize(sm).fontFamily(mono).whitespace(nowrap) }
  style scrollDemo = { box: maxH(panel-max-h-sm).w(full).maxW(panel-max-w-sm).pad(3), paint: borderWidth(1).border(border-subtle).radius(md) }
  style centerBox = { box: pad(4), paint: borderWidth(1).border(border-subtle).radius(md) }
  style pressBtn = { animate: press({ scale: 'press' }) }
  style emptyEnter = { animate: enter({ opacity: 'fade', translate-y: 'spring:snappy' }) }
  style skeletonBox = { box: w(7).h(7) }
  style ctxCard = { box: pad(6).w(full) }
  style toastStage = { box: w(full).maxW(panel-max-w-sm) }
  // Fixed-width axis keys so variant / size / tone rows share one vertical grid.
  style axisKey = { box: w(16).shrink(0), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).tracking(wide) }
  style axisChips = { box: grow(1).minW(0) }
  style stageCanvas = { box: w(full).padY(8).minH(14) }
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
</scope>

<column gap="4" style(page)>
  <if {showViewer()}>
    <sail-component-viewer
      name={heading()}
      blurb={knownPack() ? packBlurb() : blurb()}
      pack={knownPack() ? packLabel() : 'Component primitives'}
      variant-attr={viewerVariantAttr()}
      usage={usageCode()}
      variants={viewerVariants()}
      rows={knownPack() ? packProps() : props()}
      on-variant-change={onVariantChange}
    >
      <column slot="stage" gap="4" align="stretch" style({ box: w(full).minW(0) })>
            <!-- ══ Inputs ══════════════════════════════════════════════ -->
            <if {stage.button()}>
              <!-- Unified axis toolbar (left grid) + centered specimen. No breadcrumb / disable toy. -->
              <column gap="5" style(fullW)>
                <column gap="2" style(fullW)>
                  <row gap="3" align="center" style(fullW)>
                    <span style(axisKey)>variant</span>
                    <row gap="1" align="center" style(axisChips)>
                      <for {variantChips() as c} key={c.v}>
                        <button size="sm" tone="primary" variant={c.chip} on:click={() => pickBtnVariant(c.v)}>{c.v}</button>
                      </for>
                    </row>
                  </row>
                  <row gap="3" align="center" style(fullW)>
                    <span style(axisKey)>size</span>
                    <row gap="1" align="center" style(axisChips)>
                      <for {sizeChips() as c} key={c.s}>
                        <button size="sm" tone="primary" variant={c.chip} on:click={() => pickBtnSize(c.s)}>{c.s}</button>
                      </for>
                    </row>
                  </row>
                  <row gap="3" align="center" style(fullW)>
                    <span style(axisKey)>tone</span>
                    <row gap="1" align="center" style(axisChips)>
                      <for {toneChips() as c} key={c.t}>
                        <button size="sm" tone="primary" variant={c.chip} on:click={() => pickBtnTone(c.t)}>{c.t}</button>
                      </for>
                    </row>
                  </row>
                </column>
                <center style(stageCanvas)>
                  <column gap="2" align="center">
                    <if {isIconSize()}>
                      <button
                        variant={variant()}
                        size={btnSize()}
                        tone={btnTone()}
                        on:click={press}
                        label="Settings"
                        style(pressBtn)
                      >
                        <icon name="settings"></icon>
                      </button>
                    {else =>
                      <button
                        variant={variant()}
                        size={btnSize()}
                        tone={btnTone()}
                        on:click={press}
                        style(pressBtn)
                      >
                        <icon name="check" size="sm"></icon>
                        Save changes
                      </button>}</if>
                    <p style(monoXs)>{btnMeta()}</p>
                  </column>
                </center>
              </column>
            </if>
            <if {stage.input()}>
              <column gap="3" style(stageCol)>
                <input placeholder="Ada Lovelace" bind:value={name} />
                <p style(monoXs)>hello, {name() || 'stranger'}</p>
              </column>
            </if>
            <if {stage.textarea()}>
              <column gap="3" style(stageCol)>
                <textarea placeholder="Tell us about yourself" bind:value={bio}></textarea>
                <p style(monoXs)>{bio().length} characters</p>
              </column>
            </if>
            <if {stage.select()}>
              <column gap="3" style(stageCol)>
                <select bind:value={team}>
                  <option>Engineering</option>
                  <option>Design</option>
                  <option>Research</option>
                </select>
                <p style(monoXs)>team: {team()}</p>
              </column>
            </if>
            <if {stage.combobox()}>
              <column gap="3" style(stageCol)>
                <combobox placeholder="Pick a framework — type to filter" on-select={pickFw}>
                  <option value="sail">Sail</option>
                  <option value="react">React</option>
                  <option value="svelte">Svelte</option>
                  <option value="solid">Solid</option>
                  <option value="vue">Vue</option>
                  <option value="qwik">Qwik</option>
                </combobox>
                <p style(monoXs)>picked: {fw() || 'nothing yet'}</p>
              </column>
            </if>
            <if {stage.checkbox()}>
              <column gap="3" align="center">
                <label><input type="checkbox" bind:checked={agreed} /> I agree to the terms</label>
                <p style(monoXs)>{agreed() ? 'thank you' : 'the platform checkbox, skinned'}</p>
              </column>
            </if>
            <if {stage.radioGroup()}>
              <radio-group label="Billing period">
                <label><input type="radio" name="stage-plan" checked /> Monthly</label>
                <label><input type="radio" name="stage-plan" /> Yearly</label>
              </radio-group>
            </if>
            <if {stage.switchToggle()}>
              <row gap="3" align="center">
                <switch bind:checked={notify} size={variant()} tone="primary"></switch>
                <p style(monoXs)>{notify() ? 'on' : 'off'} · size {variant()}</p>
              </row>
            </if>
            <if {stage.slider()}>
              <column gap="3" style(stageCol)>
                <slider bind:value={load} min="0" max="100"></slider>
                <progress value={load()} max="100"></progress>
                <p style(monoXs)>one signal, two components — {load()}%</p>
              </column>
            </if>
            <if {stage.toggleGroup()}>
              <toggle-group variant={variant()}>
                <toggle value="left">Left</toggle>
                <toggle value="center">Center</toggle>
                <toggle value="right">Right</toggle>
              </toggle-group>
            </if>
            <if {stage.field()}>
              <column gap="4" style(stageCol)>
                <field>
                  <label for="stage-name">Name</label>
                  <input id="stage-name" placeholder="Ada Lovelace" />
                </field>
                <field invalid>
                  <label for="stage-bad-email">Email</label>
                  <input id="stage-bad-email" value="not-an-email" />
                </field>
              </column>
            </if>
            <if {stage.label()}>
              <column gap="1" style(stageCol)>
                <label for="stage-email" required size={variant()}>Email</label>
                <input id="stage-email" placeholder="ada@lovelace.dev" />
              </column>
            </if>
            <if {stage.fieldset()}>
              <fieldset style(stageCol)>
                <legend>Account details</legend>
                <column gap="2">
                  <input placeholder="Name" />
                  <input placeholder="Email" />
                </column>
              </fieldset>
            </if>
            <if {stage.dropzone()}>
              <card dropzone style({ box: pad(6).w(full).maxW(panel-max-w-sm) })>Drop a file, or click to browse</card>
            </if>
            <if {stage.themePicker()}>
              <column gap="4" align="center">
                <column gap="1" align="center">
                  <span style(mutedXs)>Full catalog</span>
                  <theme-picker></theme-picker>
                </column>
                <column gap="1" align="center">
                  <span style(mutedXs)>Mode only</span>
                  <theme-picker mode-only></theme-picker>
                </column>
                <p style(monoXs)>themes this whole site, no reload</p>
              </column>
            </if>

            <!-- ══ Feedback ════════════════════════════════════════════ -->
            <if {stage.alert()}>
              <alert tone={variant()} style(fullW)>
                <strong>Heads up.</strong> The tone chips above restyle me — colour and icon both.
              </alert>
            </if>
            <if {stage.progressBar()}>
              <column gap="3" style(stageCol)>
                <slider bind:value={load} min="0" max="100"></slider>
                <progress value={load()} max="100"></progress>
                <p style(monoXs)>one signal drives both — {load()}%</p>
              </column>
            </if>
            <if {stage.meter()}>
              <!-- Fixed width: meter has no intrinsic width, and a percentage
                   can’t resolve inside the content-sized <center> stage. -->
              <column gap="2" style(meterCol)>
                <meter value="0.3"></meter>
                <meter value="0.7"></meter>
                <meter value="0.95"></meter>
              </column>
            </if>
            <if {stage.spinner()}>
              <row gap="4" align="center">
                <spinner size={variant()}></spinner>
                <spinner size={variant()} tone="primary"></spinner>
                <spinner size={variant()} tone="destructive"></spinner>
              </row>
            </if>
            <if {stage.skeleton()}>
              <row gap="3" align="center" style(stageCol)>
                <skeleton shape={variant()} style(skeletonBox)></skeleton>
                <column gap="2" style(grow1)>
                  <skeleton shape={variant()} size="sm"></skeleton>
                  <skeleton shape={variant()} size="sm"></skeleton>
                </column>
              </row>
            </if>
            <if {stage.emptyState()}>
              <empty-state variant={variant()} style(fullW)>
                <strong slot="title">No results</strong>
                Try a different search, or clear a filter.
              </empty-state>
            </if>
            <if {stage.errorState()}>
              <error-state variant={variant()} code="ERR_TIMEOUT" style(fullW)>
                The request timed out. Retry, or check the connection.
              </error-state>
            </if>
            <if {stage.errorBoundary()}>
              <column gap="3" style(stageCol)>
                <error-boundary message="This panel could not load.">
                  <card variant="outline" style(cardPad)>
                    <p style(mutedSm)>
                      A healthy boundary is invisible — this card is the wrapped
                      subtree. A render throw inside would swap in the fallback.
                    </p>
                  </card>
                </error-boundary>
              </column>
            </if>
            <if {stage.toast()}>
              <column gap="3" align="center" style(toastStage)>
                <button type="button" variant="outline" size="sm" on:click={pushToast} toast-show>
                  Show toast
                </button>
                <for {toastItems() as t} key={t.key}>
                  <toast tone={variant()} duration="0" on-dismiss={onToastDismiss} style(stageCol)>
                    <strong>Saved.</strong> Sticky at duration 0 — hover pauses real ones.
                  </toast>
                </for>
              </column>
            </if>

            <!-- ══ Display ═════════════════════════════════════════════ -->
            <if {stage.badge()}>
              <row gap="2" align="center">
                <badge variant={variant()} tone="primary">primary</badge>
                <badge variant={variant()} tone="success">success</badge>
                <badge variant={variant()} tone="warning">warning</badge>
                <badge variant={variant()} tone="destructive">destructive</badge>
                <badge variant={variant()} tone="muted">muted</badge>
              </row>
            </if>
            <if {stage.chip()}>
              <column gap="3" align="center">
                <row gap="2" align="center">
                  <!-- tone= so soft/outline/ghost match arms resolve --tone (rest solid = no tone). -->
                  <chip variant={variant()} tone="primary" selected>Selected</chip>
                  <if {kept()}>
                    <chip variant={variant()} tone="primary" removable on-remove={dropChip}>Removable</chip>
                  </if>
                  <if {!kept()}>
                    <button variant="ghost" size="sm" on:click={restoreChip}>bring it back</button>
                  </if>
                </row>
                <p style(monoXs)>× or Backspace fires sail-remove</p>
              </column>
            </if>
            <if {stage.statusDot()}>
              <row gap="2" align="center">
                <status-dot tone={variant()}></status-dot>
                <p style(mutedSm)>the tone is the message</p>
              </row>
            </if>
            <if {stage.card()}>
              <card variant={variant()} interactive style(cardPad5)>
                <strong>A surface</strong>
                <p style(mutedSm)>the hover lift is one attribute</p>
              </card>
            </if>
            <if {stage.avatar()}>
              <row gap="3" align="center">
                <avatar name="Ada Lovelace" size={variant()} src="https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=128&h=128&fit=crop&auto=format&q=80"></avatar>
                <avatar initials="GH" tone="primary" size={variant()}></avatar>
                <avatar name="Alan Turing" size={variant()} shape="square" src="https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=128&h=128&fit=crop&auto=format&q=80"></avatar>
              </row>
            </if>
            <if {stage.icon()}>
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
            </if>
            <if {stage.separator()}>
              <column gap="2" style(stageCol)>
                <p style(mutedSm)>above</p>
                <separator variant={variant()}></separator>
                <p style(mutedSm)>below — and vertical:</p>
                <row gap="2" align="center">
                  <span>A</span>
                  <separator orientation="vertical" variant={variant()}></separator>
                  <span>B</span>
                </row>
              </column>
            </if>
            <if {stage.accordion()}>
              <accordion style(stageCol)>
                <strong slot="trigger">Show the details</strong>
                <p>Animated open and close — and it remembers its state.</p>
              </accordion>
            </if>
            <if {stage.kbd()}>
              <p style(mutedSm)>                Press <kbd>Ctrl</kbd> + <kbd>K</kbd> to open the command palette.
              </p>
            </if>
            <if {stage.prose()}>
              <column gap="2" style(stageColMd)>
                <h3>Plain tags, already styled</h3>
                <p>Body copy sets the baseline — <strong>strong</strong>, <em>emphasis</em>,
                   <code>inline_code()</code> and <mark>a highlight</mark> need no classes.</p>
                <blockquote>A quotation carries its own rule.</blockquote>
              </column>
            </if>

            <!-- ══ Overlay ═════════════════════════════════════════════ -->
            <if {stage.tooltip()}>
              <!-- Stage is tall enough for top/bottom; long copy is fine here -->
              <tooltip content="Deletes the branch — cannot be undone" placement={variant()}>
                <button tone="destructive" variant="outline">Delete branch</button>
              </tooltip>
            </if>
            <if {stage.hoverCard()}>
              <column gap="3" align="center">
                <button variant="ghost" popovertarget="stage-hc">@ada</button>
                <hover-card id="stage-hc">
                  <card style(cardPad)>
                    <strong>Ada Lovelace</strong>
                    <p style(mutedSm)>wrote the first program.</p>
                  </card>
                </hover-card>
                <p style(monoXs)>focus-triggerable, not just mouse-only</p>
              </column>
            </if>
            <if {stage.popover()}>
              <column gap="3" align="center">
                <button popovertarget="stage-pop">Open popover</button>
                <popover id="stage-pop">
                  <card style(cardPad)>Rides the platform top layer — Escape or click outside closes it.</card>
                </popover>
                <p style(monoXs)>top layer · light dismiss · zero z-index</p>
              </column>
            </if>
            <if {stage.dropdown()}>
              <column gap="3" align="center">
                <button popovertarget="stage-menu">Actions ▾</button>
                <dropdown id="stage-menu">
                  <menu-item>Edit</menu-item>
                  <menu-item>Duplicate</menu-item>
                  <menu-item tone="destructive">Delete</menu-item>
                </dropdown>
                <p style(monoXs)>arrows rove · type-ahead · Escape closes</p>
              </column>
            </if>
            <if {stage.contextMenu()}>
              <column gap="3" align="center" style(stageCol)>
                <card id="stage-ctx" variant="outline" style(ctxCard)>
                  <center><p style(mutedSm)>right-click target</p></center>
                </card>
                <context-menu for="stage-ctx">
                  <menu-item value="cut">Cut</menu-item>
                  <menu-item value="copy">Copy</menu-item>
                  <menu-item value="paste">Paste</menu-item>
                </context-menu>
              </column>
            </if>
            <if {stage.dialog()}>
              <column gap="3" align="center">
                <button command="show-modal" commandfor="stage-dialog" style(pressBtn)>
                  Open the {variant()}
                </button>
                <p style(monoXs)>Escape and the backdrop close it — that’s the platform</p>
                <dialog id="stage-dialog" side={sheetSide()} size="md">
                  <header>Delete project</header>
                  <section>
                    This cannot be undone. The platform owns the top layer, focus trap, Escape, and backdrop — zero custom JS.
                  </section>
                  <footer>
                    <button command="close" commandfor="stage-dialog" variant="ghost" size="sm">Cancel</button>
                    <button command="close" commandfor="stage-dialog" tone="destructive" variant="solid" size="sm">Delete</button>
                  </footer>
                </dialog>
              </column>
            </if>

            <!-- ══ Navigation ══════════════════════════════════════════ -->
            <if {stage.tabs()}>
              <tabs style(fullW)>
                <tab-list variant={variant()}>
                  <tab>Overview</tab>
                  <tab>Activity</tab>
                  <tab>Settings</tab>
                </tab-list>
                <tab-panel><p style(mutedSm)>Arrows rove, Home and End jump — wired for you.</p></tab-panel>
                <tab-panel><p style(mutedSm)>The activity feed panel.</p></tab-panel>
                <tab-panel><p style(mutedSm)>The settings panel.</p></tab-panel>
              </tabs>
            </if>
            <if {stage.breadcrumb()}>
              <breadcrumb>
                <a href="/">Home</a>
                <a href="/components">Components</a>
                <small>breadcrumb</small>
              </breadcrumb>
            </if>
            <if {stage.pagination()}>
              <column gap="3" align="center">
                <pagination value={pageNo()} on-page-change={goPage}>
                  <page value="1">1</page>
                  <page value="2">2</page>
                  <page value="3">3</page>
                  <page value="4">4</page>
                </pagination>
                <p style(monoXs)>on page {pageNo()}</p>
              </column>
            </if>
            <if {stage.navbar()}>
              <navbar bordered style(fullW)>
                <strong slot="brand">SHIP</strong>
                <a href="/components">Docs</a>
                <a href="/components">Components</a>
                <button slot="actions" size="sm" variant="outline">Sign in</button>
              </navbar>
            </if>

            <!-- ══ Data ════════════════════════════════════════════════ -->
            <if {stage.table()}>
              <table style(fullW)>
                <thead>
                  <tr><th>Name</th><th>Team</th><th>Status</th></tr>
                </thead>
                <tbody>
                  <for {people() as p} key={p.id}>
                    <tr>
                      <td>{p.name}</td>
                      <td>{p.team}</td>
                      <td><badge tone={p.status === 'active' ? 'success' : 'muted'} size="sm">{p.status}</badge></td>
                    </tr>
                  </for>
                </tbody>
              </table>
            </if>

            <!-- ══ Layout ══════════════════════════════════════════════ -->
            <if {stage.layout()}>
              <column gap="3" style(stageColMd)>
                <row gap="2" align="center" justify="between">
                  <strong>row — the x axis</strong>
                  <badge tone="primary">end</badge>
                </row>
                <grid cols="3" gap="2">
                  <card style({ box: pad(3) })><center>1</center></card>
                  <card style({ box: pad(3) })><center>2</center></card>
                  <card style({ box: pad(3) })><center>3</center></card>
                </grid>
                <center style(centerBox)>
                  <p style(mutedSm)>center — both axes at once</p>
                </center>
              </column>
            </if>
            <if {stage.scrollArea()}>
              <scroll-area style(scrollDemo)>
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
            </if>
            <if {stage.ticker()}>
              <ticker duration="normal" style(ticker)>
                <p style(tickerText)>
                  tokens · signals · a11y · hot reload · forms ·&nbsp;
                </p>
              </ticker>
            </if>

            <!-- ══ Pack components ═══════════════════════════════════════ -->
            <if {stage.sailSparkline()}>
              <column gap="3" align="center" tile-stage="sail-pack-sparkline">
                <sail-sparkline data="2,5,3,8,4,6,7" width="200" height="60"></sail-sparkline>
              </column>
            </if>
            <if {stage.sailAreaChart()}>
              <column gap="3" align="center" tile-stage="sail-pack-area-chart">
                <sail-area-chart series='[{"name":"a","data":[2,4,3,5]}]' labels="Q1,Q2,Q3,Q4" width="200" height="120"></sail-area-chart>
              </column>
            </if>
            <if {stage.sailBarChart()}>
              <column gap="3" align="center" tile-stage="sail-pack-bar-chart">
                <sail-bar-chart data="2,5,3,8,4" labels="A,B,C,D,E" width="200" height="120"></sail-bar-chart>
              </column>
            </if>
            <if {stage.sailDonut()}>
              <column gap="3" align="center" tile-stage="sail-pack-donut">
                <sail-donut data='[{"label":"a","value":40},{"label":"b","value":30},{"label":"c","value":30}]' diameter="120" thickness="12"></sail-donut>
              </column>
            </if>
            <if {stage.sailGauge()}>
              <column gap="3" align="center" tile-stage="sail-pack-gauge">
                <sail-gauge value="72" diameter="120" thickness="12"></sail-gauge>
              </column>
            </if>
            <if {stage.sailHeatmap()}>
              <column gap="3" align="center" tile-stage="sail-pack-heatmap">
                <sail-heatmap rows="4" cols="6" values="1,2,3,4,3,2,4,5,6,5,4,3,2,1,2,3,4,3,2,1,2,3,4,5" cell-size="22" gap="3" tone="primary"></sail-heatmap>
              </column>
            </if>
            <if {stage.sailLineChart()}>
              <column gap="3" align="center" tile-stage="sail-pack-line-chart">
                <sail-line-chart series='[{"name":"a","data":[2,4,3,5]}]' labels="Q1,Q2,Q3,Q4" width="200" height="120"></sail-line-chart>
              </column>
            </if>
            <if {stage.sailMediaImage()}>
              <column gap="3" align="center" tile-stage="sail-pack-media-image" style(mediaStage)>
                <sail-media-image alt="Harbor at dusk" fit="cover" zoomable={false} style(fullMedia)></sail-media-image>
              </column>
            </if>
            <if {stage.sailMediaAv()}>
              <column gap="3" align="center" tile-stage="sail-pack-media-av">
                <sail-media-av></sail-media-av>
              </column>
            </if>
            <if {stage.sailCodeEditor()}>
              <column gap="3" align="center" tile-stage="sail-pack-code-editor">
                <sail-code-editor value="const greeting = 'hi';" language="js" line-numbers size="md"></sail-code-editor>
              </column>
            </if>
            <if {stage.sailMermaid()}>
              <column gap="3" align="center" tile-stage="sail-pack-mermaid">
                <sail-mermaid value="graph TD; A[Idea]-->B[Build]; B-->C[Ship]"></sail-mermaid>
              </column>
            </if>
            <if {stage.sailMessage()}>
              <column gap="3" tile-stage="sail-pack-message" style(stageColMd)>
                <sail-message from="user" status="sent">Can you summarize yesterday’s deploy?</sail-message>
                <sail-message from="assistant" status="sent">v0.4.2 went out at 14:02 — 3 services, zero rollbacks.</sail-message>
              </column>
            </if>
            <if {stage.sailCitation()}>
              <column gap="3" align="center" tile-stage="sail-pack-citation">
                <sail-citation source="docs.ship.dev" href="https://docs.ship.dev">Deploy pipeline overview</sail-citation>
              </column>
            </if>
            <if {stage.sailComposer()}>
              <column gap="3" tile-stage="sail-pack-composer" style(stageColMd)>
                <sail-composer placeholder="Message the agent…" style({ box: w(full).minW(0) })></sail-composer>
              </column>
            </if>
            <if {stage.sailThinkingDots()}>
              <column gap="3" align="center" tile-stage="sail-pack-thinking-dots" style(stageCol)>
                <row gap="2" align="center">
                  <sail-thinking-dots></sail-thinking-dots>
                  <span style({ paint: fg(muted-foreground), text: fontSize(sm) })>Agent is thinking…</span>
                </row>
              </column>
            </if>
            <if {stage.sailToolCall()}>
              <column gap="3" align="center" tile-stage="sail-pack-tool-call">
                <sail-tool-call name="list_deploys" status="ok" open>
                  {`{ "env": "prod", "limit": 5 }`}
                  <span slot="result">{`[{ "id": "d-1842", "status": "healthy" }]`}</span>
                </sail-tool-call>
              </column>
            </if>
            <if {stage.sailCanvas()}>
              <column gap="3" align="center" tile-stage="sail-pack-canvas">
                <sail-canvas width="200" height="120"></sail-canvas>
              </column>
            </if>
            <if {stage.sailBoard()}>
              <column gap="3" align="center" tile-stage="sail-pack-board">
                <sail-board width="200" height="120">
                  <sail-canvas width="200" height="120"></sail-canvas>
                </sail-board>
              </column>
            </if>
            <if {stage.sailDraw()}>
              <column gap="3" align="center" tile-stage="sail-pack-draw">
                <sail-draw width="200" height="120"></sail-draw>
              </column>
            </if>
            <if {stage.sailGraphCanvas()}>
              <column gap="3" align="center" tile-stage="sail-pack-graph-canvas">
                <sail-graph-canvas width="200" height="120" nodes='[{"id":"a","kind":"agent","title":"A","x":80,"y":60},{"id":"b","kind":"skill","title":"B","x":120,"y":60}]' edges="[]"></sail-graph-canvas>
              </column>
            </if>
            <if {stage.sailAnnotationLayer()}>
              <column gap="3" align="center" tile-stage="sail-pack-annotation-layer">
                <sail-annotation-layer></sail-annotation-layer>
              </column>
            </if>
            <if {stage.sailAnnotationList()}>
              <column gap="3" align="center" tile-stage="sail-pack-annotation-list">
                <sail-annotation-list></sail-annotation-list>
              </column>
            </if>
            <if {stage.sailAnnotationMarker()}>
              <column gap="3" align="center" tile-stage="sail-pack-annotation-marker">
                <sail-annotation-marker x="50" y="30"></sail-annotation-marker>
              </column>
            </if>

      </column>
      <if {slotCopy()}>
        <p slot="stage" style(monoXs)>{slotCopy()}</p>
      </if>

      <!-- Walkthroughs: prose + code (after usage/props). Not a second live matrix. -->
      <if {stage.button()}>
        <column gap="5" style(exampleBlock)>
          <p style(sectionLabel)>Examples</p>

          <column gap="2">
            <h3 style(exampleH)>With icons</h3>
            <p style(exampleLead)>
              Children compose: put <code>&lt;icon&gt;</code> before or after the label.
              The button is inline-flex with gap — no left/right section props.
            </p>
            <row gap="2" align="center">
              <button variant="solid" tone="primary">
                <icon name="check" size="sm"></icon>
                Save
              </button>
              <button variant="outline">
                Export
                <icon name="download" size="sm"></icon>
              </button>
            </row>
            <code-block code={btnExWithIcon()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Icon-only</h3>
            <p style(exampleLead)>
              Use <code>size="icon"</code>, <code>iconSm</code>, or <code>iconLg</code> for a square hit target.
              Always set <code>label</code> for accessible name when there is no text.
            </p>
            <row gap="2" align="center">
              <button variant="solid" tone="primary" size="icon" label="Settings">
                <icon name="settings"></icon>
              </button>
              <button variant="ghost" size="iconSm" label="Search">
                <icon name="search" size="sm"></icon>
              </button>
              <button variant="outline" size="iconLg" label="Add">
                <icon name="plus" size="lg"></icon>
              </button>
            </row>
            <code-block code={btnExIconOnly()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Sizes</h3>
            <p style(exampleLead)>
              Text controls: <code>xs</code>–<code>lg</code>. Defaults to <code>md</code>.
              Drive the live stage above for the full matrix; these are the named steps.
            </p>
            <row gap="2" align="center">
              <button variant="solid" tone="primary" size="xs">xs</button>
              <button variant="solid" tone="primary" size="sm">sm</button>
              <button variant="solid" tone="primary" size="md">md</button>
              <button variant="solid" tone="primary" size="lg">lg</button>
            </row>
            <code-block code={btnExSizes()}></code-block>
          </column>
        </column>
      </if>
    </sail-component-viewer>

    <row gap="2">
      <a variant="default" href="/components">← All components</a>
      <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
    </row>
  {else => <breadcrumb>
      <a href="/components">components</a>
      <small>{tag()}</small>
    </breadcrumb>
    <empty-state style(emptyEnter)>
      <strong slot="title">Nothing named “{tag()}” here</strong>
      Every element on the L1 surface has a detail page. Pack components
      (the 6 un-gated packs) render here too — this looks like a typo.
    </empty-state>
    <row gap="2">
      <a variant="solid" tone="primary" href="/components">← All components</a>
    </row>}</if>
</column>
