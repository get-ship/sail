<!--
  <checks-surface> — findings inbox (createTable host).
  prop panel: compact drawer chrome (no full-page header / overflowing toolbar).
  Page mode: studio-page-header + full filter strip.
  Ban-clean.
-->
<scope>
  props: {
    /** true when hosted in shell findings drawer — compact layout only */
    panel = false
  }
  style surfaceRoot = { box: grow(1).minH(0).h(full).minW(0).overflow(hidden) }
  style shrink0 = { box: shrink(0) }
  style growMin = { box: grow(1).minW(0) }
  style filterW = { box: w(28) }
  style filterFull = { box: w(full).minW(0) }
  style sepH = { box: h(4) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style errorCount = { paint: fg(destructive), text: fontSize(xs) }
  style warnCount = { paint: fg(warning), text: fontSize(xs) }
  style errorText = { box: grow(1).minW(0), paint: fg(destructive), text: fontSize(xs).truncate() }
  style errorBand = { box: shrink(0).h(7).padX(3), paint: borderBWidth(1).borderB(border) }
  style panelTools = { box: shrink(0).padX(3).padY(2).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1) }
  style findingsScroll = { box: grow(1).minW(0).minH(0).h(full).basis(0) }
  style groupHeader = { box: h(6).padX(3), when: { hover: { paint: bg(hover-bg) } } }
  style ruleMono = { box: grow(1).minW(0), paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).fontFamily(mono).truncate() }
  style findingTrigger = { box: h(6).padX(3).w(full), when: { hover: { paint: bg(hover-bg) } } }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style msgSm = { box: grow(1).minW(0), text: fontSize(sm).truncate() }
  style fixChip = { paint: fg(primary), text: fontSize(xs) }
  style detailPad = { box: padX(2).padB(2) }
  style whyPre = { box: pad(2).minH(0).maxH(40).overflow(auto), paint: bg(surface-0).radius(sm), text: fontFamily(mono).fontSize(xs).whitespace(pre-wrap) }
  style snippetBox = { box: pad(2), paint: bg(surface-0).radius(sm) }
  style snippetLine = { text: fontFamily(mono).fontSize(xs) }
  style statusBar = { box: shrink(0).h(6).padX(3), paint: borderTWidth(1).borderT(border-subtle).fg(muted-foreground) }
  style reviewSheet = { box: shrink(0).pad(3).minW(0).maxH(56).overflow(auto), paint: borderTWidth(1).borderT(border).bg(surface-1) }
  style titleSm = { text: fontSize(sm).fontWeight(semibold) }
  style fullW = { box: w(full) }
</scope>

<controller>
import { createChecksSurface } from '../compositions/checks-surface/checks-surface-vm'
export default controller(() => {
  const c = createChecksSurface()
  return {
    ruleGroups: c.ruleGroups,
    hasFindings: c.hasFindings,
    emptyTitle: c.emptyTitle,
    emptyBody: c.emptyBody,
    emptyChecked: c.emptyChecked,
    hasChecksError: c.hasChecksError,
    checksError: c.checksError,
    errorCountLabel: c.errorCountLabel,
    warningCountLabel: c.warningCountLabel,
    checksStatusNote: c.checksStatusNote,
    ruleOpts: c.ruleOpts,
    fileOpts: c.fileOpts,
    severityOpts: c.severityOpts,
    ruleFilterVal: c.ruleFilterVal,
    fileFilterVal: c.fileFilterVal,
    severityFilterVal: c.severityFilterVal,
    hasFixable: c.hasFixable,
    fixableCountLabel: c.fixableCountLabel,
    reviewOpen: c.reviewOpen,
    reviewItems: c.reviewItems,
    applyFixesLabel: c.applyFixesLabel,
    canApplyFixes: c.canApplyFixes,
    autoFixOpts: c.autoFixOpts,
    autoFixModeLabel: c.autoFixModeLabel,
    lastRunLabel: c.lastRunLabel,
    recheckLabel: c.recheckLabel,
    showSuggestDiff: c.showSuggestDiff,
    onRuleFilter: (e: Event) => c.onRuleFilter(e),
    onFileFilter: (e: Event) => c.onFileFilter(e),
    onSeverityFilter: (e: Event) => c.onSeverityFilter(e),
    onFindingClick: (id: string) => c.onFindingClick(id),
    onFindingAction: (
      id: string,
      action: 'open-source' | 'select-in-app' | 'apply-fix',
      e?: Event,
    ) => c.onFindingAction(id, action, e),
    onRuleToggle: (rule: string, e?: Event) => c.onRuleToggle(rule, e),
    onApplyRule: (rule: string, e?: Event) => c.onApplyRule(rule, e),
    onRetryChecks: () => c.onRetryChecks(),
    onRecheck: () => c.onRecheck(),
    onAutoFixMode: (mode: 'off' | 'suggest' | 'apply') => c.onAutoFixMode(mode),
    onOpenReview: () => c.onOpenReview(),
    onCloseReview: () => c.onCloseReview(),
    onReviewToggle: (id: string) => c.onReviewToggle(id),
    onApplyReview: () => c.onApplyReview(),
  }
})
</controller>

<column surface="checks" checks-surface checks-panel-mode={panel() ? 'true' : 'false'} gap="0" style(surfaceRoot)>
  <!-- Full-page chrome only (not drawer) -->
  <if {!panel()}>
    <studio-page-header checks-header title="Findings">
      <p error-count style(errorCount)>{errorCountLabel()}</p>
      <p warning-count style(warnCount)>{warningCountLabel()}</p>
      <p last-run style(mutedXs)>{lastRunLabel()}</p>
      <row slot="actions" gap="2" align="center" nowrap style(shrink0)>
        <button type="button" size="sm" variant="ghost" tone="muted" recheck on:click={onRecheck}>
          {recheckLabel()}
        </button>
        <if {hasFixable()}>
          <button type="button" size="sm" review-fixes tone="primary" on:click={onOpenReview}>
            {fixableCountLabel()}
          </button>
        </if>
      </row>
    </studio-page-header>

    <studio-toolbar checks-toolbar>
      <select filter-rule label="Filter by rule" value={ruleFilterVal()} on:change={onRuleFilter} style(filterW)>
        <for {ruleOpts() as opt} key={opt.key}>
          <option value={opt.value}>{opt.label}</option>
        </for>
      </select>
      <separator orientation="vertical" style(sepH)>      </separator>
      <select filter-file label="Filter by file" value={fileFilterVal()} on:change={onFileFilter} style(filterW)>
        <for {fileOpts() as opt} key={opt.key}>
          <option value={opt.value}>{opt.label}</option>
        </for>
      </select>
      <separator orientation="vertical" style(sepH)>      </separator>
      <select filter-severity label="Filter by severity" value={severityFilterVal()} on:change={onSeverityFilter} style(filterW)>
        <for {severityOpts() as opt} key={opt.key}>
          <option value={opt.value}>{opt.label}</option>
        </for>
      </select>
      <separator orientation="vertical" style(sepH)>      </separator>
      <row autofix-control align="center" gap="1" nowrap label={autoFixModeLabel()}>
        <p style(mutedXs)>Autofix</p>
        <for {autoFixOpts() as m} key={m.key}>
          <button
            type="button"
            size="sm"
            autofix-mode={m.mode}
            active={m.active ? 'true' : 'false'}
            variant={m.active ? 'soft' : 'ghost'}
            tone={m.active ? 'primary' : 'muted'}
            on:click={() => onAutoFixMode(m.mode)}
          >
            {m.label}
          </button>
        </for>
      </row>
    </studio-toolbar>
  </if>

  <!-- Drawer: one dense tools strip — no dual headers, no overflowing selects -->
  <if {panel()}>
    <column findings-panel-tools gap="2" style(panelTools)>
      <row align="center" justify="between" nowrap gap="2" style({ box: minW(0) })>
        <row align="center" gap="2" nowrap style(growMin)>
          <badge error-count tone="destructive" size="sm" variant="soft" pill>{errorCountLabel()}</badge>
          <badge warning-count tone="warning" size="sm" variant="soft" pill>{warningCountLabel()}</badge>
        </row>
        <button type="button" size="sm" variant="ghost" tone="muted" recheck on:click={onRecheck} style(shrink0)>
          {recheckLabel()}
        </button>
      </row>
      <select
        filter-severity
        label="Filter by severity"
        value={severityFilterVal()}
        on:change={onSeverityFilter}
        size="sm" style(filterFull)>
        <for {severityOpts() as opt} key={opt.key}>
          <option value={opt.value}>{opt.label}</option>
        </for>
      </select>
      <p last-run style(mutedXs, { text: truncate() })>{lastRunLabel()}</p>
    </column>
  </if>

  <if {hasChecksError()}>
    <row checks-error align="center" gap="2" style(errorBand)>
      <p style(errorText)>{checksError()}</p>
      <button type="button" size="sm" variant="ghost" retry-checks on:click={onRetryChecks}>
        Retry
      </button>
    </row>
  </if>

  <scroll-area findings-list findings-table findings-table-host style(findingsScroll)>
    <column gap="0" style({ box: padY(1) })>
      <if {!hasFindings()}>
        <if {!hasChecksError()}>
          <studio-empty
            findings-empty
            checks-empty
            title={emptyTitle()}
          >
            {emptyBody()}
            <p empty-checked style(mutedXs)>{emptyChecked()}</p>
          </studio-empty>
        </if>
      </if>
      <if {hasFindings()}>
        <!-- Page mode only: ceremonial column headers (list is rule groups below) -->
        <if {!panel()}>
          <table findings-grid hoverable style(fullW)>
            <thead>
              <tr>
                <th>Severity</th>
                <th>Rule</th>
                <th>Span</th>
                <th>Summary</th>
              </tr>
            </thead>
          </table>
        </if>
        <for {ruleGroups() as g} key={g.key}>
          <column gap="0" rule-group={g.rule} severity-group="rule">
            <row
              align="center"
              gap="2"
              nowrap
              rule-header={g.rule}
              group-row-id={g.groupRowId} style(groupHeader)>
              <button
                type="button"
                size="sm"
                variant="ghost"
                tone="muted"
                rule-toggle={g.rule}
                group-row-id={g.groupRowId}
                expanded={g.open ? 'true' : 'false'}
                on:click={(e) => onRuleToggle(g.rule, e)}
              >
                {g.open ? '▾' : '▸'}
              </button>
              <p style(ruleMono)>
                {g.rule}
              </p>
              <if {g.showErrorBadge}>
                <badge tone="destructive" size="sm" variant="soft" rule-errors>
                  {g.errorBadge}
                </badge>
              </if>
              <if {g.showWarningBadge}>
                <badge tone="warning" size="sm" variant="soft" rule-warnings>
                  {g.warningBadge}
                </badge>
              </if>
              <if {g.showInfoBadge}>
                <badge tone="muted" size="sm" variant="soft" rule-info>
                  {g.infoBadge}
                </badge>
              </if>
              <p style(mutedXs)>{g.countLabel}</p>
              <if {g.showApplyAll}>
                <button
                  type="button"
                  size="sm"
                  variant="soft"
                  tone="primary"
                  apply-rule={g.rule}
                  on:click={(e) => onApplyRule(g.rule, e)}
                >
                  {g.applyAllLabel}
                </button>
              </if>
            </row>
            <for {g.rows as f} key={f.key}>
              <accordion
                open={f.expanded}
                selected={f.selected ? 'true' : 'false'}
                severity={f.severity}
                rule={f.rule}
                on:click={() => onFindingClick(f.id)}
              >
                <row
                  slot="trigger"
                  gap="2"
                  align="center"
                  nowrap style(findingTrigger)>
                  <badge
                    severity-pill
                    tone={f.severityTone}
                    size="sm"
                    variant="soft"
                  >
                    {f.severityLabel}
                  </badge>
                  <p
                    row-rule style(monoXs)>
                    {f.rule}
                  </p>
                  <p
                    file-span style(monoXs)>
                    {f.fileSpan}
                  </p>
                  <p style(msgSm)>{f.message}</p>
                  <if {f.showFixChip}>
                    <p fix-chip style(fixChip)>fix</p>
                  </if>
                </row>
                <column gap="2" style(detailPad)>
                  <card finding-detail variant="outline" padding="sm">
                    <column gap="2">
                      <p style(mutedXs)>Why</p>
                      <pre
                        why-text style(whyPre)>
                        {f.whyText}
                      </pre>
                      <if {f.showDiff}>
                        <column gap="1" why-suggest>
                          <p style(mutedXs)>Suggestion</p>
                          <sail-diff
                            suggest-diff
                            from={f.diffFrom}
                            to={f.diffTo}
                          ></sail-diff>
                        </column>
                      </if>
                      <column
                        why-snippet
                        gap="0" style(snippetBox)>                        <for {f.snippet as ln} key={ln.key}>
                          <p
                            snippet-line={ln.n}
                            highlight={ln.highlight ? 'true' : 'false'} style(snippetLine)>                            {ln.n} {ln.text}
                          </p>
                        </for>
</column>
                      <row gap="2" align="center" nowrap>
                        <button
                          type="button"
                          size="sm"
                          finding-action="open-source"
                          open-source-url={f.openSourceUrl}
                          deeplink-url={f.openSourceUrl}
                          variant="soft"
                          tone="primary"
                          on:click={(e) => onFindingAction(f.id, 'open-source', e)}
                        >
                          Open in Edit
                        </button>
                        <button
                          type="button"
                          size="sm"
                          finding-action="select-in-app"
                          variant="soft"
                          tone="muted"
                          on:click={(e) => onFindingAction(f.id, 'select-in-app', e)}
                        >
                          Select element
                        </button>
                        <if {f.showBothViewports}>
                          <badge both-viewports tone="muted" size="sm" variant="outline">
                            Mobile + desktop
                          </badge>
                        </if>
                        <if {f.showApply}>
                          <button
                            type="button"
                            size="sm"
                            finding-action="apply-fix"
                            variant="soft"
                            tone="primary"
                            on:click={(e) => onFindingAction(f.id, 'apply-fix', e)}
                          >
                            Apply fix
                          </button>
                        </if>
                      </row>
                    </column>
                  </card>
                </column>
              </accordion>
            </for>
          </column>
        </for>
      </if>
    </column>
  </scroll-area>

  <row checks-status align="center" style(statusBar)>    <p style({ text: fontSize(xs) })>      {checksStatusNote()}
    </p>
</row>

  <!-- Inline review sheet: L1 dialog overflow:hidden + maxH clips tall content. -->
  <if {reviewOpen()}>
    <column review-dialog review-sheet gap="2" style(reviewSheet)>
      <row align="center" justify="between" nowrap gap="2">
        <p style(titleSm)>Review fixes</p>
        <button type="button" size="sm" variant="ghost" tone="muted" on:click={onCloseReview}>
          Cancel
        </button>
      </row>
      <p style(mutedXs)>
        Accept or skip each automatic fix, then apply.
      </p>
      <if {!reviewItems().length}>
        <p review-empty style(mutedXs)>
          No automatic fixes — only compiler rewrites appear here.
        </p>
      </if>
      <for {reviewItems() as item} key={item.key}>
        <card review-item={item.findingId} variant="outline" padding="sm">
          <column gap="2">
            <row gap="2" align="center" nowrap>
              <badge tone="primary" size="sm" variant="soft">{item.rule}</badge>
              <p style(msgSm)>{item.message}</p>
            </row>
            <column review-diff gap="1" style({ box: minW(0) })>
              <sail-diff from={item.before} to={item.after}></sail-diff>
            </column>
            <row gap="2">
              <button
                type="button"
                size="sm"
                review-id={item.findingId}
                accepted={item.accepted ? 'true' : 'false'}
                variant={item.accepted ? 'soft' : 'ghost'}
                tone={item.accepted ? 'primary' : 'muted'}
                on:click={() => onReviewToggle(item.findingId)}
              >
                {item.acceptLabel}
              </button>
            </row>
          </column>
        </card>
      </for>
      <row gap="2" justify="end" nowrap>
        <button
          type="button"
          size="sm"
          apply-review
          tone="primary"
          disabled={!canApplyFixes()}
          on:click={onApplyReview}
        >
          {applyFixesLabel()}
        </button>
      </row>
    </column>
  </if>
</column>
