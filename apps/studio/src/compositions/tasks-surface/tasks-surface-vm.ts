/**
 * Tasks surface VM — thread monitoring + chat-elements detail
 * (sail-message-list · sail-composer · sail-markdown · sail-diff).
 */
import { mountAmbient } from '../../vendor/runtime'
import { getBatchItems, getSelection, getShellState } from '../../vendor/studio-shell'
import { acceptLiveNode, assignComposerAttachments, assignMessageListHost, attachLatestCrop, commitIntent, createThread, getTasksState, loadThreads, reanchorToNode, selectThread, sendReply, setActorFilter, setDriftFilter, setOpenFilter, setReplyDraft, subscribeTasks, uiAnchorCard, uiAttachmentChips, uiAttachmentCount, uiAttachmentDiff, uiEmptyCopy, uiHeader as uiTasksHeader, uiMessageListItems, uiThreadMarkdown, uiThreadRows, type ActorFilter, type DriftFilter, type OpenFilter } from '../../tasks/index.ts'
import { getEditState } from '../../edit/index.ts'

import { signal, debounce } from '../../vendor/runtime'
type TaskThreadUi = {
  id: string
  title: string
  pathLabel: string
  status: string
  statusLabel: string
  statusTone: string
  actorChip: string
  driftChip: string
  drift: string
  showDrift: boolean
  updatedLabel: string
  selected: boolean
  key: string
  /** Resolved (done) threads collapse to a quieter row. */
  collapsed: boolean
}

type AttachChipUi = {
  id: string
  label: string
  key: string
  kind: string
  isRender: boolean
  isDiff: boolean
  thumbSrc: string
  hasThumb: boolean
}

export function createTasksSurface() {
  const taskThreadRows = signal<TaskThreadUi[]>([])
  const hasTaskThreads = signal(false)
  const threadCountLabel = signal('0 threads')
  const taskTitle = signal('Tasks')
  const taskDrift = signal('Not linked')
  const taskStatusLabel = signal('To do')
  const taskStatusTone = signal('muted')
  const hasTaskSelected = signal(false)
  const taskAnchorHas = signal(false)
  const taskAnchorStatus = signal('Not linked')
  const taskAnchorStatusRaw = signal('none')
  const taskAnchorTone = signal('muted')
  const taskAnchorPath = signal('—')
  const taskAnchorFileSpan = signal('—')
  const taskAnchorMovedHint = signal('')
  const taskAttachChips = signal<AttachChipUi[]>([])
  const hasTaskAttach = signal(false)
  const taskError = signal('')
  /** Alert only when list has rows; empty list uses empty-state for load errors. */
  const showTaskErrorAlert = signal(false)
  const taskCommitPath = signal('')
  const hasTaskCommit = signal(false)
  const openFilterVal = signal('open')
  const actorFilterVal = signal('all')
  const driftFilterVal = signal('all')
  const attachCountLabel = signal('0')
  const showAcceptLive = signal(false)
  const showRebind = signal(false)
  const emptyTitle = signal('No open work')
  const emptyBody = signal('Start a task when you want help on a part of the app.')
  const threadMarkdown = signal('')
  const hasThreadMarkdown = signal(false)
  /** True when conversation has turns — drives message-list height (empty shrinks). */
  const hasMessageItems = signal(false)
  const hasTaskDiff = signal(false)
  const diffFrom = signal('')
  const diffTo = signal('')
  const canAttachCrop = signal(false)
  /** Composer submit lifecycle: idle | sending | sent | error */
  const composerStatus = signal<'idle' | 'sending' | 'sent' | 'error'>('idle')
  const composerStatusLabel = signal('')
  const composerDisabled = signal(true)
  const showComposerStatus = signal(false)

  const pushChatHosts = () => {
    const items = uiMessageListItems()
    hasMessageItems.set(items.length > 0)
    queueMicrotask(() => {
      assignMessageListHost('[messages-host]', items)
      assignComposerAttachments('[task-composer]', [])
    })
  }

  const sync = () => {
    const ts = getTasksState()
    const rows = uiThreadRows().map((r) => ({
      ...r,
      collapsed: r.status === 'done' || r.status === 'cancelled',
    }))
    taskThreadRows.set(rows)
    hasTaskThreads.set(rows.length > 0)
    const h = uiTasksHeader()
    taskTitle.set(h.title)
    taskDrift.set(h.drift)
    taskStatusLabel.set(h.statusLabel)
    taskStatusTone.set(h.statusTone)
    hasTaskSelected.set(h.hasSelected)
    threadCountLabel.set(h.threadCountLabel)
    const a = uiAnchorCard()
    taskAnchorHas.set(a.has)
    taskAnchorStatus.set(a.status)
    taskAnchorStatusRaw.set(a.statusRaw)
    taskAnchorTone.set(a.statusTone)
    taskAnchorPath.set(a.path)
    taskAnchorFileSpan.set(a.fileSpan)
    taskAnchorMovedHint.set(a.movedHint)
    showAcceptLive.set(a.showAcceptLive)
    showRebind.set(a.showRebind)
    const chips = uiAttachmentChips()
    taskAttachChips.set(chips)
    hasTaskAttach.set(chips.length > 0)
    attachCountLabel.set(String(uiAttachmentCount()))
    const d = uiAttachmentDiff()
    hasTaskDiff.set(d.has)
    diffFrom.set(d.from)
    diffTo.set(d.to)
    const md = uiThreadMarkdown()
    threadMarkdown.set(md)
    hasThreadMarkdown.set(md.trim().length > 0 && h.hasSelected)
    taskError.set(ts.error ?? '')
    showTaskErrorAlert.set(!!ts.error)
    taskCommitPath.set(ts.commitPath ?? '')
    hasTaskCommit.set(!!ts.commitPath)
    openFilterVal.set(ts.openFilter)
    actorFilterVal.set(ts.actorFilter)
    driftFilterVal.set(ts.driftFilter)
    const batchN = getBatchItems().length
    canAttachCrop.set(batchN > 0 && h.hasSelected)
    composerDisabled.set(!h.hasSelected || composerStatus() === 'sending')
    if (ts.error) {
      emptyTitle.set('No open work')
      emptyBody.set('Start a task when you want help on a part of the app.')
    } else {
      const empty = uiEmptyCopy()
      emptyTitle.set(empty.title)
      emptyBody.set(empty.body)
    }
    pushChatHosts()
  }

  sync()

  // Shell owns initial loadThreads + rail refresh; this surface only mirrors store.
  mountAmbient(() => {
    const unsub = subscribeTasks(() => sync())
    if (getTasksState().threads.length === 0 && !getTasksState().loading) {
      void loadThreads().then(() => sync())
    }
    queueMicrotask(() => pushChatHosts())
    return () => unsub()
  })

  const onTaskThreadClick = (id: string) => {
    if (!id) return
    composerStatus.set('idle')
    composerStatusLabel.set('')
    showComposerStatus.set(false)
    void selectThread(id).then(() => sync())
  }

  const onTaskOpenFilter = (e: Event) => {
    setOpenFilter((e.target as HTMLSelectElement).value as OpenFilter)
    sync()
  }

  const onTaskActorFilter = (e: Event) => {
    setActorFilter((e.target as HTMLSelectElement).value as ActorFilter)
    sync()
  }

  const onTaskDriftFilter = (e: Event) => {
    setDriftFilter((e.target as HTMLSelectElement).value as DriftFilter)
    sync()
  }

  const clearSentStatus = debounce(() => {
    if (composerStatus() === 'sent') {
      composerStatus.set('idle')
      composerStatusLabel.set('')
      showComposerStatus.set(false)
      sync()
    }
  }, 1200)

  const onComposerSubmit = (detail: { text?: string; attachments?: unknown[] }) => {
    const text = (detail?.text ?? '').trim()
    if (!text || !getTasksState().selectedId) return
    setReplyDraft(text)
    composerStatus.set('sending')
    composerStatusLabel.set('Sending…')
    showComposerStatus.set(true)
    composerDisabled.set(true)
    void sendReply()
      .then(() => {
        const err = getTasksState().error
        if (err) {
          composerStatus.set('error')
          composerStatusLabel.set('Could not send — try again')
        } else {
          composerStatus.set('sent')
          composerStatusLabel.set('Sent')
          clearSentStatus()
        }
        sync()
      })
      .catch(() => {
        composerStatus.set('error')
        composerStatusLabel.set('Could not send — try again')
        showComposerStatus.set(true)
        sync()
      })
  }

  const onTaskCommit = () => {
    void commitIntent().then(() => sync())
  }

  const onCloseDetail = () => {
    void selectThread(null).then(() => {
      composerStatus.set('idle')
      composerStatusLabel.set('')
      showComposerStatus.set(false)
      sync()
    })
  }

  const onNewTaskQuick = () => {
    const sel = getShellState().selection.nodeId
    const path =
      getEditState().activePath ?? getShellState().sailPath ?? 'routes/page.sail'
    void createThread({
      title: 'New task',
      body: 'Started from Tasks',
      anchor: {
        nodeId: sel || `${path}#0#new`,
        elementPath: sel
          ? `${path.split('/').pop()} › ${sel}`
          : `${path.split('/').pop()} › root`,
        contentHash: (sel || 'new').split('#').pop() || 'new',
        file: path.endsWith('.sail') ? path : undefined,
      },
    }).then(() => sync())
  }

  const onRetryTasks = () => {
    void loadThreads().then(() => sync())
  }

  const onAcceptLiveNode = () => {
    void acceptLiveNode().then(() => sync())
  }

  const onRebindSelection = () => {
    const nodeId = getSelection().nodeId ?? getShellState().selection.nodeId
    if (!nodeId) {
      void reanchorToNode({ nodeId: '' }).then(() => sync())
      return
    }
    const path = getEditState().activePath ?? getShellState().sailPath ?? ''
    void reanchorToNode({
      nodeId,
      elementPath: path ? `${path.split('/').pop()} › ${nodeId}` : nodeId,
      file: path.endsWith('.sail') ? path : undefined,
    }).then(() => sync())
  }

  const onAttachCrop = () => {
    void attachLatestCrop().then(() => sync())
  }

  return {
    taskThreadRows,
    hasTaskThreads,
    threadCountLabel,
    taskTitle,
    taskDrift,
    taskStatusLabel,
    taskStatusTone,
    hasTaskSelected,
    taskAnchorHas,
    taskAnchorStatus,
    taskAnchorStatusRaw,
    taskAnchorTone,
    taskAnchorPath,
    taskAnchorFileSpan,
    taskAnchorMovedHint,
    taskAttachChips,
    hasTaskAttach,
    taskError,
    showTaskErrorAlert,
    taskCommitPath,
    hasTaskCommit,
    openFilterVal,
    actorFilterVal,
    driftFilterVal,
    attachCountLabel,
    showAcceptLive,
    showRebind,
    emptyTitle,
    emptyBody,
    threadMarkdown,
    hasThreadMarkdown,
    hasMessageItems,
    hasTaskDiff,
    diffFrom,
    diffTo,
    canAttachCrop,
    composerStatusLabel,
    composerDisabled,
    showComposerStatus,
    onTaskThreadClick,
    onTaskOpenFilter,
    onTaskActorFilter,
    onTaskDriftFilter,
    onComposerSubmit,
    onTaskCommit,
    onCloseDetail,
    onNewTaskQuick,
    onRetryTasks,
    onAcceptLiveNode,
    onRebindSelection,
    onAttachCrop,
  }
}
