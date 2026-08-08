<!--
  <sail-studio-chrome> — shell: activity rail + status + route slot.
  Rail: Edit · Components · Themes · Tasks · Config (Checks = status panel).
-->
<scope>
  style shellRoot = { box: w(full).minH(screen).h(screen).minW(0).overflow(hidden), paint: bg(background).fg(foreground) }
  style railItem = { box: w(10).h(10).minW(10).minH(10), paint: radius(md).fg(muted-foreground), when: { hover: { paint: bg(hover-bg).fg(foreground) }, current: { paint: bg(selected-bg).fg(primary) } } }
  style exitBtn = { box: w(10).h(10).minW(10).minH(10), paint: radius(md).fg(muted-foreground).bg(transparent).borderWidth(0).cursor(pointer), when: { hover: { paint: bg(hover-bg).fg(foreground) } } }
  style centerFill = { box: w(full).h(full) }
  style mainCol = { box: grow(1).minW(0).minH(0).basis(0).overflow(hidden).h(full) }
  style batchTray = { box: shrink(0).padX(3).h(7).minH(7), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1) }
  style batchLabel = { paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold) }
  style grow1 = { box: grow(1) }
  style contentRow = { box: grow(1).minW(0).minH(0).basis(0).overflow(hidden).h(full) }
  style surfaceSlot = { box: grow(1).minW(0).minH(0).basis(0).overflow(hidden).h(full) }
  style statusBar = { box: shrink(0).minW(0).h(6).minH(6).padX(3).overflow(hidden), paint: borderTWidth(1).borderT(border).bg(surface-0).fg(muted-foreground) }
  style statusLine = { box: grow(1).minW(0), text: fontFamily(mono).fontSize(xs).truncate() }
  style statusMeta = { box: shrink(0), text: fontSize(xs) }
  style statusMuted = { box: shrink(0), paint: fg(muted-foreground), text: fontSize(xs) }
  style findingsBtn = { box: shrink(0).padX(2).h(5).minH(5) }
  style healthChip = { box: shrink(0).padX(1), when: { hover: { paint: bg(hover-bg) } } }
</scope>

<controller>
import { createShellPage } from '../compositions/shell/shell-page-vm'
export default controller((ctx) => {
  // Pathname from deck layout injection — chrome is reused across leaf navigations.
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

<row sail-studio-shell gap="0" nowrap align="stretch" style(shellRoot)>
  <studio-activity-bar studio-rail on:click={onRailClick}>
    <for {rail() as item} key={item.key}>
      <tooltip content={item.label} placement="right">
        <a
          href={item.href}
          activity={item.id}
          active={item.active ? 'true' : 'false'}
          label={item.label}
          current={item.active ? 'page' : undefined}
          title={item.label} style(railItem)>
          <center style(centerFill)>
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
          label="Back to app (Esc)"
          title="Back to app (Esc)"
          on:click={onExitStudio} style(exitBtn)>
          <center style(centerFill)>
            <icon name="log-out" size="sm"></icon>
          </center>
        </button>
      </tooltip>
    </if>
  </studio-activity-bar>

  <column gap="0" style(mainCol)>
    <if {hasBatch()}>
      <row batch-tray align="center" gap="2" nowrap style(batchTray)>
        <badge tone="primary" size="sm" variant="soft" batch-count>
          {batchCountLabel()}
        </badge>
        <p style(batchLabel)>
          Collected
        </p>
        <for {batchChips() as chip} key={chip.key}>
          <badge batch-chip={chip.id} tone="muted" size="sm" variant="outline">
            {chip.label}
          </badge>
        </for>
        <row style(grow1)></row>
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

    <row gap="0" nowrap align="stretch" style(contentRow)>
      <column studio-surface style(surfaceSlot)>
        <slot name="route"></slot>
      </column>
    </row>

    <!-- Findings side sheet — always mounted so bindOpen can showModal. -->
    <dialog
      checks-panel
      checks-drawer
      findings-panel
      open={checksPanelOpen()}
      side="end"
      size="lg"
      label="Findings"
      on-close={onCloseChecksPanel}
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
          label="Close findings"
          on:click={onCloseChecksPanel}
        >
          Close
        </button>
      </footer>
    </dialog>

    <row studio-status align="center" gap="3" nowrap style(statusBar)>
      <p style(statusLine)>{statusLine()}</p>
      <p save-status style(statusMeta)>{saveStatusLabel()}</p>
      <p theme-status style(statusMuted)>{themeStatusLabel()}</p>
      <button type="button" checks-badge findings-status open-checks-panel size="sm" variant={hasChecksFindings() ? 'soft' : 'ghost'} tone={findingsStatusTone()} title="Open findings (Ctrl/⌘+Shift+K)" label={findingsStatusAria()} pressed={checksPanelOpen() ? 'true' : 'false'} on:click={onToggleChecksPanel} style(findingsBtn)>
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
      <row health="ops" align="center" gap="1" style(healthChip)>
        <icon name="activity" size="sm"></icon>
        <badge tone={opsHealthTone()} size="sm" variant="soft" health-tone="ops">
          {opsHealthLabel()}
        </badge>
      </row>
      <row health="watch" align="center" gap="1" style(healthChip)>
        <icon name="radio" size="sm"></icon>
        <badge tone={watchHealthTone()} size="sm" variant="soft" health-tone="watch">
          {watchHealthLabel()}
        </badge>
      </row>
</row>
  </column>
</row>
