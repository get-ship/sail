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
</scope>

<controller>
import { createChecksSurface } from './checks-surface-vm'
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
    onFindingClick: (e: Event) => c.onFindingClick(e),
    onFindingAction: (e: Event) => c.onFindingAction(e),
    onRuleToggle: (e: Event) => c.onRuleToggle(e),
    onApplyRule: (e: Event) => c.onApplyRule(e),
    onRetryChecks: () => c.onRetryChecks(),
    onRecheck: () => c.onRecheck(),
    onAutoFixMode: (e: Event) => c.onAutoFixMode(e),
    onOpenReview: () => c.onOpenReview(),
    onCloseReview: () => c.onCloseReview(),
    onReviewToggle: (e: Event) => c.onReviewToggle(e),
    onApplyReview: () => c.onApplyReview(),
  }
})
</controller>

<column
  surface="checks"
  checks-surface
  checks-panel-mode={panel() ? 'true' : 'false'}
  gap="0"
  grow
  sx={style.minH(0).h(full).minW(0).overflow(hidden)}
>
  <!-- Full-page chrome only (not drawer) -->
  <if {!panel()}>
    <studio-page-header checks-header title="Findings">
      <p error-count sx={style.fontSize(xs).fg(destructive)}>{errorCountLabel()}</p>
      <p warning-count sx={style.fontSize(xs).fg(warning)}>{warningCountLabel()}</p>
      <p last-run sx={style.fontSize(xs).fg(muted-foreground)}>{lastRunLabel()}</p>
      <row slot="actions" gap="2" align="center" nowrap shrink="0">
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
      <select filter-rule aria-label="Filter by rule" value={ruleFilterVal()} on:change={onRuleFilter} w="28">
        <for {ruleOpts()} as="opt" key={opt.key}>
          <option value={opt.value}>{opt.label}</option>
        </for>
      </select>
      <separator orientation="vertical" sx={style.h(4)} />
      <select filter-file aria-label="Filter by file" value={fileFilterVal()} on:change={onFileFilter} w="28">
        <for {fileOpts()} as="opt" key={opt.key}>
          <option value={opt.value}>{opt.label}</option>
        </for>
      </select>
      <separator orientation="vertical" sx={style.h(4)} />
      <select filter-severity aria-label="Filter by severity" value={severityFilterVal()} on:change={onSeverityFilter} w="28">
        <for {severityOpts()} as="opt" key={opt.key}>
          <option value={opt.value}>{opt.label}</option>
        </for>
      </select>
      <separator orientation="vertical" sx={style.h(4)} />
      <row autofix-control align="center" gap="1" nowrap on:click={onAutoFixMode} aria-label={autoFixModeLabel()}>
        <p sx={style.fontSize(xs).fg(muted-foreground)}>Autofix</p>
        <for {autoFixOpts()} as="m" key={m.key}>
          <button
            type="button"
            size="sm"
            autofix-mode={m.mode}
            active={m.active ? 'true' : 'false'}
            variant={m.active ? 'soft' : 'ghost'}
            tone={m.active ? 'primary' : 'muted'}
          >
            {m.label}
          </button>
        </for>
      </row>
    </studio-toolbar>
  </if>

  <!-- Drawer: one dense tools strip — no dual headers, no overflowing selects -->
  <if {panel()}>
    <column
      findings-panel-tools
      shrink="0"
      gap="2"
      sx={style.padX(3).padY(2).borderBWidth(1).borderB(border-subtle).bg(surface-1).minW(0)}
    >
      <row align="center" justify="between" nowrap gap="2" min-w="0">
        <row align="center" gap="2" nowrap grow min-w="0">
          <badge error-count tone="destructive" size="sm" variant="soft" pill>{errorCountLabel()}</badge>
          <badge warning-count tone="warning" size="sm" variant="soft" pill>{warningCountLabel()}</badge>
        </row>
        <button type="button" size="sm" variant="ghost" tone="muted" recheck on:click={onRecheck} shrink="0">
          {recheckLabel()}
        </button>
      </row>
      <select
        filter-severity
        aria-label="Filter by severity"
        value={severityFilterVal()}
        on:change={onSeverityFilter}
        size="sm"
        sx={style.w(full).minW(0)}
      >
        <for {severityOpts()} as="opt" key={opt.key}>
          <option value={opt.value}>{opt.label}</option>
        </for>
      </select>
      <p last-run sx={style.fontSize(xs).fg(muted-foreground).truncate()}>{lastRunLabel()}</p>
    </column>
  </if>

  <if {hasChecksError()}>
    <row
      checks-error
      align="center"
      gap="2"
      shrink="0"
      sx={style.h(7).padX(3).borderBWidth(1).borderB(border)}
    >
      <p grow min-w="0" sx={style.fontSize(xs).fg(destructive).truncate()}>{checksError()}</p>
      <button type="button" size="sm" variant="ghost" retry-checks on:click={onRetryChecks}>
        Retry
      </button>
    </row>
  </if>

  <!--
    Findings host: createTable identity assigned to host.table (assignTableHost).
    Columns severity · rule · file:span · summary; groups from table.rows().
  -->
  <scroll-area
    findings-list
    findings-table
    findings-table-host
    grow
    min-w="0"
    on:click={onFindingClick}
    sx={style.minH(0).h(full).basis(0)}
  >
    <column gap="0" sx={style.padY(1)}>
      <if {!hasFindings()}>
        <if {!hasChecksError()}>
          <studio-empty
            findings-empty
            checks-empty
            title={emptyTitle()}
          >
            {emptyBody()}
            <p empty-checked sx={style.fontSize(xs).fg(muted-foreground)}>{emptyChecked()}</p>
          </studio-empty>
        </if>
      </if>
      <if {hasFindings()}>
        <!-- Page mode only: ceremonial column headers (list is rule groups below) -->
        <if {!panel()}>
          <table findings-grid hoverable sx={style.w(full)}>
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
        <for {ruleGroups()} as="g" key={g.key}>
          <column gap="0" rule-group={g.rule} severity-group="rule">
            <row
              align="center"
              gap="2"
              nowrap
              rule-header={g.rule}
              group-row-id={g.groupRowId}
              sx={style.h(6).padX(3).when({ hover: style.bg(hover-bg) })}
            >
              <button
                type="button"
                size="sm"
                variant="ghost"
                tone="muted"
                rule-toggle={g.rule}
                group-row-id={g.groupRowId}
                aria-expanded={g.open ? 'true' : 'false'}
                on:click={onRuleToggle}
              >
                {g.open ? '▾' : '▸'}
              </button>
              <p
                grow
                min-w="0"
                sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).fontFamily(mono).truncate()}
              >
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
              <p sx={style.fontSize(xs).fg(muted-foreground)}>{g.countLabel}</p>
              <if {g.showApplyAll}>
                <button
                  type="button"
                  size="sm"
                  variant="soft"
                  tone="primary"
                  apply-rule={g.rule}
                  on:click={onApplyRule}
                >
                  {g.applyAllLabel}
                </button>
              </if>
            </row>
            <for {g.rows} as="f" key={f.key}>
              <collapsible
                open={f.expanded}
                finding-id={f.id}
                selected={f.selected ? 'true' : 'false'}
                severity={f.severity}
                rule={f.rule}
              >
                <row
                  slot="trigger"
                  gap="2"
                  align="center"
                  nowrap
                  sx={style.h(6).padX(3).w(full).when({ hover: style.bg(hover-bg) })}
                >
                  <badge
                    severity-pill
                    tone={f.severityTone}
                    size="sm"
                    variant="soft"
                  >
                    {f.severityLabel}
                  </badge>
                  <p
                    row-rule
                    sx={style.fontSize(xs).fg(muted-foreground).fontFamily(mono)}
                  >
                    {f.rule}
                  </p>
                  <p
                    file-span
                    sx={style.fontSize(xs).fg(muted-foreground).fontFamily(mono)}
                  >
                    {f.fileSpan}
                  </p>
                  <p grow min-w="0" sx={style.fontSize(sm).truncate()}>{f.message}</p>
                  <if {f.showFixChip}>
                    <p fix-chip sx={style.fontSize(xs).fg(primary)}>fix</p>
                  </if>
                </row>
                <column
                  gap="2"
                  sx={style.padX(2).padB(2)}
                  on:click={onFindingAction}
                >
                  <card finding-detail variant="outline" padding="sm">
                    <column gap="2">
                      <p sx={style.fontSize(xs).fg(muted-foreground)}>Why</p>
                      <pre
                        why-text
                        sx={style.fontFamily(mono).fontSize(xs).bg(surface-0).pad(2).radius(sm).minH(0).maxH(40).overflow(auto).whitespace(pre-wrap)}
                      >
                        {f.whyText}
                      </pre>
                      <if {f.showDiff}>
                        <column gap="1" why-suggest>
                          <p sx={style.fontSize(xs).fg(muted-foreground)}>Suggestion</p>
                          <sail-diff
                            suggest-diff
                            from={f.diffFrom}
                            to={f.diffTo}
                          ></sail-diff>
                        </column>
                      </if>
                      <column
                        why-snippet
                        gap="0"
                        sx={style.fontFamily(mono).fontSize(xs).bg(surface-0).pad(2).radius(sm)}
                      >
                        <for {f.snippet} as="ln" key={ln.key}>
                          <p
                            snippet-line={ln.n}
                            highlight={ln.highlight ? 'true' : 'false'}
                          >
                            {ln.n} {ln.text}
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
                        >
                          Open in Edit
                        </button>
                        <button
                          type="button"
                          size="sm"
                          finding-action="select-in-app"
                          variant="soft"
                          tone="muted"
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
                          >
                            Apply fix
                          </button>
                        </if>
                      </row>
                    </column>
                  </card>
                </column>
              </collapsible>
            </for>
          </column>
        </for>
      </if>
    </column>
  </scroll-area>

  <row
    checks-status
    shrink="0"
    align="center"
    sx={style.h(6).padX(3).borderTWidth(1).borderT(border-subtle).fontSize(xs).fg(muted-foreground)}
  >
    <p>{checksStatusNote()}</p>
  </row>

  <!--
    Review sheet (not L1 dialog): dialog ships overflow:hidden + maxH and
    sail-layout hard-fails content taller than the closed measure. Inline sheet
    keeps the same markers for tests/apply flow.
  -->
  <if {reviewOpen()}>
    <column
      review-dialog
      review-sheet
      gap="2"
      shrink="0"
      sx={style.borderTWidth(1).borderT(border).bg(surface-1).pad(3).minW(0).maxH(56).overflow(auto)}
    >
      <row align="center" justify="between" nowrap gap="2">
        <p sx={style.fontSize(sm).fontWeight(semibold)}>Review fixes</p>
        <button type="button" size="sm" variant="ghost" tone="muted" on:click={onCloseReview}>
          Cancel
        </button>
      </row>
      <p sx={style.fontSize(xs).fg(muted-foreground)}>
        Accept or skip each automatic fix, then apply.
      </p>
      <if {!reviewItems().length}>
        <p review-empty sx={style.fontSize(xs).fg(muted-foreground)}>
          No automatic fixes — only compiler rewrites appear here.
        </p>
      </if>
      <for {reviewItems()} as="item" key={item.key}>
        <card review-item={item.findingId} variant="outline" padding="sm">
          <column gap="2">
            <row gap="2" align="center" nowrap>
              <badge tone="primary" size="sm" variant="soft">{item.rule}</badge>
              <p grow min-w="0" sx={style.fontSize(sm).truncate()}>{item.message}</p>
            </row>
            <column review-diff gap="1" sx={style.minW(0)}>
              <sail-diff from={item.before} to={item.after}></sail-diff>
            </column>
            <row gap="2" on:click={onReviewToggle}>
              <button
                type="button"
                size="sm"
                review-id={item.findingId}
                accepted={item.accepted ? 'true' : 'false'}
                variant={item.accepted ? 'soft' : 'ghost'}
                tone={item.accepted ? 'primary' : 'muted'}
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
