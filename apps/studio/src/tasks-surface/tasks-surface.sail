<!--
  <tasks-surface> — anchored threads, drift, crops, @agent handoff.
  Chat-elements dogfood: sail-message-list · sail-composer · sail-markdown · sail-diff.
  Ban-clean: no div/span; layout keywords + L1 only.
-->
<controller>
import { createTasksSurface } from './tasks-surface-vm'
export default controller(() => {
  const vm = createTasksSurface()
  return {
    taskThreadRows: vm.taskThreadRows,
    hasTaskThreads: vm.hasTaskThreads,
    threadCountLabel: vm.threadCountLabel,
    taskTitle: vm.taskTitle,
    taskDrift: vm.taskDrift,
    taskStatusLabel: vm.taskStatusLabel,
    taskStatusTone: vm.taskStatusTone,
    hasTaskSelected: vm.hasTaskSelected,
    taskAnchorHas: vm.taskAnchorHas,
    taskAnchorStatus: vm.taskAnchorStatus,
    taskAnchorStatusRaw: vm.taskAnchorStatusRaw,
    taskAnchorTone: vm.taskAnchorTone,
    taskAnchorPath: vm.taskAnchorPath,
    taskAnchorFileSpan: vm.taskAnchorFileSpan,
    taskAnchorMovedHint: vm.taskAnchorMovedHint,
    taskAttachChips: vm.taskAttachChips,
    hasTaskAttach: vm.hasTaskAttach,
    taskError: vm.taskError,
    showTaskErrorAlert: vm.showTaskErrorAlert,
    taskCommitPath: vm.taskCommitPath,
    hasTaskCommit: vm.hasTaskCommit,
    openFilterVal: vm.openFilterVal,
    actorFilterVal: vm.actorFilterVal,
    driftFilterVal: vm.driftFilterVal,
    attachCountLabel: vm.attachCountLabel,
    showAcceptLive: vm.showAcceptLive,
    showRebind: vm.showRebind,
    emptyTitle: vm.emptyTitle,
    emptyBody: vm.emptyBody,
    threadMarkdown: vm.threadMarkdown,
    hasThreadMarkdown: vm.hasThreadMarkdown,
    hasMessageItems: vm.hasMessageItems,
    hasTaskDiff: vm.hasTaskDiff,
    diffFrom: vm.diffFrom,
    diffTo: vm.diffTo,
    canAttachCrop: vm.canAttachCrop,
    composerDisabled: vm.composerDisabled,
    composerStatusLabel: vm.composerStatusLabel,
    showComposerStatus: vm.showComposerStatus,
    onTaskThreadClick: (e: Event) => vm.onTaskThreadClick(e),
    onTaskOpenFilter: (e: Event) => vm.onTaskOpenFilter(e),
    onTaskActorFilter: (e: Event) => vm.onTaskActorFilter(e),
    onTaskDriftFilter: (e: Event) => vm.onTaskDriftFilter(e),
    onComposerSubmit: (e: Event) => vm.onComposerSubmit(e),
    onTaskCommit: () => vm.onTaskCommit(),
    onCloseDetail: () => vm.onCloseDetail(),
    onNewTaskQuick: () => vm.onNewTaskQuick(),
    onRetryTasks: () => vm.onRetryTasks(),
    onAcceptLiveNode: () => vm.onAcceptLiveNode(),
    onRebindSelection: () => vm.onRebindSelection(),
    onAttachCrop: () => vm.onAttachCrop(),
  }
})
</controller>

<column
  data-surface="tasks"
  data-tasks-surface
  gap="0"
  grow
  sx={style.minH(0).h(full).minW(0).overflow(hidden)}
>
  <studio-page-header data-tasks-header title="Tasks">
    <p data-thread-count sx={style.fontSize(xs).fg(muted-foreground)}>
      {threadCountLabel()}
    </p>
    <button slot="actions" type="button" size="sm" tone="primary" data-new-task on:click={onNewTaskQuick}>
      New task
    </button>
  </studio-page-header>

  <studio-toolbar data-tasks-toolbar>
    <select
      data-filter-open
      value={openFilterVal()}
      on:change={onTaskOpenFilter}
      aria-label="Show open or all tasks"
      w="28"
    >
      <option value="open">Open</option>
      <option value="all">All</option>
    </select>
    <separator orientation="vertical" sx={style.h(4)} />
    <select
      data-filter-actor
      value={actorFilterVal()}
      on:change={onTaskActorFilter}
      aria-label="Filter by who last replied"
      w="28"
    >
      <option value="all">Anyone</option>
      <option value="human">You</option>
      <option value="agent">Agent</option>
    </select>
    <select
      data-filter-drift
      value={driftFilterVal()}
      on:change={onTaskDriftFilter}
      aria-label="Filter by linked location"
      w="28"
    >
      <option value="all">Any location</option>
      <option value="ok">In place</option>
      <option value="moved">Moved</option>
      <option value="lost">Lost</option>
    </select>
    <p sx={style.fontSize(xs).fg(muted-foreground)}>Filter by reply or linked UI.</p>
  </studio-toolbar>

  <if {showTaskErrorAlert()}>
    <row
      data-tasks-error
      align="center"
      gap="2"
      sx={style.padX(3).h(7).borderBWidth(1).borderB(border)}
    >
      <p grow min-w="0" sx={style.fontSize(xs).fg(destructive).truncate()}>{taskError()}</p>
      <button type="button" size="sm" variant="ghost" data-retry-tasks on:click={onRetryTasks}>
        Retry
      </button>
    </row>
  </if>

  <scroll-area
    data-threads-list
    grow
    min-w="0"
    on:click={onTaskThreadClick}
    sx={style.minH(0).h(full).basis(0)}
  >
    <row justify="center" sx={style.w(full)}>
      <column gap="0" data-tasks-table sx={style.padY(1).w(full).maxW(panel-max-w-lg).padX(3)}>
      <if {!hasTaskThreads()}>
        <if {!showTaskErrorAlert()}>
          <studio-empty data-tasks-empty title={emptyTitle()}>
            {emptyBody()}
          </studio-empty>
        </if>
      </if>
      <if {hasTaskThreads()}>
        <for {taskThreadRows()} as="row" key={row.key}>
          <row
            data-task-id={row.id}
            data-thread-row
            data-drift={row.drift}
            data-selected={row.selected ? 'true' : 'false'}
            data-collapsed={row.collapsed ? 'true' : 'false'}
            gap="2"
            align="center"
            nowrap
            sx={style.h(6).padX(3).when({ hover: style.bg(hover-bg) })}
          >
            <badge
              data-status-badge={row.status}
              data-status-dot={row.status}
              tone={row.statusTone}
              size="sm"
              variant="solid"
              pill
              aria-label={row.statusLabel}
              shrink="0"
              sx={style.w(2).h(2).minW(2).pad(0)}
            ></badge>
            <p grow min-w="0" sx={style.fontSize(sm).truncate()}>{row.title}</p>
            <if {!row.collapsed}>
              <!-- shrink=0: keep meta as a tight trailing cluster (not crushed/scattered) -->
              <row align="center" gap="2" nowrap shrink="0" data-thread-meta>
                <p data-actor-chip={row.actorChip} sx={style.fontSize(xs).fg(muted-foreground)}>
                  {row.actorChip}
                </p>
                <if {row.showDrift}>
                  <p data-drift-chip={row.driftChip} sx={style.fontSize(xs).fg(warning)}>
                    {row.driftChip}
                  </p>
                </if>
                <p sx={style.fontSize(xs).fg(muted-foreground)}>{row.updatedLabel}</p>
              </row>
            </if>
          </row>
        </for>
      </if>
    </column>
    </row>
  </scroll-area>

  <!-- Side sheet: definite px height so section maxH fits under taffy (vh alone under-measured). -->
  <dialog data-task-detail open={hasTaskSelected()} size="lg" side="end" sx={style.h(panel-max-h-lg).maxH(panel-vh-md).minH(0).overflow(hidden)}>
    <header>
      <row align="center" gap="2" nowrap justify="between" sx={style.w(full)}>
        <row align="center" gap="2" nowrap grow min-w="0">
          <strong sx={style.fontSize(md)}>{taskTitle()}</strong>
          <badge data-status-detail tone={taskStatusTone()} size="sm" variant="soft" pill>
            {taskStatusLabel()}
          </badge>
          <badge data-anchor-status={taskDrift()} tone="muted" size="sm" pill>
            {taskDrift()}
          </badge>
        </row>
        <button type="button" size="sm" variant="ghost" tone="muted" on:click={onCloseDetail}>
          Close
        </button>
      </row>
    </header>
    <!--
      Section: maxH only (not dual forced-h). Order: anchor first (differentiator),
      then attachments, then messages (empty shrinks; filled gets maxH for virtualizer).
    -->
    <section sx={style.minH(0).maxH(panel-max-h-sm).overflow(auto)}>
      <column data-task-conversation gap="3" sx={style.minH(0)}>
        <if {hasTaskCommit()}>
          <p data-commit-path sx={style.fontSize(xs).fg(muted-foreground).fontFamily(mono)}>
            Committed {taskCommitPath()}
          </p>
        </if>

        <if {taskAnchorHas()}>
          <column
            data-anchor-card
            data-anchor-chip
            data-drift={taskAnchorStatusRaw()}
            gap="2"
            sx={style.pad(2).bg(surface-0).radius(sm).borderWidth(1).border(border-subtle)}
          >
            <row align="center" gap="2" nowrap>
              <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>
                Linked UI
              </p>
              <badge
                data-anchor-drift={taskAnchorStatusRaw()}
                tone={taskAnchorTone()}
                size="sm"
                variant="soft"
                pill
              >
                {taskAnchorStatus()}
              </badge>
            </row>
            <p sx={style.fontSize(xs).fg(muted-foreground)}>{taskAnchorPath()}</p>
            <p data-anchor-span sx={style.fontSize(xs).fontFamily(mono)}>
              {taskAnchorFileSpan()}
            </p>
            <if {taskAnchorMovedHint()}>
              <p data-anchor-hint sx={style.fontSize(xs).fg(muted-foreground)}>
                {taskAnchorMovedHint()}
              </p>
            </if>
            <if {showAcceptLive()}>
              <button
                type="button"
                size="sm"
                variant="soft"
                data-accept-live
                on:click={onAcceptLiveNode}
              >
                Accept new location
              </button>
            </if>
            <if {showRebind()}>
              <button
                type="button"
                size="sm"
                variant="outline"
                data-rebind-selection
                on:click={onRebindSelection}
              >
                Re-link to selection
              </button>
            </if>
          </column>
        </if>

        <if {hasTaskAttach()}>
          <column data-attachments gap="2">
            <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>
              Attachments · {attachCountLabel()}
            </p>
            <row gap="2" align="start">
              <for {taskAttachChips()} as="a" key={a.key}>
                <column gap="1" data-attach-chip data-attach-kind={a.kind} sx={style.maxW(28)}>
                  <if {a.hasThumb}>
                    <img
                      data-attach-thumb
                      src={a.thumbSrc}
                      alt={a.label}
                      sx={style.w(24).h(16).radius(sm).borderWidth(1).border(border-subtle)}
                    />
                  </if>
                  <if {!a.hasThumb}>
                    <badge
                      data-attach-thumb="false"
                      tone="muted"
                      size="sm"
                      variant="outline"
                    >
                      {a.label}
                    </badge>
                  </if>
                  <if {a.hasThumb}>
                    <p sx={style.fontSize(xs).fg(muted-foreground).truncate()}>{a.label}</p>
                  </if>
                </column>
              </for>
            </row>
            <if {hasTaskDiff()}>
              <sail-diff
                data-attach-diff
                from={diffFrom()}
                to={diffTo()}
              ></sail-diff>
            </if>
          </column>
        </if>

        <if {hasThreadMarkdown()}>
          <sail-markdown
            data-thread-markdown
            value={threadMarkdown()}
            sx={style.fontSize(sm)}
          ></sail-markdown>
        </if>

        <!--
          Message list height: empty shrinks to empty-state; with turns use maxH
          only (virtualizer needs a bound). Never dual-force h=maxH=sm under a
          sm section — that buried the anchor chip.
        -->
        <if {hasMessageItems()}>
          <column data-messages gap="0" sx={style.minH(0).maxH(panel-max-h-sm).h(panel-max-h-sm)}>
            <sail-message-list
              data-messages-host
              sticky-bottom
              sx={style.h(full).minH(0).w(full)}
            >
              <empty-state slot="empty" variant="inline" size="sm" w="full">
                <strong slot="title">No messages yet</strong>
                Ask the agent below or wait for a reply.
              </empty-state>
            </sail-message-list>
          </column>
        </if>
        <if {!hasMessageItems()}>
          <column data-messages gap="0" sx={style.minH(0)}>
            <sail-message-list
              data-messages-host
              sticky-bottom
              sx={style.minH(0).w(full)}
            >
              <empty-state slot="empty" variant="inline" size="sm" w="full">
                <strong slot="title">No messages yet</strong>
                Ask the agent below or wait for a reply.
              </empty-state>
            </sail-message-list>
          </column>
        </if>
      </column>
    </section>
    <footer>
      <column gap="2" sx={style.w(full)} data-reply-composer>
        <if {showComposerStatus()}>
          <p data-composer-status sx={style.fontSize(xs).fg(muted-foreground)}>
            {composerStatusLabel()}
          </p>
        </if>
        <sail-composer
          data-task-composer
          data-reply-input
          multiline
          disabled={composerDisabled()}
          placeholder="Ask the agent… use @agent to hand off"
          on:sail-composer-submit={onComposerSubmit}
          sx={style.w(full)}
        >
          <row slot="trailing" gap="1" align="center" nowrap>
            <button
              type="button"
              size="sm"
              variant="ghost"
              data-attach-crop
              disabled={!canAttachCrop()}
              on:click={onAttachCrop}
            >
              Attach screenshot
            </button>
            <button
              type="button"
              size="sm"
              variant="outline"
              data-commit-intent
              on:click={onTaskCommit}
            >
              Commit
            </button>
          </row>
        </sail-composer>
        <p data-attach-hint sx={style.fontSize(xs).fg(muted-foreground)}>
          Crops from the tray attach here — no file dialogs.
        </p>
      </column>
    </footer>
  </dialog>
</column>
