<!--
  <design-surface> — five design decisions + token power mode + specimen.
  Front door: Brand color · Type · Spacing density · Corners · Icons.
  Power mode: full token groups. Ban-clean: no div/span; layout + L1 only.
-->
<controller>
import { createDesignSurface } from './design-surface-vm'
export default controller(() => {
  const vm = createDesignSurface()
  return {
    groupRows: vm.groupRows,
    decisionRows: vm.decisionRows,
    tokenRows: vm.tokenRows,
    hasTokenRows: vm.hasTokenRows,
    designThemeOpts: vm.designThemeOpts,
    designThemeId: vm.designThemeId,
    designForkLabel: vm.designForkLabel,
    designOverridesLabel: vm.designOverridesLabel,
    designThemeLabel: vm.designThemeLabel,
    activeGroupLabel: vm.activeGroupLabel,
    activeGroupId: vm.activeGroupId,
    tokenSearchVal: vm.tokenSearchVal,
    isSearching: vm.isSearching,
    overridesOnlyOn: vm.overridesOnlyOn,
    designStatusNote: vm.designStatusNote,
    designError: vm.designError,
    hasDesignError: vm.hasDesignError,
    designWriteError: vm.designWriteError,
    hasDesignWriteError: vm.hasDesignWriteError,
    clampNote: vm.clampNote,
    hasClampNote: vm.hasClampNote,
    diffRows: vm.diffRows,
    hasDiff: vm.hasDiff,
    tokenCountLabel: vm.tokenCountLabel,
    brandHex: vm.brandHex,
    brandRamp: vm.brandRamp,
    contrastRows: vm.contrastRows,
    hasContrastFail: vm.hasContrastFail,
    contrastFailLabel: vm.contrastFailLabel,
    isBrandDecision: vm.isBrandDecision,
    saveReviewOpen: vm.saveReviewOpen,
    blastLabel: vm.blastLabel,
    saveButtonLabel: vm.saveButtonLabel,
    canConfirmSave: vm.canConfirmSave,
    onDecisionClick: (e: Event) => vm.onDecisionClick(e),
    onGroupClick: (e: Event) => vm.onGroupClick(e),
    onRetryDesign: () => vm.onRetryDesign(),
    onDesignTheme: (e: Event) => vm.onDesignTheme(e),
    onSaveTheme: () => vm.onSaveTheme(),
    onCancelSaveReview: () => vm.onCancelSaveReview(),
    onExportDiff: () => vm.onExportDiff(),
    onPreviewInApp: () => vm.onPreviewInApp(),
    onTokenSearch: (e: Event) => vm.onTokenSearch(e),
    onOverridesOnly: (e: Event) => vm.onOverridesOnly(e),
    onTokenText: (e: Event) => vm.onTokenText(e),
    onTokenNum: (e: Event) => vm.onTokenNum(e),
    onTokenReset: (e: Event) => vm.onTokenReset(e),
    onColorEditToggle: (e: Event) => vm.onColorEditToggle(e),
    onTokenColor: (e: Event) => vm.onTokenColor(e),
    onBrandColor: (e: Event) => vm.onBrandColor(e),
    onTokenRowActivate: (e: Event) => vm.onTokenRowActivate(e),
  }
})
</controller>

<column
  data-surface="design"
  data-design-surface
  gap="0"
  grow
  sx={style.minH(0).h(full).minW(0).overflow(hidden)}
>
  <row
    data-design-header
    align="center"
    gap="3"
    shrink="0"
    nowrap
    justify="between"
    sx={style.h(8).maxH(8).padX(3).borderBWidth(1).borderB(border-subtle).bg(surface-1).minW(0).overflow(hidden)}
  >
    <row align="center" gap="2" nowrap grow min-w="0" sx={style.minW(0).minH(0).maxH(full).overflow(hidden)}>
      <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
        Design
      </p>
      <select data-design-theme on:change={onDesignTheme} value={designThemeId()} w="28" size="sm" sx={style.maxH(6)}>
        <for {designThemeOpts()} as="t" key={t.key}>
          <option value={t.id}>{t.label}</option>
        </for>
      </select>
      <p data-design-theme-label sx={style.fontSize(xs).fg(muted-foreground).truncate()}>
        {designThemeLabel()}
      </p>
      <p data-design-fork sx={style.fontSize(xs).fg(muted-foreground).truncate()}>
        {designForkLabel()}
      </p>
      <p data-design-overrides sx={style.fontSize(xs).fg(primary)}>
        {designOverridesLabel()}
      </p>
    </row>
    <row gap="2" align="center" nowrap shrink="0" sx={style.minW(0)}>
      <button
        type="button"
        size="sm"
        variant="ghost"
        tone="muted"
        data-preview-in-app
        on:click={onPreviewInApp}
      >
        Preview
      </button>
      <button
        type="button"
        size="sm"
        variant="ghost"
        tone="muted"
        data-export-diff
        on:click={onExportDiff}
      >
        Copy changes
      </button>
      <button type="button" size="sm" tone="primary" data-save-theme on:click={onSaveTheme}>
        {saveButtonLabel()}
      </button>
    </row>
  </row>

  <if {hasDesignError()}>
    <row data-design-error align="center" gap="2" sx={style.padX(3).h(7).borderBWidth(1).borderB(border)}>
      <p grow min-w="0" sx={style.fg(destructive).fontSize(xs).truncate()}>{designError()}</p>
      <button type="button" size="sm" variant="ghost" data-retry-design on:click={onRetryDesign}>
        Retry
      </button>
    </row>
  </if>
  <if {hasDesignWriteError()}>
    <row data-design-write-error align="center" gap="2" sx={style.padX(3).h(7).borderBWidth(1).borderB(border)}>
      <p sx={style.fg(destructive).fontSize(xs)}>{designWriteError()}</p>
    </row>
  </if>
  <if {hasClampNote()}>
    <row data-clamp-note align="center" gap="2" sx={style.padX(3).h(6).borderBWidth(1).borderB(border-subtle).bg(surface-1)}>
      <p sx={style.fontSize(xs).fg(muted-foreground)}>{clampNote()}</p>
    </row>
  </if>
  <if {saveReviewOpen()}>
    <row
      data-save-review
      data-blast-radius
      align="center"
      gap="3"
      justify="between"
      sx={style.padX(3).padY(2).borderBWidth(1).borderB(border-subtle).bg(surface-1)}
    >
      <column gap="0" grow min-w="0">
        <p sx={style.fontSize(xs).fontWeight(semibold)}>Review before save</p>
        <p sx={style.fontSize(xs).fg(muted-foreground).truncate()}>{blastLabel()}</p>
      </column>
      <row gap="2" shrink="0">
        <button type="button" size="sm" variant="ghost" data-cancel-save on:click={onCancelSaveReview}>
          Cancel
        </button>
        <button type="button" size="sm" tone="primary" data-confirm-save on:click={onSaveTheme}>
          Confirm save
        </button>
      </row>
    </row>
  </if>

  <scroll-area
    data-decision-bar
    shrink="0"
    sx={style.maxH(16).borderBWidth(1).borderB(border-subtle).bg(surface-0).minW(0)}
  >
    <row
      gap="2"
      align="stretch"
      nowrap
      on:click={onDecisionClick}
      sx={style.padX(3).padY(2).minW(0)}
    >
      <for {decisionRows()} as="d" key={d.key}>
        <button
          type="button"
          size="sm"
          variant={d.active ? 'soft' : 'ghost'}
          tone={d.tone}
          data-decision={d.id}
          data-active={d.active ? 'true' : 'false'}
          sx={style.minW(28).h(12).maxH(12).padX(3).radius(md).when({ hover: style.bg(hover-bg) })}
        >
          <column gap="1" align="start" sx={style.w(full).maxH(10).overflow(hidden)}>
            <row gap="2" align="center" nowrap>
              <if {d.isColor}>
                <center
                  data-swatch={d.swatch}
                  shrink="0"
                  sx={style.w(3).h(3).radius(full).borderWidth(1).border(border-subtle)}
                  aria-hidden="true"
                ></center>
              </if>
              <p sx={style.fontSize(sm).fontWeight(semibold).truncate()}>{d.label}</p>
            </row>
            <p sx={style.fontSize(xs).fg(muted-foreground).truncate()}>{d.summary}</p>
          </column>
        </button>
      </for>
    </row>
  </scroll-area>

  <row grow gap="0" nowrap align="stretch" sx={style.minH(0).basis(0)}>
    <scroll-area
      data-group-rail
      data-decisions-rail
      shrink="0"
      sx={style.w(44).minH(0).h(full).borderRWidth(1).borderR(border-subtle).bg(surface-1)}
    >
      <column gap="0">
        <row align="center" sx={style.h(7).padX(3).borderBWidth(1).borderB(border-subtle)}>
          <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
            All tokens
          </p>
        </row>
        <column gap="0" on:click={onGroupClick} sx={style.padY(1)}>
          <for {groupRows()} as="g" key={g.key}>
            <button
              type="button"
              size="sm"
              variant={g.active ? 'soft' : 'ghost'}
              tone={g.tone}
              data-group={g.id}
              data-active={g.active ? 'true' : 'false'}
              sx={style.w(full).h(6).padX(3).radius(none).when({ hover: style.bg(hover-bg) })}
            >
              <row gap="2" align="center" grow nowrap>
                <p grow min-w="0" sx={style.truncate().fontSize(sm)}>{g.label}</p>
                <p sx={style.fontSize(xs).fg(muted-foreground)}>{g.count}</p>
              </row>
            </button>
          </for>
        </column>
      </column>
    </scroll-area>

    <scroll-area
      data-token-editor
      grow
      min-w="0"
      sx={style.minH(0).h(full).basis(0).minW(0).bg(surface-0)}
    >
      <column gap="0" sx={style.minW(0)}>
        <if {isBrandDecision()}>
          <column data-panel="brand" data-brand-panel gap="3" sx={style.pad(3).borderBWidth(1).borderB(border-subtle)}>
            <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
              Brand color
            </p>
            <row gap="3" align="center" nowrap>
              <sail-color-picker
                data-brand-picker
                size="lg"
                tone="primary"
                value={brandHex()}
                on:sail-change={onBrandColor}
              ></sail-color-picker>
              <column gap="1" grow min-w="0">
                <p sx={style.fontSize(sm).fontWeight(medium)}>Pick one brand color</p>
                <p sx={style.fontSize(xs).fg(muted-foreground)}>
                  The scale below updates live. Contrast failures are flagged before you save.
                </p>
              </column>
            </row>
            <row data-brand-ramp gap="2" align="center" nowrap sx={style.overflow(auto)}>
              <for {brandRamp()} as="s" key={s.key}>
                <column gap="1" align="center" sx={style.minW(10)}>
                  <center
                    data-swatch={s.hex}
                    data-ramp-step={s.key}
                    sx={style.w(10).h(8).radius(md).borderWidth(1).border(border-subtle)}
                    aria-hidden="true"
                  ></center>
                  <p sx={style.fontSize(xs).fg(muted-foreground)}>{s.label}</p>
                </column>
              </for>
            </row>
            <column data-contrast-flags gap="1">
              <p
                sx={style.fontSize(xs).fontWeight(medium)}
                data-contrast-summary
              >
                {contrastFailLabel()}
              </p>
              <for {contrastRows()} as="c" key={c.key}>
                <row data-contrast-id={c.id} gap="2" align="center" nowrap>
                  <badge size="sm" tone={c.tone}>{c.passAA ? 'AA' : 'fail'}</badge>
                  <p grow min-w="0" sx={style.fontSize(xs).truncate()}>{c.label}</p>
                  <p sx={style.fontSize(xs).fontFamily(mono).fg(muted-foreground)}>{c.ratio}</p>
                </row>
              </for>
            </column>
          </column>
        </if>

        <row
          align="center"
          gap="2"
          nowrap
          shrink="0"
          sx={style.h(7).padX(2).borderBWidth(1).borderB(border-subtle).minW(0)}
        >
          <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
            {activeGroupLabel()}
          </p>
          <p sx={style.fontSize(xs).fg(muted-foreground)}>{tokenCountLabel()}</p>
          <input
            type="search"
            data-token-search
            placeholder="Search all tokens…"
            value={tokenSearchVal()}
            on:input={onTokenSearch}
            grow
            size="sm"
          />
          <separator orientation="vertical" sx={style.h(4)} />
          <label sx={style.fontSize(xs).fg(muted-foreground)} data-overrides-only>
            <input type="checkbox" checked={overridesOnlyOn()} on:change={onOverridesOnly} />
            Changed only
          </label>
        </row>
        <p data-design-helper sx={style.fontSize(xs).fg(muted-foreground).padX(2).padB(1)}>
          Search any token, or show only values you've changed.
        </p>

        <if {!hasTokenRows()}>
          <empty-state variant="placeholder" size="sm" w="full" data-token-empty sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3)}>
            <strong slot="title">No tokens match</strong>
            Clear search or the changed-only filter.
          </empty-state>
        </if>

        <column gap="0" data-token-list on:click={onTokenRowActivate} sx={style.padY(1)}>
          <for {tokenRows()} as="tok" key={tok.key}>
            <row
              data-token-path={tok.path}
              data-token-group={tok.groupId}
              data-dirty={tok.dirty ? 'true' : 'false'}
              gap="2"
              align="center"
              nowrap
              sx={style.h(7).padX(3).minW(0).when({ hover: style.bg(hover-bg) })}
            >
              <if {tok.isColor}>
                <sail-color-picker
                  size="sm"
                  data-token-path={tok.path}
                  value={tok.pickerHex}
                  on:sail-change={onTokenColor}
                ></sail-color-picker>
              </if>
              <column gap="0" grow min-w="0">
                <p sx={style.fontSize(sm).truncate()}>{tok.label}</p>
                <p sx={style.fontSize(xs).fg(muted-foreground).truncate()}>{tok.display}</p>
              </column>
              <if {tok.showGroup}>
                <p sx={style.fontSize(xs).fg(muted-foreground)}>{tok.groupLabel}</p>
              </if>
              <if {tok.isColor}>
                <button
                  type="button"
                  size="sm"
                  variant="ghost"
                  data-color-edit
                  data-token-path={tok.path}
                  on:click={onColorEditToggle}
                >
                  Raw
                </button>
              </if>
              <if {tok.isDim}>
                <input
                  type="range"
                  min={tok.min}
                  max={tok.max}
                  step={tok.step}
                  value={tok.num}
                  data-token-path={tok.path}
                  data-token-unit={tok.unit}
                  on:input={onTokenNum}
                  aria-label={tok.label}
                />
                <p sx={style.fontSize(xs).fontFamily(mono).w(10).truncate()}>{tok.display}</p>
              </if>
              <if {tok.isText}>
                <input
                  type="text"
                  data-token-path={tok.path}
                  value={tok.value}
                  on:change={onTokenText}
                  aria-label={tok.label}
                  w="32"
                  size="sm"
                />
              </if>
              <if {tok.dirty}>
                <button
                  type="button"
                  size="sm"
                  variant="ghost"
                  data-token-reset
                  data-token-path={tok.path}
                  on:click={onTokenReset}
                >
                  Reset
                </button>
              </if>
            </row>
            <if {tok.colorEditOpen}>
              <row gap="2" align="center" data-color-raw-edit sx={style.padX(3).padB(1)}>
                <p sx={style.fontSize(xs).fg(muted-foreground)}>Raw value</p>
                <input
                  type="text"
                  data-token-path={tok.path}
                  value={tok.value}
                  on:change={onTokenText}
                  aria-label={`Raw value for ${tok.label}`}
                  grow
                  size="sm"
                />
              </row>
            </if>
          </for>
        </column>

        <if {hasDiff()}>
          <column
            data-design-diff
            gap="2"
            sx={style.fontSize(xs).pad(2).bg(surface-1).radius(sm)}
          >
            <strong>Changes</strong>
            <for {diffRows()} as="r" key={r.key}>
              <row data-diff-row gap="2" align="center" nowrap>
                <if {r.isColor}>
                  <center
                    data-swatch={r.toValue}
                    shrink="0"
                    sx={style.w(4).h(4).radius(sm).borderWidth(1).border(border-subtle)}
                    aria-hidden="true"
                  ></center>
                </if>
                <p grow min-w="0" sx={style.truncate()}>
                  {r.label}: {r.fromDisplay} → {r.toDisplay}
                </p>
              </row>
            </for>
          </column>
        </if>
      </column>
    </scroll-area>

    <design-inspector shrink="0" sx={style.w(64).minH(0).h(full)} />
  </row>

  <row
    data-design-status
    shrink="0"
    justify="between"
    align="center"
    nowrap
    sx={style.h(6).padX(3).borderTWidth(1).borderT(border-subtle).fontSize(xs).fg(muted-foreground)}
  >
    <p>{designStatusNote()}</p>
    <p>{designThemeLabel()} · {designOverridesLabel()}</p>
  </row>
</column>
