/**
 * Tasks surface VM — agent monitoring table + thread detail dialog.
 * Ops client is wired by page.sail on studio mount.
 */
import { mountAmbient, signal } from '@ship/sail-runtime'
import {
  acceptLiveNode,
  commitIntent,
  createThread,
  getTasksState,
  loadThreads,
  reanchorToNode,
  selectThread,
  sendReply,
  setActorFilter,
  setDriftFilter,
  setOpenFilter,
  setReplyDraft,
  subscribeTasks,
  uiAnchorCard,
  uiAttachmentChips,
  uiAttachmentCount,
  uiConversation,
  uiEmptyCopy,
  uiHeader as uiTasksHeader,
  uiThreadRows,
  type ActorFilter,
  type DriftFilter,
  type OpenFilter,
} from '@ship/studio/tasks'
import { getEditState } from '@ship/studio/edit'
import { getSelection, getShellState } from '@ship/sail-studio-shell'

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
}

type TaskEventUi = {
  id: string
  actorLabel: string
  actorKind: string
  time: string
  body: string
  hasBody: boolean
  hasArtifacts: boolean
  artifacts: { id: string; label: string; kind: string; key: string }[]
  key: string
}

export function createTasksSurface() {
  const taskThreadRows = signal<TaskThreadUi[]>([])
  const hasTaskThreads = signal(false)
  const threadCountLabel = signal('0 threads')
  const taskConv = signal<TaskEventUi[]>([])
  const taskTitle = signal('Tasks')
  const taskDrift = signal('Not linked')
  const taskStatusLabel = signal('To do')
  const taskStatusTone = signal('muted')
  const hasTaskSelected = signal(false)
  const taskReply = signal('')
  const taskAnchorHas = signal(false)
  const taskAnchorSpan = signal('—')
  const taskAnchorStatus = signal('Not linked')
  const taskAnchorPath = signal('—')
  const taskAttachChips = signal<{ id: string; label: string; key: string }[]>([])
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

  const sync = () => {
    const ts = getTasksState()
    const rows = uiThreadRows()
    taskThreadRows.set(rows)
    hasTaskThreads.set(rows.length > 0)
    taskConv.set(uiConversation())
    const h = uiTasksHeader()
    taskTitle.set(h.title)
    taskDrift.set(h.drift)
    taskStatusLabel.set(h.statusLabel)
    taskStatusTone.set(h.statusTone)
    hasTaskSelected.set(h.hasSelected)
    threadCountLabel.set(h.threadCountLabel)
    taskReply.set(ts.replyDraft)
    const a = uiAnchorCard()
    taskAnchorHas.set(a.has)
    taskAnchorSpan.set(a.span)
    taskAnchorStatus.set(a.status)
    taskAnchorPath.set(a.path)
    showAcceptLive.set(a.showAcceptLive)
    showRebind.set(a.showRebind)
    const chips = uiAttachmentChips()
    taskAttachChips.set(chips)
    hasTaskAttach.set(chips.length > 0)
    attachCountLabel.set(String(uiAttachmentCount()))
    taskError.set(ts.error ?? '')
    // Error strip always when store.error is set; empty-state is neutral only.
    showTaskErrorAlert.set(!!ts.error)
    taskCommitPath.set(ts.commitPath ?? '')
    hasTaskCommit.set(!!ts.commitPath)
    openFilterVal.set(ts.openFilter)
    actorFilterVal.set(ts.actorFilter)
    driftFilterVal.set(ts.driftFilter)
    // Empty copy ignores error titles when we show the error strip separately.
    if (ts.error) {
      emptyTitle.set('No open work')
      emptyBody.set('Start a task when you want help on a part of the app.')
    } else {
      const empty = uiEmptyCopy()
      emptyTitle.set(empty.title)
      emptyBody.set(empty.body)
    }
  }

  sync()

  // Shell owns initial loadThreads + rail refresh; this surface only mirrors store.
  mountAmbient(() => {
    const unsub = subscribeTasks(() => sync())
    if (getTasksState().threads.length === 0 && !getTasksState().loading) {
      void loadThreads().then(() => sync())
    }
    return () => unsub()
  })

  const onTaskThreadClick = (e: Event) => {
    const t = (e.target as Element | null)?.closest?.('[data-task-id]') as HTMLElement | null
    if (!t?.dataset.taskId) return
    void selectThread(t.dataset.taskId).then(() => sync())
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

  const onTaskReplyInput = (e: Event) => {
    setReplyDraft((e.target as HTMLInputElement).value)
    sync()
  }

  const onTaskSend = () => {
    void sendReply().then(() => sync())
  }

  const onTaskCommit = () => {
    void commitIntent().then(() => sync())
  }

  const onCloseDetail = () => {
    void selectThread(null).then(() => sync())
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

  return {
    taskThreadRows,
    hasTaskThreads,
    threadCountLabel,
    taskConv,
    taskTitle,
    taskDrift,
    taskStatusLabel,
    taskStatusTone,
    hasTaskSelected,
    taskReply,
    taskAnchorHas,
    taskAnchorSpan,
    taskAnchorStatus,
    taskAnchorPath,
    taskAttachChips,
    hasTaskAttach,
    taskError,
    showTaskErrorAlert,
    taskCommitPath,
    hasTaskCommit,
    openFilterVal,
    actorFilterVal,
    onRetryTasks,
    driftFilterVal,
    attachCountLabel,
    showAcceptLive,
    showRebind,
    emptyTitle,
    emptyBody,
    onTaskThreadClick,
    onTaskOpenFilter,
    onTaskActorFilter,
    onTaskDriftFilter,
    onTaskReplyInput,
    onTaskSend,
    onTaskCommit,
    onCloseDetail,
    onNewTaskQuick,
    onAcceptLiveNode,
    onRebindSelection,
  }
}
