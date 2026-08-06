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

<column surface="tasks" tasks-surface gap="0">
  style({  box: grow(1).minH(0).h(full).minW(0).overflow(hidden)  })

  <row tasks-header align="center" justify="between" gap="3" nowrap>
    style({  box: shrink(0).h(10).padX(4), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1)  })

    <row align="center" gap="3" nowrap>
      style({ box: grow(1).minW(0) })

      <p>
        style({ text: fontSize(md).fontWeight(semibold) })
Tasks</p>
      <p thread-count>
        style({ paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) })

        {threadCountLabel()}
      </p>
      <select
        filter-open
        size="sm"
        value={openFilterVal()}
        on:change={onTaskOpenFilter}
        label="Open or all"

      >
        style({ box: w(24) })

        <option value="open">Open</option>
        <option value="all">All</option>
      </select>
      <select
        filter-actor
        size="sm"
        value={actorFilterVal()}
        on:change={onTaskActorFilter}
        label="Last reply"

      >
        style({ box: w(28) })

        <option value="all">Anyone</option>
        <option value="human">You</option>
        <option value="agent">Agent</option>
      </select>
    </row>
    <button type="button" size="sm" tone="primary" new-task on:click={onNewTaskQuick}>
      style({ box: shrink(0) })

      New task
    </button>
  </row>

  <if {showTaskErrorAlert()}>
    <row tasks-error align="center" gap="2">
      style({  box: shrink(0).padX(3).h(7), paint: borderBWidth(1).borderB(border)  })

      <p>
        style({ box: grow(1).minW(0), paint: fg(destructive), text: fontSize(xs).truncate() })
{taskError()}</p>
      <button type="button" size="sm" variant="ghost" retry-tasks on:click={onRetryTasks}>
        Retry
      </button>
    </row>
  </if>

  <row tasks-workbench gap="0" nowrap align="stretch">
    style({  box: grow(1).minH(0).basis(0).h(full).minW(0).overflow(hidden)  })

    <column threads-list tasks-nav gap="0">
      style({  box: grow(1).minH(0).h(full).minW(panel-min-w-md).maxW(panel-max-w-md).basis(0).overflow(hidden), paint: borderRWidth(1).borderR(border-subtle).bg(surface-1)  })

      <scroll-area on:click={onTaskThreadClick}>
        style({  box: grow(1).minH(0).basis(0).h(full)  })

        <column gap="0" tasks-table>
          style({ box: padY(1).padX(1) })

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
                <badge slot="leading" status-badge={row.status} tone={row.statusTone} size="sm" variant="soft" pill label={row.statusLabel}>
                  style({ box: shrink(0) })

                  {row.statusLabel}
                </badge>
              </studio-list-row>
            </for>
          </if>
        </column>
      </scroll-area>
    </column>

    <column task-detail gap="0">
      style({  box: grow(1).minW(0).minH(0).h(full).basis(0).overflow(hidden), paint: bg(surface-0)  })

      <if {!hasTaskSelected()}>
        <column align="center" justify="center">
          style({  box: grow(1).minH(0).h(full).pad(4)  })

          <empty-state variant="placeholder" size="sm" task-detail-empty>
            style({ box: w(full) })

            <strong slot="title">Select a task</strong>
            Or create a new one.
          </empty-state>
        </column>
      </if>

      <if {hasTaskSelected()}>
        <row task-detail-header align="center" gap="2" nowrap justify="between">
          style({  box: shrink(0).h(10).padX(3).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1)  })

          <row align="center" gap="2" nowrap>
            style({ box: grow(1).minW(0) })

            <strong>
              style({ text: fontSize(md).truncate() })
{taskTitle()}</strong>
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
          <row anchor-card anchor-chip drift={taskAnchorStatusRaw()} align="center" gap="2" nowrap>
            style({  box: shrink(0).padX(3).padY(2).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(surface-0)  })

            <badge anchor-drift={taskAnchorStatusRaw()} tone={taskAnchorTone()} size="sm" variant="soft" pill>
              style({ box: shrink(0) })

              {taskAnchorStatus()}
            </badge>
            <p>
              style({ box: grow(1).minW(0), text: fontSize(xs).fontFamily(mono).truncate() })

              {taskAnchorPath()}
            </p>
          </row>
        </if>

        <if {hasTaskAttach()}>
          <row attachments gap="2" align="center">
            style({  box: shrink(0).padX(3).padY(2), paint: borderBWidth(1).borderB(border-subtle)  })

            <for {taskAttachChips()} as="a" key={a.key}>
              <badge attach-chip attach-kind={a.kind} tone="muted" size="sm" variant="outline" pill>
                {a.label}
              </badge>
            </for>
          </row>
        </if>

        <column task-conversation gap="0">
          style({  box: grow(1).minH(0).basis(0).h(full).overflow(hidden)  })

          <if {hasThreadMarkdown()}>
            <scroll-area>
              style({  box: shrink(0).maxH(32).minH(0)  })

              <column>
                style({ box: pad(3) })

                <sail-markdown thread-markdown value={threadMarkdown()}>
                  <span>
                    style({ box: display(contents), text: fontSize(sm) })
                    
                  </span>
</sail-markdown>
              </column>
            </scroll-area>
          </if>
          <sail-message-list messages-host messages sticky-bottom>
            style({  box: grow(1).minH(0).h(full).w(full).basis(0)  })

            <empty-state slot="empty" variant="inline" size="sm">
              style({ box: w(full) })

              <strong slot="title">No messages yet</strong>
              Write below to start.
            </empty-state>
          </sail-message-list>
        </column>

        <column reply-composer gap="2">
          style({  box: shrink(0).pad(3), paint: borderTWidth(1).borderT(border-subtle).bg(surface-1)  })

          <sail-composer
            task-composer
            reply-input
            multiline
            disabled={composerDisabled()}
            placeholder="Message…"
            on:sail-composer-submit={onComposerSubmit}

          >
            style({ box: w(full) })

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
