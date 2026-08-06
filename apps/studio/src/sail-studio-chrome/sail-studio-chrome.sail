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

<row sail-studio-shell gap="0" nowrap align="stretch">
  style({  box: w(full).minH(screen).h(screen).minW(0).overflow(hidden), paint: bg(background).fg(foreground)  })
  <span>
    style({ box: display(contents), text: fontFamily(body).fontSize(sm) })
    <studio-activity-bar studio-rail on:click={onRailClick}>
    <for {rail()} as="item" key={item.key}>
      <tooltip content={item.label} placement="right">
        <a
          href={item.href}
          activity={item.id}
          active={item.active ? 'true' : 'false'}
          label={item.label}
          current={item.active ? 'page' : undefined}
          title={item.label}

        >
          style({ box: w(10).h(10).minW(10).minH(10), paint: radius(md).fg(muted-foreground) }).when(hover, { paint: bg(hover-bg).fg(foreground) }).when(current, { paint: bg(selected-bg).fg(primary) })

          <center>
            style({ box: w(full).h(full) })

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
          on:click={onExitStudio}

        >
          style({ box: w(10).h(10).minW(10).minH(10), paint: radius(md).fg(muted-foreground).bg(transparent).borderWidth(0).cursor(pointer) }).when(hover, { paint: bg(hover-bg).fg(foreground) })

          <center>
            style({ box: w(full).h(full) })

            <icon name="log-out" size="sm"></icon>
          </center>
        </button>
      </tooltip>
    </if>
  </studio-activity-bar>

  <column gap="0">
    style({  box: grow(1).minW(0).minH(0).basis(0).overflow(hidden).h(full)  })

    <if {hasBatch()}>
      <row batch-tray align="center" gap="2" nowrap>
        style({  box: shrink(0).padX(3).h(7).minH(7), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1)  })

        <badge tone="primary" size="sm" variant="soft" batch-count>
          {batchCountLabel()}
        </badge>
        <p>
          style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold) })

          Collected
        </p>
        <for {batchChips()} as="chip" key={chip.key}>
          <badge batch-chip={chip.id} tone="muted" size="sm" variant="outline">
            {chip.label}
          </badge>
        </for>
        <row>
          style({ box: grow(1) })
</row>
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

    <row gap="0" nowrap align="stretch">
      style({  box: grow(1).minW(0).minH(0).basis(0).overflow(hidden).h(full)  })

      <column studio-surface>
        style({  box: grow(1).minW(0).minH(0).basis(0).overflow(hidden).h(full)  })

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
      label="Findings"
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
          label="Close findings"
          on:click={onCloseChecksPanel}
        >
          Close
        </button>
      </footer>
    </dialog>

    <row studio-status align="center" gap="3" nowrap>
      style({  box: shrink(0).minW(0).h(6).minH(6).padX(3).overflow(hidden), paint: borderTWidth(1).borderT(border).bg(surface-0).fg(muted-foreground)  })
      <span>
        style({ box: display(contents), text: fontSize(xs) })
        <p>
        style({  box: grow(1).minW(0), text: fontFamily(mono).truncate()  })
{statusLine()}</p>
      <p save-status>
        style({ box: shrink(0), text: fontSize(xs) })
{saveStatusLabel()}</p>
      <p theme-status>
        style({ box: shrink(0), paint: fg(muted-foreground), text: fontSize(xs) })
{themeStatusLabel()}</p>
      <!-- Findings status: colored error · warn counts (not a mute "Checks" label) -->
      <button type="button" checks-badge findings-status open-checks-panel size="sm" variant={hasChecksFindings() ? 'soft' : 'ghost'} tone={findingsStatusTone()} title="Open findings (Ctrl/⌘+Shift+K)" label={findingsStatusAria()} pressed={checksPanelOpen() ? 'true' : 'false'} on:click={onToggleChecksPanel}>
        style({  box: shrink(0).padX(2).h(5).minH(5)  })

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
      <row health="ops" align="center" gap="1">
        style({  box: shrink(0).padX(1)  }).when(hover, { paint: bg(hover-bg) })

        <icon name="activity" size="sm"></icon>
        <badge tone={opsHealthTone()} size="sm" variant="soft" health-tone="ops">
          {opsHealthLabel()}
        </badge>
      </row>
      <row health="watch" align="center" gap="1">
        style({  box: shrink(0).padX(1)  }).when(hover, { paint: bg(hover-bg) })

        <icon name="radio" size="sm"></icon>
        <badge tone={watchHealthTone()} size="sm" variant="soft" health-tone="watch">
          {watchHealthLabel()}
        </badge>
      </row>
      </span>
</row>
  </column>
  </span>
</row>
