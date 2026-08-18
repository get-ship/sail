<!--
  <sail-studio-chrome> — shell: activity rail + status + route slot.
  Rail: Edit · Components · Themes · Tasks · Config (Checks = status panel).
-->
<scope>
  // no preset: min-w lock
  style shellRoot = { size({ w: 100%, h: screen, min-w: 0, min-h: screen }).overflow(hidden).bg(canvas).fg(canvas) }
  // no preset: size lock
  style railItem = { size({ w: 10, h: 10, min-w: 10, min-h: 10 }).radius(md).fg(muted), when: { hover: bg(hover-bg).fg(canvas), current: bg(selected-bg).fg(primary) } }
  // no preset: size lock + border width 0
  style exitBtn = { size({ w: 10, h: 10, min-w: 10, min-h: 10 }).radius(md).fg(muted).bg(transparent).border({ width: 0 }).cursor(pointer), when: { hover: bg(hover-bg).fg(canvas) } }
  style centerFill = { size(max) }
  style mainCol = { size(grow).overflow(hidden) }
  // no preset: size lock + edge-only border
  style batchTray = { pad(3).size({ h: 7, min-h: 7 }).border({ edge: b, color: border-subtle, width: 1 }).bg(surface-1) }
  style batchLabel = { fg(muted).type(label) }
  style grow1 = { size(grow) }
  style contentRow = { size(grow).overflow(hidden) }
  style surfaceSlot = { size(grow).overflow(hidden) }
  // no preset: min-w lock + edge-only border
  style statusBar = { size({ h: 6, min-w: 0, min-h: 6 }).pad(3).overflow(hidden).border({ edge: t, color: border, width: 1 }).bg(surface-0).fg(muted) }
  style statusLine = { size(grow).type(mono-sm) }
  style statusMeta = { size(hug).type(body-sm) }
  style statusMuted = { size(hug).fg(muted).type(body-sm) }
  // no preset: size lock
  style findingsBtn = { pad(2).size({ h: 6, min-h: 6 }) }
  style healthChip = { size(hug).pad(1), when: { hover: bg(hover-bg) } }
</scope>

<controller>
import type { UiEvent } from '../compositions/ui-event'
import { createShellPage } from '../compositions/shell/shell-page-vm'
export default controller((ctx) => {
  // Pathname from deck layout injection — chrome is reused across leaf navigations.
  const routePath = () => {
    const r = (ctx as { route?: { pathname?: () => string } }).route
    if (r?.pathname) return r.pathname()
    return '/edit'
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
    onRailClick: (e: UiEvent) => vm.onRailClick(e),
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
