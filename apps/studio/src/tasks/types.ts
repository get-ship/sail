import type { StudioOpsMethods } from '@ship/sail-studio-client'

export type ActorKind = 'human' | 'agent'

export type DriftStatus = 'ok' | 'moved' | 'lost' | 'none'

export type TaskStatus =
  | 'backlog'
  | 'todo'
  | 'in_progress'
  | 'review'
  | 'done'
  | 'cancelled'

export type AnchorSpan = {
  start: number
  end: number
  line?: number
  col?: number
  end_line?: number
  end_col?: number
}

export type TaskAnchor = {
  nodeId: string
  elementPath: string
  contentHash: string
  file?: string | undefined
  span?: AnchorSpan | undefined
}

export type DriftInfo = {
  status: DriftStatus
  span?: AnchorSpan | null
  liveNodeId?: string | null
  liveHash?: string | null
  spanDisplay?: string | null
}

export type Actor = { kind: ActorKind | string; name: string }

export type TaskEvent = {
  id: string
  at: string
  type: string
  actor: Actor
  body?: string | null
  meta?: Record<string, unknown> | null
}

export type TaskArtifact = {
  id: string
  path: string
  mime: string
  kind: 'file' | 'render' | string
  label?: string | null
  region?: Record<string, unknown> | null
  created_at: string
}

export type TaskThread = {
  id: string
  title: string
  status: TaskStatus | string
  labels: string[]
  created_at: string
  updated_at: string
  created_by: Actor
  assignee?: Actor | null
  route?: string | null
  anchor?: TaskAnchor | null | undefined
  artifacts: TaskArtifact[]
  events: TaskEvent[]
  drift?: DriftInfo | null
}

export type ThreadListItem = {
  id: string
  title: string
  status: string
  updated_at: string
  labels: string[]
  created_by?: Actor | undefined
  assignee?: Actor | null
  /** Last comment|created|agent_note actor — matches ThreadList actor filter. */
  last_actor?: Actor | null
  route?: string | null
  anchor?: TaskAnchor | null | undefined
  artifact_count?: number | undefined
  drift?: DriftInfo | null
}

export type ActorFilter = 'all' | 'human' | 'agent'
export type DriftFilter = 'all' | 'ok' | 'moved' | 'lost'
export type OpenFilter = 'open' | 'all'

/** Tasks/Work surface client — derived from generated ops (no hand-written shapes). */
export type TasksOpsClient = Partial<
  Pick<
    StudioOpsMethods,
    | 'sailTaskThreadList'
    | 'sailTaskList'
    | 'sailTaskGet'
    | 'sailTaskThreadCreate'
    | 'sailTaskCreate'
    | 'sailTaskThreadReply'
    | 'sailTaskComment'
    | 'sailTaskAddArtifact'
    | 'sailTaskReanchor'
    | 'sailTaskCommitIntent'
    | 'sailTaskAgentNote'
  >
>

export type TasksSurfaceState = {
  projectRoot: string
  threads: ThreadListItem[]
  selectedId: string | null
  selected: TaskThread | null
  openFilter: OpenFilter
  actorFilter: ActorFilter
  driftFilter: DriftFilter
  replyDraft: string
  loading: boolean
  error: string | null
  commitPath: string | null
  /** Last batch create count for UI chip. */
  lastBatchCount: number
}
