
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
  file?: string
  span?: AnchorSpan
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
  anchor?: TaskAnchor | null
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
  created_by?: Actor
  assignee?: Actor | null
  /** Last comment|created|agent_note actor — matches ThreadList actor filter. */
  last_actor?: Actor | null
  route?: string | null
  anchor?: TaskAnchor | null
  artifact_count?: number
  drift?: DriftInfo | null
}

export type ActorFilter = 'all' | 'human' | 'agent'
export type DriftFilter = 'all' | 'ok' | 'moved' | 'lost'
export type OpenFilter = 'open' | 'all'

export type TasksOpsClient = {
  sailTaskThreadList?: (args: {
    project?: string
    status?: string
    actor?: string
    drift?: string
    open?: boolean
  }) => Promise<{ ok: boolean; tasks: unknown[] }>
  sailTaskList?: (args: {
    project?: string
    status?: string
    actor?: string
    drift?: string
    open?: boolean
  }) => Promise<{ ok: boolean; tasks: unknown[] }>
  sailTaskGet?: (args: {
    project?: string
    id: string
  }) => Promise<{ ok: boolean; task: unknown }>
  sailTaskThreadCreate?: (args: {
    project?: string
    title: string
    body?: string
    anchor: Record<string, unknown>
    actor?: { kind?: string; name?: string }
  }) => Promise<{ ok?: boolean; task?: unknown }>
  sailTaskCreate?: (args: {
    project?: string
    title: string
    body?: string
    anchor?: Record<string, unknown>
    actor?: { kind?: string; name?: string }
  }) => Promise<{ ok?: boolean; task?: unknown }>
  sailTaskThreadReply?: (args: {
    project?: string
    id: string
    body: string
    actor?: { kind?: string; name?: string }
  }) => Promise<{ ok?: boolean; event?: unknown; task?: unknown }>
  sailTaskComment?: (args: {
    project?: string
    id: string
    body: string
    actor?: { kind?: string; name?: string }
  }) => Promise<{ ok?: boolean; event?: unknown; task?: unknown }>
  sailTaskAddArtifact?: (args: {
    project?: string
    id: string
    filename?: string
    content_base64?: string
    mime?: string
    kind?: string
    label?: string
    region?: Record<string, unknown>
    actor?: { kind?: string; name?: string }
  }) => Promise<{ ok?: boolean; artifact_id?: string; path?: string; task?: unknown }>
  sailTaskReanchor?: (args: {
    project?: string
    id: string
    anchor: Record<string, unknown>
    actor?: { kind?: string; name?: string }
  }) => Promise<{ ok?: boolean; task?: unknown }>
  sailTaskCommitIntent?: (args: {
    project?: string
    id: string
    actor?: { kind?: string; name?: string }
  }) => Promise<{ ok?: boolean; path?: string; bytes?: number; task?: unknown }>
  sailTaskAgentNote?: (args: {
    project?: string
    id: string
    body: string
    actor?: { kind?: string; name?: string }
  }) => Promise<{ ok?: boolean; event?: unknown; task?: unknown }>
}

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
