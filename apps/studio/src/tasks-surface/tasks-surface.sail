<!--
  <tasks-surface> — thread list + conversation + composer.
-->
<scope>
  style surfaceRoot = { box: grow(1).minH(0).h(full).minW(0).overflow(hidden) }
  style pageHeader = { box: shrink(0).h(10).padX(4), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1) }
  style growMin = { box: grow(1).minW(0) }
  style shrink0 = { box: shrink(0) }
  style titleMd = { text: fontSize(md).fontWeight(semibold) }
  style countMono = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style filterOpen = { box: w(24) }
  style filterActor = { box: w(28) }
  style errorBand = { box: shrink(0).padX(3).h(7), paint: borderBWidth(1).borderB(border) }
  style errorText = { box: grow(1).minW(0), paint: fg(destructive), text: fontSize(xs).truncate() }
  style workbench = { box: grow(1).minH(0).basis(0).h(full).minW(0).overflow(hidden) }
  style navPane = { box: grow(1).minH(0).h(full).minW(panel-min-w-md).maxW(panel-max-w-md).basis(0).overflow(hidden), paint: borderRWidth(1).borderR(border-subtle).bg(surface-1) }
  style scrollGrow = { box: grow(1).minH(0).basis(0).h(full) }
  style listPad = { box: padY(1).padX(1) }
  style detailPane = { box: grow(1).minW(0).minH(0).h(full).basis(0).overflow(hidden), paint: bg(surface-0) }
  style emptyCenter = { box: grow(1).minH(0).h(full).pad(4) }
  style fullW = { box: w(full) }
  style detailHead = { box: shrink(0).h(10).padX(3).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1) }
  style titleTruncate = { text: fontSize(md).truncate() }
  style anchorRow = { box: shrink(0).padX(3).padY(2).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(surface-0) }
  style pathMono = { box: grow(1).minW(0), text: fontSize(xs).fontFamily(mono).truncate() }
  style attachRow = { box: shrink(0).padX(3).padY(2), paint: borderBWidth(1).borderB(border-subtle) }
  style conversation = { box: grow(1).minH(0).basis(0).h(full).overflow(hidden) }
  style mdScroll = { box: shrink(0).maxH(32).minH(0) }
  style mdPad = { box: pad(3) }
  style messagesHost = { box: grow(1).minH(0).h(full).w(full).basis(0) }
  style composer = { box: shrink(0).pad(3), paint: borderTWidth(1).borderT(border-subtle).bg(surface-1) }
</scope>

<controller>
import type { UiEvent } from '../compositions/ui-event'
import { createTasksSurface } from '../compositions/tasks-surface/tasks-surface-vm'
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
    onTaskThreadClick: (id: string) => c.onTaskThreadClick(id),
    onTaskOpenFilter: (e: UiEvent) => c.onTaskOpenFilter(e),
    onTaskActorFilter: (e: UiEvent) => c.onTaskActorFilter(e),
    onComposerSubmit: (detail: { text?: string; attachments?: unknown[] }) => c.onComposerSubmit(detail),
    onCloseDetail: () => c.onCloseDetail(),
    onNewTaskQuick: () => c.onNewTaskQuick(),
    onRetryTasks: () => c.onRetryTasks(),
    onAttachCrop: () => c.onAttachCrop(),
  }
})
</controller>

<column surface="tasks" tasks-surface gap="0" style(surfaceRoot)>
  <row tasks-header align="center" justify="between" gap="3" nowrap style(pageHeader)>
    <row align="center" gap="3" nowrap style(growMin)>
      <p style(titleMd)>Tasks</p>
      <p thread-count style(countMono)>
        {threadCountLabel()}
      </p>
      <select
        filter-open
        size="sm"
        value={openFilterVal()}
        on:change={onTaskOpenFilter}
        label="Open or all" style(filterOpen)>
        <option value="open">Open</option>
        <option value="all">All</option>
      </select>
      <select
        filter-actor
        size="sm"
        value={actorFilterVal()}
        on:change={onTaskActorFilter}
        label="Last reply" style(filterActor)>
        <option value="all">Anyone</option>
        <option value="human">You</option>
        <option value="agent">Agent</option>
      </select>
    </row>
    <button type="button" size="sm" tone="primary" new-task on:click={onNewTaskQuick} style(shrink0)>
      New task
    </button>
  </row>

  <if {showTaskErrorAlert()}>
    <row tasks-error align="center" gap="2" style(errorBand)>
      <p style(errorText)>{taskError()}</p>
      <button type="button" size="sm" variant="ghost" retry-tasks on:click={onRetryTasks}>
        Retry
      </button>
    </row>
  </if>

  <row tasks-workbench gap="0" nowrap align="stretch" style(workbench)>
    <column threads-list tasks-nav gap="0" style(navPane)>
      <scroll-area style(scrollGrow)>
        <column gap="0" tasks-table style(listPad)>
          <if {!hasTaskThreads()}>
            <if {!showTaskErrorAlert()}>
              <studio-empty tasks-empty title={emptyTitle()} size="sm">
                {emptyBody()}
              </studio-empty>
            </if>
          </if>
          <if {hasTaskThreads()}>
            <for {taskThreadRows() as row} key={row.key}>
              <studio-list-row
                thread-row
                drift={row.drift}
                title={row.title}
                meta={row.updatedLabel}
                selected={row.selected}
                on:click={() => onTaskThreadClick(row.id)}
              >
                <badge slot="leading" status-badge={row.status} tone={row.statusTone} size="sm" variant="soft" pill label={row.statusLabel} style(shrink0)>
                  {row.statusLabel}
                </badge>
              </studio-list-row>
            </for>
          </if>
        </column>
      </scroll-area>
    </column>

    <column task-detail gap="0" style(detailPane)>
      <if {!hasTaskSelected()}>
        <column align="center" justify="center" style(emptyCenter)>
          <empty-state variant="placeholder" size="sm" task-detail-empty style(fullW)>
            <strong slot="title">Select a task</strong>
            Or create a new one.
          </empty-state>
        </column>
      </if>

      <if {hasTaskSelected()}>
        <row task-detail-header align="center" gap="2" nowrap justify="between" style(detailHead)>
          <row align="center" gap="2" nowrap style(growMin)>
            <strong style(titleTruncate)>{taskTitle()}</strong>
            <badge status-detail tone={taskStatusTone()} size="sm" variant="soft" pill>
              {taskStatusLabel()}
            </badge>
          </row>
          <button type="button" size="sm" variant="ghost" tone="muted" on:click={onCloseDetail}>
            Close
          </button>
        </row>

        <if {taskAnchorHas()}>
          <row anchor-card anchor-chip drift={taskAnchorStatusRaw()} align="center" gap="2" nowrap style(anchorRow)>
            <badge anchor-drift={taskAnchorStatusRaw()} tone={taskAnchorTone()} size="sm" variant="soft" pill style(shrink0)>
              {taskAnchorStatus()}
            </badge>
            <p style(pathMono)>
              {taskAnchorPath()}
            </p>
          </row>
        </if>

        <if {hasTaskAttach()}>
          <row attachments gap="2" align="center" style(attachRow)>
            <for {taskAttachChips() as a} key={a.key}>
              <badge attach-chip attach-kind={a.kind} tone="muted" size="sm" variant="outline" pill>
                {a.label}
              </badge>
            </for>
          </row>
        </if>

        <column task-conversation gap="0" style(conversation)>
          <if {hasThreadMarkdown()}>
            <scroll-area style(mdScroll)>
              <column style(mdPad)>
                <sail-markdown thread-markdown value={threadMarkdown()}></sail-markdown>
              </column>
            </scroll-area>
          </if>
          <sail-message-list messages-host messages sticky-bottom style(messagesHost)>
            <empty-state slot="empty" variant="inline" size="sm" style(fullW)>
              <strong slot="title">No messages yet</strong>
              Write below to start.
            </empty-state>
          </sail-message-list>
        </column>

        <column reply-composer gap="2" style(composer)>
          <sail-composer
            task-composer
            reply-input
            multiline
            disabled={composerDisabled()}
            placeholder="Message…"
            on-submit={onComposerSubmit} style(fullW)>
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
