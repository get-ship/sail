<!--
  <sail-studio-chrome> — shell: activity rail + status + route slot.
  Rail: Edit · Components · Themes · Tasks · Config (Checks = status panel).
  Selected activity uses primary color (fg + soft fill) — the only accent.
  Tooltips placement=right; rail overflow visible so bubbles paint.
  Exit matches activity item geometry. Ban-clean: layout + L1 only.
-->
<controller>
import { createShellPage } from '../shell/shell-page-vm'
export default controller((ctx) => {
  // Live pathname from deck layout injection — chrome is reused across leaf
  // navigations; popstate alone does not fire for SPA pushState.
  const routePath = () => {
    const r = (ctx as { route?: { pathname?: () => string } }).route
    if (r?.pathname) return r.pathname()
    return typeof location !== 'undefined' ? location.pathname : '/edit'
  }
  const vm = createShellPage({ pathname: routePath })
  return {
    rail: vm.rail,
    statusLine: vm.statusLine,
    themeStatusLabel: vm.themeStatusLabel,
    saveStatusLabel: vm.saveStatusLabel,
    hasBatch: vm.hasBatch,
    batchCountLabel: vm.batchCountLabel,
    batchChips: vm.batchChips,
    batchNewLabel: vm.batchNewLabel,
    opsHealthLabel: vm.opsHealthLabel,
    watchHealthLabel: vm.watchHealthLabel,
    opsHealthTone: vm.opsHealthTone,
    watchHealthTone: vm.watchHealthTone,
    checksPanelOpen: vm.checksPanelOpen,
    hasChecksFindings: vm.hasChecksFindings,
    hasFindingsErrors: vm.hasFindingsErrors,
    hasFindingsWarnings: vm.hasFindingsWarnings,
    findingsErrorLabel: vm.findingsErrorLabel,
    findingsWarningLabel: vm.findingsWarningLabel,
    findingsStatusTone: vm.findingsStatusTone,
    findingsStatusAria: vm.findingsStatusAria,
    showExit: vm.showExit,
    onExitStudio: () => vm.onExitStudio(),
    onBatchNewTask: () => vm.onBatchNewTask(),
    onBatchSendExisting: () => vm.onBatchSendExisting(),
    onToggleChecksPanel: () => vm.onToggleChecksPanel(),
    onCloseChecksPanel: () => vm.onCloseChecksPanel(),
    onRailClick: (e: Event) => vm.onRailClick(e),
  }
})
</controller>

<row
  sail-studio-shell
  gap="0"
  w="full"
  nowrap
  align="stretch"
  sx={style.minH(screen).h(screen).w(full).minW(0).bg(background).fg(foreground).fontFamily(body).fontSize(sm).overflow(hidden)}
>
  <studio-activity-bar studio-rail on:click={onRailClick}>
    <for {rail()} as="item" key={item.key}>
      <tooltip content={item.label} placement="right">
        <a
          href={item.href}
          activity={item.id}
          active={item.active ? 'true' : 'false'}
          aria-label={item.label}
          aria-current={item.active ? 'page' : undefined}
          title={item.label}
          sx={style.w(10).h(10).minW(10).minH(10).radius(md).fg(muted-foreground).when({ hover: style.bg(hover-bg).fg(foreground) }).when({ current: style.bg(selected-bg).fg(primary) })}
        >
          <center sx={style.w(full).h(full)}>
            <icon name={item.icon} size="sm"></icon>
          </center>
        </a>
      </tooltip>
    </for>
    <if {showExit}>
      <tooltip slot="footer" content="Back to app (Esc)" placement="right">
        <button
          type="button"
          exit-studio
          activity="exit"
          aria-label="Back to app (Esc)"
          title="Back to app (Esc)"
          on:click={onExitStudio}
          sx={style.w(10).h(10).minW(10).minH(10).radius(md).fg(muted-foreground).bg(transparent).borderWidth(0).cursor(pointer).when({ hover: style.bg(hover-bg).fg(foreground) })}
        >
          <center sx={style.w(full).h(full)}>
            <icon name="log-out" size="sm"></icon>
          </center>
        </button>
      </tooltip>
    </if>
  </studio-activity-bar>

  <column grow gap="0" min-w="0" sx={style.minH(0).basis(0).overflow(hidden).h(full).minW(0)}>
    <if {hasBatch()}>
      <row
        batch-tray
        align="center"
        gap="2"
        shrink="0"
        nowrap
        sx={style.padX(3).borderBWidth(1).borderB(border-subtle).bg(surface-1).h(7).minH(7)}
      >
        <badge tone="primary" size="sm" variant="soft" batch-count>
          {batchCountLabel()}
        </badge>
        <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>
          Collected
        </p>
        <for {batchChips()} as="chip" key={chip.key}>
          <badge batch-chip={chip.id} tone="muted" size="sm" variant="outline">
            {chip.label}
          </badge>
        </for>
        <row grow></row>
        <button
          type="button"
          size="sm"
          variant="ghost"
          batch-send-existing
          on:click={onBatchSendExisting}
        >
          Add to open task
        </button>
        <button
          type="button"
          size="sm"
          tone="primary"
          batch-new-task
          on:click={onBatchNewTask}
        >
          {batchNewLabel()}
        </button>
      </row>
    </if>

    <row
      grow
      gap="0"
      nowrap
      align="stretch"
      min-w="0"
      sx={style.minH(0).basis(0).overflow(hidden).h(full).minW(0)}
    >
      <column
        grow
        studio-surface
        min-w="0"
        sx={style.minH(0).basis(0).overflow(hidden).h(full).minW(0)}
      >
        <slot name="route"></slot>
      </column>
    </row>

    <!--
      Findings: L1 dialog side sheet (overlay) — always mounted for showModal.
      Flat header/section/footer anatomy (nested chrome failed closed-dialog layout lint).
    -->
    <dialog
      checks-panel
      checks-drawer
      findings-panel
      open={checksPanelOpen()}
      side="end"
      size="lg"
      aria-label="Findings"
      on:sail-close={onCloseChecksPanel}
    >
      <header findings-panel-head checks-panel-head>
        Findings
      </header>
      <section findings-panel-body>
        <if {checksPanelOpen()}>
          <checks-surface panel />
        </if>
      </section>
      <footer>
        <button
          type="button"
          size="sm"
          variant="ghost"
          tone="muted"
          close-checks-panel
          aria-label="Close findings"
          on:click={onCloseChecksPanel}
        >
          Close
        </button>
      </footer>
    </dialog>

    <row
      studio-status
      align="center"
      gap="3"
      shrink="0"
      nowrap
      min-w="0"
      sx={style.h(6).minH(6).padX(3).borderTWidth(1).borderT(border).bg(surface-0).fontSize(xs).fg(muted-foreground).minW(0).overflow(hidden)}
    >
      <p grow min-w="0" sx={style.fontFamily(mono).minW(0).truncate()}>{statusLine()}</p>
      <p save-status shrink="0" sx={style.fontSize(xs)}>{saveStatusLabel()}</p>
      <p theme-status shrink="0" sx={style.fontSize(xs).fg(muted-foreground)}>{themeStatusLabel()}</p>
      <!-- Findings status: colored error · warn counts (not a mute "Checks" label) -->
      <button
        type="button"
        checks-badge
        findings-status
        open-checks-panel
        size="sm"
        variant={hasChecksFindings() ? 'soft' : 'ghost'}
        tone={findingsStatusTone()}
        title="Open findings (Ctrl/⌘+Shift+K)"
        aria-label={findingsStatusAria()}
        aria-pressed={checksPanelOpen() ? 'true' : 'false'}
        on:click={onToggleChecksPanel}
        shrink="0"
        sx={style.padX(2).h(5).minH(5)}
      >
        <row findings-status-inner align="center" gap="1" nowrap>
          <if {hasFindingsErrors()}>
            <badge
              findings-errors
              checks-badge-count
              tone="destructive"
              size="sm"
              variant="solid"
              pill
            >
              {findingsErrorLabel()}
            </badge>
          </if>
          <if {hasFindingsWarnings()}>
            <badge
              findings-warnings
              tone="warning"
              size="sm"
              variant="solid"
              pill
            >
              {findingsWarningLabel()}
            </badge>
          </if>
          <if {!hasChecksFindings()}>
            <badge findings-clean checks-badge-count tone="muted" size="sm" variant="soft" pill>
              0
            </badge>
          </if>
        </row>
      </button>
      <row
        health="ops"
        align="center"
        gap="1"
        shrink="0"
        sx={style.padX(1).when({ hover: style.bg(hover-bg) })}
      >
        <icon name="activity" size="sm"></icon>
        <badge tone={opsHealthTone()} size="sm" variant="soft" health-tone="ops">
          {opsHealthLabel()}
        </badge>
      </row>
      <row
        health="watch"
        align="center"
        gap="1"
        shrink="0"
        sx={style.padX(1).when({ hover: style.bg(hover-bg) })}
      >
        <icon name="radio" size="sm"></icon>
        <badge tone={watchHealthTone()} size="sm" variant="soft" health-tone="watch">
          {watchHealthLabel()}
        </badge>
      </row>
    </row>
  </column>
</row>
