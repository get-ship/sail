<!--
  <design-inspector> — live specimen + scale-family / type / density tools.
  Mounted by design-surface. Ban-clean: no div/span; layout + L1 only.
-->
<controller>
import { createDesignInspector } from './design-inspector-vm'
export default controller(() => {
  const vm = createDesignInspector()
  return {
    designThemeOpts: vm.designThemeOpts, designThemeId: vm.designThemeId,
    radiusSteps: vm.radiusSteps, iconScaleSteps: vm.iconScaleSteps,
    inheritRows: vm.inheritRows, densityCompact: vm.densityCompact,
    densityDefault: vm.densityDefault, densitySpacious: vm.densitySpacious,
    scaleTogetherOn: vm.scaleTogetherOn, blastLabel: vm.blastLabel,
    inspectOpen: vm.inspectOpen, inspectSites: vm.inspectSites,
    isType: vm.isType, isDensity: vm.isDensity, isRadius: vm.isRadius,
    isIcons: vm.isIcons, isColor: vm.isColor, showScaleFamily: vm.showScaleFamily,
    typeDisplay: vm.typeDisplay, typeBody: vm.typeBody, typeMono: vm.typeMono,
    iconPack: vm.iconPack, iconQuery: vm.iconQuery, iconGlyphs: vm.iconGlyphs,
    forkName: vm.forkName, fontSearchQ: vm.fontSearchQ, fontHits: vm.fontHits,
    fontNote: vm.fontNote, designDensity: vm.designDensity,
    densityLabel: vm.densityLabel, overrideRows: vm.overrideRows,
    hasOverrides: vm.hasOverrides, groupLabel: vm.groupLabel,
    clampNote: vm.clampNote, hasClampNote: vm.hasClampNote,
    onDensityClick: (e: Event) => vm.onDensityClick(e),
    onScaleTogether: (e: Event) => vm.onScaleTogether(e),
    onRadiusInput: (e: Event) => vm.onRadiusInput(e),
    onInspectToggle: (e: Event) => vm.onInspectToggle(e),
    onInheritReset: (e: Event) => vm.onInheritReset(e),
    onInheritOverride: (e: Event) => vm.onInheritOverride(e),
    onForkTheme: (e: Event) => vm.onForkTheme(e),
    onForkName: (e: Event) => vm.onForkName(e),
    onIconScaleInput: (e: Event) => vm.onIconScaleInput(e),
    onTypeRole: (e: Event) => vm.onTypeRole(e),
    onFontSearch: (e: Event) => vm.onFontSearch(e),
    onFontAdd: (e: Event) => vm.onFontAdd(e),
    onFontHit: (e: Event) => vm.onFontHit(e),
    onIconPack: (e: Event) => vm.onIconPack(e),
    onIconQuery: (e: Event) => vm.onIconQuery(e),
  }
})
</controller>

<scroll-area
  data-design-inspector
  data-specimen-canvas
  gap="0"
  grow
  sx={style.minH(0).h(full).minW(56).borderLWidth(1).borderL(border-subtle).bg(surface-1)}
>
  <column gap="4" sx={style.pad(4).fontSize(sm)}>
    <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>Live specimen</p>
    <column
      data-specimen-live
      data-specimen-fallback
      data-specimen-panel
      gap="4"
      sx={style.pad(5).radius(lg).bg(background).fg(foreground).borderWidth(1).border(border-subtle).minW(0)}
    >
      <p data-specimen-hero sx={style.fontSize(xl).fontWeight(semibold).fontFamily(heading)}>
        Aa · Preview
      </p>
      <p sx={style.fontSize(sm).fg(muted-foreground).leading(snug)}>
        Surfaces update as you edit — this is the truth, not a mock.
      </p>
      <row gap="3">
        <button type="button" tone="primary">Primary</button>
        <button type="button" variant="outline">Outline</button>
        <button type="button" tone="destructive">Danger</button>
        <badge tone="success">ok</badge>
      </row>
      <card sx={style.pad(3).radius(lg).borderWidth(1).border(border-subtle)}>
        <p sx={style.fontWeight(medium)}>Card surface</p>
        <p sx={style.fontSize(sm).fg(muted-foreground)}>Density: {densityLabel()}</p>
      </card>
      <row gap="2">
        <badge tone="primary" size="sm">primary</badge>
        <badge tone="success" size="sm">success</badge>
        <badge tone="warning" size="sm">warning</badge>
        <badge tone="destructive" size="sm">danger</badge>
      </row>
      <alert tone="info" sx={style.fontSize(sm)}>
        Alert · info tone for status copy
      </alert>
      <input type="search" placeholder="Search field…" />
      <row gap="2" align="center">
        <icon name="sparkles" size="sm" label="sparkles"></icon>
        <icon name="check" size="md" label="check"></icon>
        <icon name="settings" size="lg" label="settings"></icon>
      </row>
    </column>

    <if {showScaleFamily()}>
      <if {isRadius()}>
        <column data-panel="radius" gap="2">
          <row align="center" gap="2" justify="between" nowrap>
            <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>
              Corner scale
            </p>
            <label sx={style.fontSize(xs)} data-scale-together>
              <input type="checkbox" checked={scaleTogetherOn()} on:change={onScaleTogether} />
              Link steps
            </label>
          </row>
          <column gap="2" data-radius-sliders>
            <for {radiusSteps()} as="s" key={s.rowKey}>
              <row align="center" gap="2">
                <p sx={style.w(8).fontSize(xs)}>{s.label}</p>
                <input
                  type="range"
                  min="0"
                  max="32"
                  step="1"
                  value={s.valuePx}
                  data-radius-step={s.key}
                  on:input={onRadiusInput}
                />
                <p sx={style.fontFamily(mono).fontSize(xs).w(8)}>{s.raw}</p>
              </row>
            </for>
          </column>
          <if {hasClampNote()}>
            <p data-clamp-note sx={style.fontSize(xs).fg(muted-foreground)}>{clampNote()}</p>
          </if>
          <row gap="2" align="center">
            <p data-blast-radius sx={style.fontSize(xs).fg(muted-foreground)}>{blastLabel()}</p>
            <button
              type="button"
              variant="soft"
              tone="muted"
              data-inspect-sites
              on:click={onInspectToggle}
            >
              Where used
            </button>
          </row>
          <if {inspectOpen()}>
            <scroll-area data-inspect-panel sx={style.maxH(40)}>
              <column gap="1" sx={style.fontSize(xs).fontFamily(mono)}>
                <for {inspectSites()} as="s" key={s.key}>
                  <p data-site-file>{s.file}</p>
                </for>
              </column>
            </scroll-area>
          </if>
          <column data-inheritance gap="1" sx={style.padT(2)}>
            <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>
              Inheritance · base → theme → override
            </p>
            <for {inheritRows()} as="n" key={n.key}>
              <row data-inherit-id={n.id} align="center" gap="2" sx={style.fontSize(xs)}>
                <p grow min-w="0">{n.label}</p>
                <if {n.canReset}>
                  <button type="button" data-inherit-reset on:click={onInheritReset}>
                    Reset
                  </button>
                </if>
                <if {n.canCustomize}>
                  <button
                    type="button"
                    variant="soft"
                    data-inherit-override
                    on:click={onInheritOverride}
                  >
                    Customize
                  </button>
                </if>
              </row>
            </for>
          </column>
        </column>
      </if>

      <if {isIcons()}>
        <column data-panel="icons" gap="2">
          <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>Icon pack</p>
          <label sx={style.fontSize(xs)}>
            Pack
            <select data-icon-pack value={iconPack()} on:change={onIconPack}>
              <option value="lucide">Lucide</option>
              <option value="phosphor">Phosphor (soon)</option>
            </select>
          </label>
          <input
            type="search"
            data-icon-query
            placeholder="Search icons…"
            value={iconQuery()}
            on:input={onIconQuery}
          />
          <scroll-area data-icon-glyph-browser sx={style.maxH(32)}>
            <column gap="1">
              <for {iconGlyphs()} as="g" key={g.key}>
                <row
                  data-icon-glyph={g.name}
                  align="center"
                  gap="2"
                  sx={style.pad(1).fontSize(xs)}
                  title={g.name}
                >
                  <icon name={g.name} size="sm" label={g.name}></icon>
                  <p>{g.name}</p>
                </row>
              </for>
            </column>
          </scroll-area>
          <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>Icon sizes</p>
          <column gap="2" data-icon-size-sliders>
            <for {iconScaleSteps()} as="s" key={s.rowKey}>
              <row align="center" gap="2">
                <p sx={style.w(8).fontSize(xs)}>{s.label}</p>
                <input
                  type="range"
                  min="8"
                  max="48"
                  step="1"
                  value={s.valuePx}
                  data-icon-step={s.key}
                  on:input={onIconScaleInput}
                />
                <p sx={style.fontFamily(mono).fontSize(xs).w(8)}>{s.raw}</p>
              </row>
            </for>
          </column>
        </column>
      </if>
    </if>

    <if {isType()}>
      <column data-panel="type" gap="2">
        <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>Type roles</p>
        <column gap="1" data-type-role-display>
          <label sx={style.fontSize(xs)}>
            Display
            <input data-type-role="display" value={typeDisplay()} on:change={onTypeRole} />
          </label>
          <p sx={style.fontSize(xl).fontWeight(semibold).fontFamily(heading)}>{typeDisplay()}</p>
        </column>
        <column gap="1" data-type-role-body>
          <label sx={style.fontSize(xs)}>
            Body
            <input data-type-role="body" value={typeBody()} on:change={onTypeRole} />
          </label>
          <p sx={style.fontSize(sm)}>The quick brown fox — {typeBody()}</p>
        </column>
        <column gap="1" data-type-role-mono>
          <label sx={style.fontSize(xs)}>
            Mono
            <input data-type-role="mono" value={typeMono()} on:change={onTypeRole} />
          </label>
          <p sx={style.fontSize(xs).fontFamily(mono)}>const ok = true // {typeMono()}</p>
        </column>
        <row gap="2">
          <input
            type="search"
            data-font-search
            placeholder="Search fonts…"
            value={fontSearchQ()}
            on:input={onFontSearch}
          />
          <button type="button" data-font-add on:click={onFontAdd}>
            Add
          </button>
        </row>
        <column gap="2" data-font-hits data-font-cmdk>
          <for {fontHits()} as="f" key={f.key}>
            <button
              type="button"
              variant="ghost"
              data-font-hit={f.name}
              on:click={onFontHit}
              sx={style.w(full).minH(9).h(auto).padY(2).whitespace(normal)}
            >
              {f.name}
            </button>
          </for>
        </column>
        <p data-font-note sx={style.fontSize(xs).fg(muted-foreground)}>{fontNote()}</p>
      </column>
    </if>

    <if {isDensity()}>
      <column data-panel="density" gap="3">
        <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>Spacing density</p>
        <column gap="2" data-density-switch on:click={onDensityClick}>
          <button
            type="button"
            data-density="compact"
            variant={densityCompact() ? 'soft' : 'ghost'}
            tone={densityCompact() ? 'primary' : 'muted'}
            sx={style.w(full).minH(9)}
          >
            Compact
          </button>
          <button
            type="button"
            data-density="default"
            variant={densityDefault() ? 'soft' : 'ghost'}
            tone={densityDefault() ? 'primary' : 'muted'}
            sx={style.w(full).minH(9)}
          >
            Comfortable
          </button>
          <button
            type="button"
            data-density="spacious"
            variant={densitySpacious() ? 'soft' : 'ghost'}
            tone={densitySpacious() ? 'primary' : 'muted'}
            sx={style.w(full).minH(9)}
          >
            Roomy
          </button>
        </column>
        <p sx={style.fontSize(xs).fg(muted-foreground).leading(snug)}>
          Active: {densityLabel()} — specimen list spacing follows live.
        </p>
      </column>
    </if>

    <if {isColor()}>
      <column data-panel="brand-fork" gap="2">
        <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>Copy as new theme</p>
        <row gap="2" align="center">
          <input
            type="text"
            data-fork-name
            placeholder="my-brand"
            value={forkName()}
            on:input={onForkName}
          />
          <button type="button" data-fork-theme on:click={onForkTheme}>
            Create
          </button>
        </row>
      </column>
    </if>

    <column gap="2" data-advanced-overrides>
      <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>
        Changed tokens
      </p>
      <if {!hasOverrides()}>
        <p sx={style.fontSize(xs).fg(muted-foreground)}>No changes yet.</p>
      </if>
      <if {hasOverrides()}>
        <column gap="1">
          <for {overrideRows()} as="o" key={o.key}>
            <row gap="2" align="center" data-override-path={o.path} nowrap>
              <if {o.isColor}>
                <center
                  data-swatch={o.value}
                  shrink="0"
                  sx={style.w(4).h(4).radius(sm).borderWidth(1).border(border-subtle)}
                  aria-hidden="true"
                ></center>
              </if>
              <column gap="0" grow min-w="0">
                <p sx={style.fontSize(xs).truncate()}>{o.label}</p>
                <p sx={style.fontSize(xs).fg(muted-foreground).truncate()}>{o.display}</p>
              </column>
            </row>
          </for>
        </column>
      </if>
    </column>
  </column>
</scroll-area>
