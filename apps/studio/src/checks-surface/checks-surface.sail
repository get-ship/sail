<!--
  <checks-surface> — compiler findings, triaged to done.
  Severity groups · expandable suggestion · Review fixes dialog.
  Ban-clean: no div/span; layout keywords + L1 only.
-->
<controller>
import { createChecksSurface } from './checks-surface-vm'
export default controller(() => {
  const vm = createChecksSurface()
  return {
    severityGroups: vm.severityGroups,
    hasFindings: vm.hasFindings,
    emptyTitle: vm.emptyTitle,
    emptyBody: vm.emptyBody,
    hasChecksError: vm.hasChecksError,
    checksError: vm.checksError,
    errorCountLabel: vm.errorCountLabel,
    warningCountLabel: vm.warningCountLabel,
    checksStatusNote: vm.checksStatusNote,
    ruleOpts: vm.ruleOpts,
    fileOpts: vm.fileOpts,
    ruleFilterVal: vm.ruleFilterVal,
    fileFilterVal: vm.fileFilterVal,
    hasFixable: vm.hasFixable,
    fixableCountLabel: vm.fixableCountLabel,
    reviewOpen: vm.reviewOpen,
    reviewItems: vm.reviewItems,
    applyFixesLabel: vm.applyFixesLabel,
    canApplyFixes: vm.canApplyFixes,
    onRuleFilter: (e: Event) => vm.onRuleFilter(e),
    onFileFilter: (e: Event) => vm.onFileFilter(e),
    onFindingClick: (e: Event) => vm.onFindingClick(e),
    onFindingAction: (e: Event) => vm.onFindingAction(e),
    onRetryChecks: () => vm.onRetryChecks(),
    onOpenReview: () => vm.onOpenReview(),
    onCloseReview: () => vm.onCloseReview(),
    onReviewToggle: (e: Event) => vm.onReviewToggle(e),
    onApplyReview: () => vm.onApplyReview(),
  }
})
</controller>

<column
  data-surface="checks"
  data-checks-surface
  gap="0"
  grow
  sx={style.minH(0).h(full).minW(0).overflow(hidden)}
>
  <row
    data-checks-header
    align="center"
    gap="3"
    shrink="0"
    nowrap
    justify="between"
    sx={style.h(7).padX(3).borderBWidth(1).borderB(border-subtle).bg(surface-1)}
  >
    <row align="center" gap="2" nowrap grow min-w="0">
      <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
        Checks
      </p>
      <p data-error-count sx={style.fontSize(xs).fg(destructive)}>{errorCountLabel()}</p>
      <p data-warning-count sx={style.fontSize(xs).fg(warning)}>{warningCountLabel()}</p>
    </row>
    <row gap="2" align="center" nowrap shrink="0">
      <if {hasFixable()}>
        <button
          type="button"
          size="sm"
          data-review-fixes
          tone="primary"
          on:click={onOpenReview}
        >
          {fixableCountLabel()}
        </button>
      </if>
    </row>
  </row>

  <row
    data-checks-toolbar
    align="center"
    gap="2"
    shrink="0"
    nowrap
    sx={style.h(7).padX(2).borderBWidth(1).borderB(border-subtle).bg(surface-1)}
  >
    <select
      data-filter-rule
      aria-label="Filter by rule"
      value={ruleFilterVal()}
      on:change={onRuleFilter}
      w="28"
    >
      <for {ruleOpts()} as="opt" key={opt.key}>
        <option value={opt.value}>{opt.label}</option>
      </for>
    </select>
    <separator orientation="vertical" sx={style.h(4)} />
    <select
      data-filter-file
      aria-label="Filter by file"
      value={fileFilterVal()}
      on:change={onFileFilter}
      w="28"
    >
      <for {fileOpts()} as="opt" key={opt.key}>
        <option value={opt.value}>{opt.label}</option>
      </for>
    </select>
    <p sx={style.fontSize(xs).fg(muted-foreground)}>Filter findings by rule or file.</p>
  </row>

  <if {hasChecksError()}>
    <row
      data-checks-error
      align="center"
      gap="2"
      shrink="0"
      sx={style.h(7).padX(3).borderBWidth(1).borderB(border)}
    >
      <p grow min-w="0" sx={style.fontSize(xs).fg(destructive).truncate()}>{checksError()}</p>
      <button type="button" size="sm" variant="ghost" data-retry-checks on:click={onRetryChecks}>
        Retry
      </button>
    </row>
  </if>

  <scroll-area
    data-findings-list
    grow
    min-w="0"
    on:click={onFindingClick}
    sx={style.minH(0).h(full).basis(0)}
  >
    <column gap="2" sx={style.padY(1)}>
      <if {!hasFindings()}>
        <if {!hasChecksError()}>
          <empty-state data-findings-empty variant="placeholder" size="sm" w="full" sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3)}>
            <strong slot="title">{emptyTitle()}</strong>
            {emptyBody()}
          </empty-state>
        </if>
      </if>
      <for {severityGroups()} as="g" key={g.key}>
        <column gap="0" data-severity-group={g.severity}>
          <row align="center" gap="2" nowrap sx={style.h(6).padX(3)}>
            <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
              {g.label}
            </p>
            <p sx={style.fontSize(xs).fg(muted-foreground)}>{g.countLabel}</p>
          </row>
          <for {g.rows} as="f" key={f.key}>
            <collapsible
              open={f.expanded}
              data-finding-id={f.id}
              data-selected={f.selected ? 'true' : 'false'}
              data-severity={f.severity}
              data-rule={f.rule}
            >
              <row
                slot="trigger"
                gap="2"
                align="center"
                nowrap
                sx={style.h(6).padX(3).w(full).when({ hover: style.bg(hover-bg) })}
              >
                <p data-severity-pill sx={style.fontSize(xs).fg(muted-foreground).fontFamily(mono)}>
                  {f.rule}
                </p>
                <p grow min-w="0" sx={style.fontSize(sm).truncate()}>{f.message}</p>
                <if {f.showFixChip}>
                  <p data-fix-chip sx={style.fontSize(xs).fg(primary)}>fix</p>
                </if>
                <p
                  data-file-span
                  sx={style.fontSize(xs).fg(muted-foreground).fontFamily(mono)}
                >
                  {f.fileSpan}
                </p>
              </row>
              <column
                gap="2"
                sx={style.padX(2).padB(2)}
                on:click={onFindingAction}
              >
                <pre
                  data-why-text
                  sx={style.fontFamily(mono).fontSize(xs).bg(surface-0).pad(2).radius(sm).overflow(auto).whitespace(pre-wrap)}
                >
                  {f.whyText}
                </pre>
                <if {f.hasSuggest}>
                  <column gap="1" data-why-suggest>
                    <p sx={style.fontSize(xs).fg(muted-foreground)}>Suggestion</p>
                    <p sx={style.fontSize(sm)}>{f.suggestText}</p>
                  </column>
                </if>
                <column
                  data-why-snippet
                  gap="0"
                  sx={style.fontFamily(mono).fontSize(xs).bg(surface-0).pad(2).radius(sm)}
                >
                  <for {f.snippet} as="ln" key={ln.key}>
                    <p
                      data-snippet-line={ln.n}
                      data-highlight={ln.highlight ? 'true' : 'false'}
                    >
                      {ln.n} {ln.text}
                    </p>
                  </for>
                </column>
                <row gap="2" align="center" nowrap>
                  <button
                    type="button"
                    size="sm"
                    data-finding-action="open-source"
                    data-open-source-url={f.openSourceUrl}
                    data-deeplink-url={f.openSourceUrl}
                    variant="soft"
                    tone="muted"
                  >
                    Open source
                  </button>
                  <button
                    type="button"
                    size="sm"
                    data-finding-action="select-in-app"
                    variant="soft"
                    tone="muted"
                  >
                    Select in app
                  </button>
                  <if {f.showBothViewports}>
                    <badge data-both-viewports tone="muted" size="sm" variant="outline">
                      Mobile + desktop
                    </badge>
                  </if>
                  <if {f.showApply}>
                    <button
                      type="button"
                      size="sm"
                      data-finding-action="apply-fix"
                      variant="soft"
                      tone="primary"
                    >
                      Apply fix
                    </button>
                  </if>
                </row>
              </column>
            </collapsible>
          </for>
        </column>
      </for>
    </column>
  </scroll-area>

  <row
    data-checks-status
    shrink="0"
    align="center"
    sx={style.h(6).padX(3).borderTWidth(1).borderT(border-subtle).fontSize(xs).fg(muted-foreground)}
  >
    <p>{checksStatusNote()}</p>
  </row>

  <!-- Definite dialog height so section maxH fits (taffy in-flow was ~200px otherwise). -->
  <dialog data-review-dialog open={reviewOpen()} size="lg" sx={style.h(panel-max-h-md).maxH(panel-vh-md).overflow(hidden)}>
    <header>
      <strong>Review fixes</strong>
    </header>
    <section sx={style.minH(0).maxH(panel-max-h-sm).overflow(auto)}>
      <column gap="3" sx={style.padY(1).minH(0)}>
        <p sx={style.fontSize(sm).fg(muted-foreground)}>
          Accept or skip each automatic fix, then apply the ones you want.
        </p>
        <if {!reviewItems().length}>
          <empty-state variant="inline" size="sm" w="full">
            <strong slot="title">No automatic fixes</strong>
            Only fixes the compiler can rewrite in source appear here.
          </empty-state>
        </if>
        <for {reviewItems()} as="item" key={item.key}>
          <card
            data-review-item={item.findingId}
            variant="outline"
            padding="sm"
          >
            <column gap="2">
              <row gap="2" align="center" nowrap>
                <badge tone="primary" size="sm" variant="soft">{item.rule}</badge>
                <p grow min-w="0" sx={style.fontSize(sm)}>{item.message}</p>
                <p sx={style.fontSize(xs).fg(muted-foreground).fontFamily(mono)}>
                  {item.fileSpan}
                </p>
              </row>
              <column
                data-review-diff
                gap="1"
                sx={style.fontFamily(mono).fontSize(xs).bg(surface-0).pad(2).radius(sm)}
              >
                <p data-diff-before sx={style.fg(destructive)}>− {item.before}</p>
                <p data-diff-after sx={style.fg(primary)}>+ {item.after}</p>
              </column>
              <row gap="2" on:click={onReviewToggle}>
                <button
                  type="button"
                  size="sm"
                  data-review-id={item.findingId}
                  data-accepted={item.accepted ? 'true' : 'false'}
                  variant={item.accepted ? 'soft' : 'ghost'}
                  tone={item.accepted ? 'primary' : 'muted'}
                >
                  {item.acceptLabel}
                </button>
                <p sx={style.fontSize(xs).fg(muted-foreground)}>
                  {item.accepted ? 'Will apply' : 'Skipped'}
                </p>
              </row>
            </column>
          </card>
        </for>
      </column>
    </section>
    <footer>
      <row gap="2" justify="end" nowrap>
        <button type="button" size="sm" variant="ghost" tone="muted" on:click={onCloseReview}>
          Cancel
        </button>
        <button
          type="button"
          size="sm"
          data-apply-review
          tone="primary"
          disabled={!canApplyFixes()}
          on:click={onApplyReview}
        >
          {applyFixesLabel()}
        </button>
      </row>
    </footer>
  </dialog>
</column>
