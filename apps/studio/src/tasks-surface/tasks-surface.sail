<!--
  <tasks-surface> — what agents are doing right now.
  Monitoring table of threads; detail in a side dialog.
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
    taskConv: vm.taskConv,
    taskTitle: vm.taskTitle,
    taskDrift: vm.taskDrift,
    taskStatusLabel: vm.taskStatusLabel,
    taskStatusTone: vm.taskStatusTone,
    hasTaskSelected: vm.hasTaskSelected,
    taskReply: vm.taskReply,
    taskAnchorHas: vm.taskAnchorHas,
    taskAnchorSpan: vm.taskAnchorSpan,
    taskAnchorStatus: vm.taskAnchorStatus,
    taskAnchorPath: vm.taskAnchorPath,
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
    onTaskThreadClick: (e: Event) => vm.onTaskThreadClick(e),
    onTaskOpenFilter: (e: Event) => vm.onTaskOpenFilter(e),
    onTaskActorFilter: (e: Event) => vm.onTaskActorFilter(e),
    onTaskDriftFilter: (e: Event) => vm.onTaskDriftFilter(e),
    onTaskReplyInput: (e: Event) => vm.onTaskReplyInput(e),
    onTaskSend: () => vm.onTaskSend(),
    onTaskCommit: () => vm.onTaskCommit(),
    onCloseDetail: () => vm.onCloseDetail(),
    onNewTaskQuick: () => vm.onNewTaskQuick(),
    onRetryTasks: () => vm.onRetryTasks(),
    onAcceptLiveNode: () => vm.onAcceptLiveNode(),
    onRebindSelection: () => vm.onRebindSelection(),
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
  <row
    data-tasks-header
    align="center"
    gap="2"
    shrink="0"
    nowrap
    justify="between"
    sx={style.h(7).padX(3).borderBWidth(1).borderB(border-subtle).bg(surface-1)}
  >
    <row align="center" gap="2" nowrap grow min-w="0">
      <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
        Tasks
      </p>
      <p data-thread-count sx={style.fontSize(xs).fg(muted-foreground)}>
        {threadCountLabel()}
      </p>
    </row>
    <button type="button" size="sm" tone="primary" data-new-task on:click={onNewTaskQuick}>
      New task
    </button>
  </row>

  <row
    data-tasks-toolbar
    align="center"
    gap="2"
    shrink="0"
    nowrap
    sx={style.h(7).padX(2).borderBWidth(1).borderB(border-subtle).bg(surface-1)}
  >
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
    <p sx={style.fontSize(xs).fg(muted-foreground)}>Filter open work by who replied or linked UI.</p>
  </row>

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
    <column gap="0" data-tasks-table sx={style.padY(1)}>
      <if {!hasTaskThreads()}>
        <if {!showTaskErrorAlert()}>
          <empty-state data-tasks-empty variant="placeholder" size="sm" w="full" sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3)}>
            <strong slot="title">{emptyTitle()}</strong>
            {emptyBody()}
          </empty-state>
        </if>
      </if>
      <if {hasTaskThreads()}>
        <for {taskThreadRows()} as="row" key={row.key}>
          <row
            data-task-id={row.id}
            data-thread-row
            data-drift={row.drift}
            data-selected={row.selected ? 'true' : 'false'}
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
              sx={style.w(2).h(2).minW(2).pad(0)}
            ></badge>
            <p grow min-w="0" sx={style.fontSize(sm).truncate()}>{row.title}</p>
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
        </for>
      </if>
    </column>
  </scroll-area>

  <dialog data-task-detail open={hasTaskSelected()} size="lg" side="end" sx={style.maxH(panel-vh-md).overflow(auto)}>
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
    <section sx={style.minH(0).maxH(panel-max-h-sm).overflow(auto)}>
      <column data-task-conversation gap="3" sx={style.minH(0)}>
        <if {hasTaskCommit()}>
          <p data-commit-path sx={style.fontSize(xs).fg(muted-foreground).fontFamily(mono)}>
            Saved {taskCommitPath()}
          </p>
        </if>
        <scroll-area data-messages sx={style.minH(0).maxH(40)}>
          <column gap="2">
            <if {!taskConv().length}>
              <empty-state variant="inline" size="sm" w="full">
                <strong slot="title">No messages yet</strong>
                Reply below or wait for the agent.
              </empty-state>
            </if>
            <for {taskConv()} as="msg" key={msg.key}>
              <card data-message data-actor={msg.actorKind} variant="outline" padding="sm">
                <column gap="1">
                  <badge data-actor-pill={msg.actorKind} tone="muted" size="sm" pill>
                    {msg.actorLabel} · {msg.time}
                  </badge>
                  <if {msg.hasBody}>
                    <p sx={style.fontSize(sm)}>{msg.body}</p>
                  </if>
                  <if {msg.hasArtifacts}>
                    <row gap="2">
                      <for {msg.artifacts} as="art" key={art.key}>
                        <badge data-artifact={art.kind} tone="muted" size="sm" variant="outline">
                          {art.label}
                        </badge>
                      </for>
                    </row>
                  </if>
                </column>
              </card>
            </for>
          </column>
        </scroll-area>

        <if {taskAnchorHas()}>
          <column data-anchor-card gap="2" sx={style.pad(2).bg(surface-0).radius(sm)}>
            <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>Linked UI</p>
            <p sx={style.fontSize(xs).fg(muted-foreground)}>{taskAnchorPath()}</p>
            <p sx={style.fontSize(xs).fontFamily(mono)}>
              {taskAnchorSpan()} · {taskAnchorStatus()}
            </p>
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
          <column data-attachments gap="1">
            <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>
              Attachments · {attachCountLabel()}
            </p>
            <row gap="1">
              <for {taskAttachChips()} as="a" key={a.key}>
                <badge data-attach-chip tone="muted" size="sm" variant="outline">{a.label}</badge>
              </for>
            </row>
          </column>
        </if>
      </column>
    </section>
    <footer>
      <column gap="2" sx={style.w(full)}>
        <row
          data-reply-composer
          align="center"
          gap="2"
          nowrap
          sx={style.w(full)}
        >
          <input
            type="text"
            data-reply-input
            grow
            placeholder="Reply… use @agent to hand off"
            value={taskReply()}
            on:input={onTaskReplyInput}
          />
          <button type="button" size="sm" tone="primary" data-send-reply on:click={onTaskSend}>
            Send
          </button>
        </row>
        <row gap="2" justify="between" nowrap align="center">
          <p data-attach-hint sx={style.fontSize(xs).fg(muted-foreground)}>
            Attach with ⌘⇧A
          </p>
          <button
            type="button"
            size="sm"
            variant="outline"
            data-commit-intent
            on:click={onTaskCommit}
          >
            Save decision
          </button>
        </row>
      </column>
    </footer>
  </dialog>
</column>
