<!--
  <sail-studio-chrome> — shell: activity rail + status + route slot.
  Surfaces own sidebar + main. VSCode grammar: density, left-accent active.
  Stable zones: rail/status fixed; surface slot minH(0)/grow so leaf remounts
  do not shift chrome. Brand/header rhythm: h(8). Ban-clean: layout + L1 only.
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
    showExit: vm.showExit,
    onExitStudio: () => vm.onExitStudio(),
    onBatchNewTask: () => vm.onBatchNewTask(),
    onBatchSendExisting: () => vm.onBatchSendExisting(),
    onRailClick: (e: Event) => vm.onRailClick(e),
  }
})
</controller>

<row
  data-sail-studio-shell
  gap="0"
  w="full"
  nowrap
  align="stretch"
  sx={style.minH(screen).h(screen).w(full).minW(0).bg(background).fg(foreground).fontFamily(body).fontSize(sm).overflow(hidden)}
>
  <!-- Activity rail: single width token w(12) owned by studio-activity-bar -->
  <studio-activity-bar data-studio-rail on:click={onRailClick}>
    <for {rail()} as="item" key={item.key}>
      <tooltip content={item.label} placement="right">
        <a
          href={item.href}
          data-activity={item.id}
          data-active={item.active ? 'true' : 'false'}
          aria-label={item.label}
          aria-current={item.active ? 'page' : undefined}
          title={item.label}
          sx={style.w(10).h(10).radius(md).fg(muted-foreground).borderLWidth(2).borderL(transparent).when({ hover: style.bg(hover-bg).fg(foreground) }).when({ current: style.bg(surface-2).fg(foreground).borderL(primary) })}
        >
          <center sx={style.w(full).h(full)}>
            <!-- icon unlabeled: link aria-label is the single accessible name -->
            <icon name={item.icon} size="sm"></icon>
          </center>
        </a>
      </tooltip>
    </for>
    <column slot="footer" shrink="0" align="center" gap="0" sx={style.padB(1)}>
      <if {showExit}>
        <tooltip content="Back to app (Esc)" placement="right">
          <button
            type="button"
            data-exit-studio
            aria-label="Back to app (Esc)"
            title="Back to app (Esc)"
            on:click={onExitStudio}
            sx={style.w(10).h(10).radius(md).fg(muted-foreground).borderLWidth(2).borderL(transparent).when({ hover: style.bg(hover-bg).fg(foreground) })}
          >
            <center sx={style.w(full).h(full)}>
              <icon name="log-out" size="sm"></icon>
            </center>
          </button>
        </tooltip>
      </if>
    </column>
  </studio-activity-bar>

  <!-- Main column: batch (optional) + surface (grow) + status (fixed). -->
  <column grow gap="0" min-w="0" sx={style.minH(0).basis(0).overflow(hidden).h(full).minW(0)}>
    <if {hasBatch()}>
      <row
        data-batch-tray
        align="center"
        gap="2"
        shrink="0"
        nowrap
        sx={style.padX(3).borderBWidth(1).borderB(border-subtle).bg(surface-1).h(7).minH(7)}
      >
        <badge tone="primary" size="sm" variant="soft" data-batch-count>
          {batchCountLabel()}
        </badge>
        <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>
          Collected
        </p>
        <for {batchChips()} as="chip" key={chip.key}>
          <badge data-batch-chip={chip.id} tone="muted" size="sm" variant="outline">
            {chip.label}
          </badge>
        </for>
        <row grow></row>
        <button
          type="button"
          size="sm"
          variant="ghost"
          data-batch-send-existing
          on:click={onBatchSendExisting}
        >
          Add to open task
        </button>
        <button
          type="button"
          size="sm"
          tone="primary"
          data-batch-new-task
          on:click={onBatchNewTask}
        >
          {batchNewLabel()}
        </button>
      </row>
    </if>

    <column
      grow
      data-studio-surface
      min-w="0"
      sx={style.minH(0).basis(0).overflow(hidden).h(full).minW(0)}
    >
      <slot name="route"></slot>
    </column>

    <row
      data-studio-status
      align="center"
      gap="3"
      shrink="0"
      nowrap
      min-w="0"
      sx={style.h(6).minH(6).padX(3).borderTWidth(1).borderT(border).bg(surface-0).fontSize(xs).fg(muted-foreground).minW(0).overflow(hidden)}
    >
      <p grow min-w="0" sx={style.fontFamily(mono).minW(0).truncate()}>{statusLine()}</p>
      <p data-save-status shrink="0" sx={style.fontSize(xs)}>{saveStatusLabel()}</p>
      <p data-theme-status shrink="0" sx={style.fontSize(xs).fg(muted-foreground)}>{themeStatusLabel()}</p>
      <row
        data-health="ops"
        align="center"
        gap="1"
        shrink="0"
        sx={style.padX(1).when({ hover: style.bg(hover-bg) })}
      >
        <icon name="activity" size="sm"></icon>
        <badge tone={opsHealthTone()} size="sm" variant="soft" data-health-tone="ops">
          {opsHealthLabel()}
        </badge>
      </row>
      <row
        data-health="watch"
        align="center"
        gap="1"
        shrink="0"
        sx={style.padX(1).when({ hover: style.bg(hover-bg) })}
      >
        <icon name="radio" size="sm"></icon>
        <badge tone={watchHealthTone()} size="sm" variant="soft" data-health-tone="watch">
          {watchHealthLabel()}
        </badge>
      </row>
    </row>
  </column>
</row>
