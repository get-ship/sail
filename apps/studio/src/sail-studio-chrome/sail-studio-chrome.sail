<!--
  <sail-studio-chrome> — shell: activity bar + status bar + route slot.
  Surfaces own sidebar + main. VSCode grammar: density, left-accent active.
  Ban-clean: no div/span; layout keywords + L1 only.
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
    hasBatch: vm.hasBatch,
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
  }
})
</controller>

<row
  data-sail-studio-shell
  gap="0"
  w="full"
  nowrap
  align="stretch"
  sx={style.minH(screen).h(screen).w(full).bg(background).fg(foreground).fontFamily(body).fontSize(sm).overflow(hidden)}
>
  <column
    data-studio-rail
    shrink="0"
    gap="0"
    align="center"
    sx={style.w(12).bg(surface-1).borderRWidth(1).borderR(border).padY(1)}
  >
    <for {rail()} as="item" key={item.key}>
      <tooltip content={item.label} placement="right">
        <a
          href={item.href}
          data-activity={item.id}
          data-active={item.active ? 'true' : 'false'}
          aria-label={item.label}
          aria-current={item.active ? 'page' : undefined}
          sx={style.w(12).h(12).fg(muted-foreground).borderLWidth(2).borderL(transparent).when({ hover: style.bg(hover-bg).fg(foreground) }).when({ current: style.bg(surface-2).fg(foreground).borderL(primary) })}
        >
          <center sx={style.w(full).h(full)}>
            <icon name={item.icon} size="sm" label={item.label}></icon>
          </center>
        </a>
      </tooltip>
    </for>
    <column grow></column>
    <if {showExit}>
      <tooltip content="Back to app (Esc)" placement="right">
        <button
          type="button"
          data-exit-studio
          aria-label="Back to app"
          title="Back to app (Esc)"
          on:click={onExitStudio}
          sx={style.w(12).h(12).fg(muted-foreground).borderLWidth(2).borderL(transparent).when({ hover: style.bg(hover-bg).fg(foreground) })}
        >
          <center sx={style.w(full).h(full)}>
            <icon name="log-out" size="sm" label="Back to app"></icon>
          </center>
        </button>
      </tooltip>
    </if>
  </column>

  <column grow gap="0" min-w="0" sx={style.minH(0).basis(0).overflow(hidden).h(full)}>
    <if {hasBatch()}>
      <row
        data-batch-tray
        align="center"
        gap="2"
        shrink="0"
        sx={style.padX(2).padY(1).borderBWidth(1).borderB(border-subtle).bg(surface-1).h(7)}
      >
        <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
          Collected
        </p>
        <for {batchChips()} as="chip" key={chip.key}>
          <p data-batch-chip={chip.id} sx={style.fontSize(xs).fg(muted-foreground)}>
            {chip.label}
          </p>
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

    <column grow data-studio-surface sx={style.minH(0).basis(0).overflow(hidden).h(full).minW(0)}>
      <slot name="route"></slot>
    </column>

    <row
      data-studio-status
      align="center"
      gap="3"
      shrink="0"
      nowrap
      min-w="0"
      sx={style.h(6).padX(3).borderTWidth(1).borderT(border).bg(surface-1).fontSize(xs).fg(muted-foreground).minW(0).overflow(hidden)}
    >
      <p grow min-w="0" sx={style.fontFamily(mono).minW(0).truncate()}>{statusLine()}</p>
      <row
        data-health="ops"
        align="center"
        gap="1"
        shrink="0"
        sx={style.padX(2).when({ hover: style.bg(hover-bg) })}
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
        sx={style.padX(2).when({ hover: style.bg(hover-bg) })}
      >
        <icon name="radio" size="sm"></icon>
        <badge tone={watchHealthTone()} size="sm" variant="soft" data-health-tone="watch">
          {watchHealthLabel()}
        </badge>
      </row>
    </row>
  </column>
</row>
