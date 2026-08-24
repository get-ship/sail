<!--
  <tasks-surface> — thread list + conversation + composer.
-->
<scope>
  style surfaceRoot = { overflow(hidden) }
  // no preset: size lock + edge-only border
  style pageHeader = { size({ h: 10 }).pad(4).border(subtle-b).bg(surface) }
  style growMin = { size(grow) }
  style shrink0 = { size(hug) }
  style titleMd = { type(heading) }
  style countMono = { fg(muted).type(mono-sm) }
  // no preset: size lock
  style filterOpen = { size({ w: 24 }) }
  // no preset: size lock
  style filterActor = { size({ w: 28 }) }
  // no preset: size lock + edge-only border
  style errorBand = { pad(3).size({ h: 7 }).border(default-b) }
  style errorText = { size(grow).fg(danger).type(body-sm) }
  style workbench = { size(grow).overflow(hidden) }
  // no preset: edge-only border
  style navPane = { size(grow).overflow(hidden).border(subtle-r).bg(surface) }
  style scrollGrow = { size(grow) }
  style listPad = { pad(1) }
  style detailPane = { size(grow).overflow(hidden).bg(app) }
  style emptyCenter = { size(max).pad(4) }
  style fullW = { size(max) }
  // no preset: min-w lock + edge-only border
  style detailHead = { size({ h: 10, min-w: 0 }).pad(3).border(subtle-b).bg(surface) }
  style titleTruncate = { type(body) }
  // no preset: min-w lock + edge-only border
  style anchorRow = { pad(3).size({ min-w: 0 }).border(subtle-b).bg(app) }
  style pathMono = { size(grow).type(mono-sm) }
  // no preset: edge-only border
  style attachRow = { size(hug).pad(3).border(subtle-b) }
  style conversation = { size(max).overflow(hidden) }
  // no preset: size lock
  style mdScroll = { size({ min-h: 0, max-h: 32 }) }
  style mdPad = { pad(3) }
  style messagesHost = { size(grow) }
  // no preset: edge-only border
  style composer = { size(hug).pad(3).border(subtle-t).bg(surface) }
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
