<!--
  <tasks-surface> — thread list + conversation + composer.
-->
<controller>
import { createTasksSurface } from './tasks-surface-vm'
export default controller(() => {
  const c = createTasksSurface()
  return {
    taskThreadRows: c.taskThreadRows,
    hasTaskThreads: c.hasTaskThreads,
    threadCountLabel: c.threadCountLabel,
    taskTitle: c.taskTitle,
    taskStatusLabel: c.taskStatusLabel,
    taskStatusTone: c.taskStatusTone,
    hasTaskSelected: c.hasTaskSelected,
    taskAnchorHas: c.taskAnchorHas,
    taskAnchorStatus: c.taskAnchorStatus,
    taskAnchorStatusRaw: c.taskAnchorStatusRaw,
    taskAnchorTone: c.taskAnchorTone,
    taskAnchorPath: c.taskAnchorPath,
    taskAttachChips: c.taskAttachChips,
    hasTaskAttach: c.hasTaskAttach,
    taskError: c.taskError,
    showTaskErrorAlert: c.showTaskErrorAlert,
    openFilterVal: c.openFilterVal,
    actorFilterVal: c.actorFilterVal,
    emptyTitle: c.emptyTitle,
    emptyBody: c.emptyBody,
    threadMarkdown: c.threadMarkdown,
    hasThreadMarkdown: c.hasThreadMarkdown,
    hasMessageItems: c.hasMessageItems,
    composerDisabled: c.composerDisabled,
    canAttachCrop: c.canAttachCrop,
    onTaskThreadClick: (e: Event) => c.onTaskThreadClick(e),
    onTaskOpenFilter: (e: Event) => c.onTaskOpenFilter(e),
    onTaskActorFilter: (e: Event) => c.onTaskActorFilter(e),
    onComposerSubmit: (e: Event) => c.onComposerSubmit(e),
    onCloseDetail: () => c.onCloseDetail(),
    onNewTaskQuick: () => c.onNewTaskQuick(),
    onRetryTasks: () => c.onRetryTasks(),
    onAttachCrop: () => c.onAttachCrop(),
  }
})
</controller>

<column
  surface="tasks"
  tasks-surface
  gap="0"
  grow
  sx={style.minH(0).h(full).minW(0).overflow(hidden)}
>
  <row
    tasks-header
    align="center"
    justify="between"
    gap="3"
    shrink="0"
    nowrap
    sx={style.h(10).padX(4).borderBWidth(1).borderB(border-subtle).bg(surface-1)}
  >
    <row align="center" gap="3" nowrap grow min-w="0">
      <p sx={style.fontSize(md).fontWeight(semibold)}>Tasks</p>
      <p thread-count sx={style.fontSize(xs).fg(muted-foreground).fontFamily(mono)}>
        {threadCountLabel()}
      </p>
      <select
        filter-open
        size="sm"
        value={openFilterVal()}
        on:change={onTaskOpenFilter}
        label="Open or all"
        sx={style.w(24)}
      >
        <option value="open">Open</option>
        <option value="all">All</option>
      </select>
      <select
        filter-actor
        size="sm"
        value={actorFilterVal()}
        on:change={onTaskActorFilter}
        label="Last reply"
        sx={style.w(28)}
      >
        <option value="all">Anyone</option>
        <option value="human">You</option>
        <option value="agent">Agent</option>
      </select>
    </row>
    <button type="button" size="sm" tone="primary" new-task on:click={onNewTaskQuick} shrink="0">
      New task
    </button>
  </row>

  <if {showTaskErrorAlert()}>
    <row
      tasks-error
      align="center"
      gap="2"
      shrink="0"
      sx={style.padX(3).h(7).borderBWidth(1).borderB(border)}
    >
      <p grow min-w="0" sx={style.fontSize(xs).fg(destructive).truncate()}>{taskError()}</p>
      <button type="button" size="sm" variant="ghost" retry-tasks on:click={onRetryTasks}>
        Retry
      </button>
    </row>
  </if>

  <row
    tasks-workbench
    grow
    gap="0"
    nowrap
    align="stretch"
    sx={style.minH(0).basis(0).h(full).minW(0).overflow(hidden)}
  >
    <column
      threads-list
      tasks-nav
      grow
      gap="0"
      sx={style.minH(0).h(full).minW(panel-min-w-md).maxW(panel-max-w-md).basis(0).borderRWidth(1).borderR(border-subtle).bg(surface-1).overflow(hidden)}
    >
      <scroll-area grow on:click={onTaskThreadClick} sx={style.minH(0).basis(0).h(full)}>
        <column gap="0" tasks-table sx={style.padY(1).padX(1)}>
          <if {!hasTaskThreads()}>
            <if {!showTaskErrorAlert()}>
              <studio-empty tasks-empty title={emptyTitle()} size="sm">
                {emptyBody()}
              </studio-empty>
            </if>
          </if>
          <if {hasTaskThreads()}>
            <for {taskThreadRows()} as="row" key={row.key}>
              <studio-list-row
                task-id={row.id}
                thread-row
                drift={row.drift}
                title={row.title}
                meta={row.updatedLabel}
                selected={row.selected}
              >
                <badge
                  slot="leading"
                  status-badge={row.status}
                  tone={row.statusTone}
                  size="sm"
                  variant="soft"
                  pill
                  label={row.statusLabel}
                  shrink="0"
                >
                  {row.statusLabel}
                </badge>
              </studio-list-row>
            </for>
          </if>
        </column>
      </scroll-area>
    </column>

    <column
      task-detail
      grow
      gap="0"
      min-w="0"
      sx={style.minH(0).h(full).basis(0).minW(0).bg(surface-0).overflow(hidden)}
    >
      <if {!hasTaskSelected()}>
        <column grow align="center" justify="center" sx={style.minH(0).h(full).pad(4)}>
          <empty-state variant="placeholder" size="sm" w="full" task-detail-empty>
            <strong slot="title">Select a task</strong>
            Or create a new one.
          </empty-state>
        </column>
      </if>

      <if {hasTaskSelected()}>
        <row
          task-detail-header
          align="center"
          gap="2"
          nowrap
          shrink="0"
          justify="between"
          sx={style.h(10).padX(3).borderBWidth(1).borderB(border-subtle).bg(surface-1).minW(0)}
        >
          <row align="center" gap="2" nowrap grow min-w="0">
            <strong sx={style.fontSize(md).truncate()}>{taskTitle()}</strong>
            <badge status-detail tone={taskStatusTone()} size="sm" variant="soft" pill>
              {taskStatusLabel()}
            </badge>
          </row>
          <button type="button" size="sm" variant="ghost" tone="muted" on:click={onCloseDetail}>
            Close
          </button>
        </row>

        <!-- Anchor: status only when present — no re-link/accept choreography in scaffold -->
        <if {taskAnchorHas()}>
          <row
            anchor-card
            anchor-chip
            drift={taskAnchorStatusRaw()}
            align="center"
            gap="2"
            nowrap
            shrink="0"
            sx={style.padX(3).padY(2).borderBWidth(1).borderB(border-subtle).bg(surface-0).minW(0)}
          >
            <badge
              anchor-drift={taskAnchorStatusRaw()}
              tone={taskAnchorTone()}
              size="sm"
              variant="soft"
              pill
              shrink="0"
            >
              {taskAnchorStatus()}
            </badge>
            <p grow min-w="0" sx={style.fontSize(xs).fontFamily(mono).truncate()}>
              {taskAnchorPath()}
            </p>
          </row>
        </if>

        <if {hasTaskAttach()}>
          <row attachments gap="2" align="center" shrink="0" sx={style.padX(3).padY(2).borderBWidth(1).borderB(border-subtle)}>
            <for {taskAttachChips()} as="a" key={a.key}>
              <badge attach-chip attach-kind={a.kind} tone="muted" size="sm" variant="outline" pill>
                {a.label}
              </badge>
            </for>
          </row>
        </if>

        <column
          task-conversation
          grow
          gap="0"
          sx={style.minH(0).basis(0).h(full).overflow(hidden)}
        >
          <if {hasThreadMarkdown()}>
            <scroll-area shrink="0" sx={style.maxH(32).minH(0)}>
              <column sx={style.pad(3)}>
                <sail-markdown thread-markdown value={threadMarkdown()} sx={style.fontSize(sm)}></sail-markdown>
              </column>
            </scroll-area>
          </if>
          <sail-message-list
            messages-host
            messages
            sticky-bottom
            grow
            sx={style.minH(0).h(full).w(full).basis(0)}
          >
            <empty-state slot="empty" variant="inline" size="sm" w="full">
              <strong slot="title">No messages yet</strong>
              Write below to start.
            </empty-state>
          </sail-message-list>
        </column>

        <column
          reply-composer
          gap="2"
          shrink="0"
          sx={style.pad(3).borderTWidth(1).borderT(border-subtle).bg(surface-1)}
        >
          <sail-composer
            task-composer
            reply-input
            multiline
            disabled={composerDisabled()}
            placeholder="Message…"
            on:sail-composer-submit={onComposerSubmit}
            sx={style.w(full)}
          >
            <row slot="trailing" gap="1" align="center" nowrap>
              <if {canAttachCrop()}>
                <button
                  type="button"
                  size="sm"
                  variant="ghost"
                  attach-crop
                  on:click={onAttachCrop}
                >
                  Attach crop
                </button>
              </if>
            </row>
          </sail-composer>
        </column>
      </if>
    </column>
  </row>
</column>
