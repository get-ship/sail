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
  style surfaceRoot = { size(grow).overflow(hidden) }
  style shrink0 = { size(hug) }
  style growMin = { size(grow) }
  // no preset: size lock
  style filterW = { size({ w: 28 }) }
  style filterFull = { size(max) }
  // no preset: size lock
  style sepH = { size({ h: 4 }) }
  style mutedXs = { fg(muted).type(body-sm) }
  style errorCount = { fg(danger).type(body-sm) }
  style warnCount = { fg(warning).type(body-sm) }
  style errorText = { size(grow).fg(danger).type(body-sm) }
  // no preset: size lock + edge-only border
  style errorBand = { size({ h: 7 }).pad(3).border({ edge: b, color: border, width: 1 }) }
  // no preset: min-w lock + edge-only border
  style panelTools = { pad(3).size({ min-w: 0 }).border({ edge: b, color: border-subtle, width: 1 }).bg(surface) }
  style findingsScroll = { size(grow) }
  // no preset: size lock
  style groupHeader = { size({ h: 6 }).pad(3), when: { hover: bg(hover-bg) } }
  style ruleMono = { size(grow).fg(muted).type(mono-sm) }
  // no preset: size lock
  style findingTrigger = { size({ w: 100%, h: 6 }).pad(3), when: { hover: bg(hover-bg) } }
  style monoXs = { fg(muted).type(mono-sm) }
  style msgSm = { size(grow).type(body-sm) }
  style fixChip = { fg(primary).type(body-sm) }
  style detailPad = { pad(2) }
  // no preset: size lock
  style whyPre = { pad(2).size({ min-h: 0, max-h: 40 }).overflow(auto).bg(app).radius(sm).type(mono-sm) }
  style snippetBox = { pad(2).bg(app).radius(sm) }
  style snippetLine = { type(mono-sm) }
  // no preset: size lock + edge-only border
  style statusBar = { size({ h: 6 }).pad(3).border({ edge: t, color: border-subtle, width: 1 }).fg(muted) }
  // no preset: min-w lock + edge-only border
  style reviewSheet = { pad(3).size({ min-w: 0, max-h: 56 }).overflow(auto).border({ edge: t, color: border, width: 1 }).bg(surface) }
  style titleSm = { type(body-sm) }
  style fullW = { size(max) }
  // no preset: type wrap
  style chipLabel = { size(hug).type({ wrap: truncate }) }
</scope>

<controller>
import type { UiEvent } from '../compositions/ui-event'
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
    onRuleFilter: (e: UiEvent) => c.onRuleFilter(e),
    onFileFilter: (e: UiEvent) => c.onFileFilter(e),
    onSeverityFilter: (e: UiEvent) => c.onSeverityFilter(e),
    onFindingClick: (id: string) => c.onFindingClick(id),
    onFindingAction: (
      id: string,
      action: 'open-source' | 'select-in-app' | 'apply-fix',
      e?: UiEvent,
    ) => c.onFindingAction(id, action, e),
    onRuleToggle: (rule: string, e?: UiEvent) => c.onRuleToggle(rule, e),
    onApplyRule: (rule: string, e?: UiEvent) => c.onApplyRule(rule, e),
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
      <!-- no preset: min-w lock -->
      <row align="center" justify="between" nowrap gap="2" style({ size({ min-w: 0 }) })>
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
      <!-- no preset: type wrap -->
      <p last-run style(mutedXs, { type({ wrap: truncate }) })>{lastRunLabel()}</p>
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
    <column gap="0" style({ pad(1) })>
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
                          <badge both-viewports tone="muted" size="sm" variant="outline" style(chipLabel)>
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

  <row checks-status align="center" style(statusBar)>    <p style({ type(body-sm) })>      {checksStatusNote()}
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
            <!-- no preset: min-w lock -->
            <column review-diff gap="1" style({ size({ min-w: 0 }) })>
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
