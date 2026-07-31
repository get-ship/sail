import type {
  DriftInfo,
  TaskAnchor,
  TaskArtifact,
  TaskEvent,
  TaskThread,
  ThreadListItem,
} from './types'

export function parseDrift(raw: unknown): DriftInfo | null {
  if (!raw || typeof raw !== 'object') return null
  const o = raw as Record<string, unknown>
  const status = String(o.status ?? 'none') as DriftInfo['status']
  return {
    status,
    span: (o.span as DriftInfo['span']) ?? null,
    liveNodeId: o.liveNodeId != null ? String(o.liveNodeId) : null,
    liveHash: o.liveHash != null ? String(o.liveHash) : null,
    spanDisplay: o.spanDisplay != null ? String(o.spanDisplay) : null,
  }
}

export function parseAnchor(raw: unknown): TaskAnchor | null {
  if (!raw || typeof raw !== 'object') return null
  const o = raw as Record<string, unknown>
  const nodeId = String(o.nodeId ?? o.node_id ?? '')
  if (!nodeId) return null
  return {
    nodeId,
    elementPath: String(o.elementPath ?? o.element_path ?? ''),
    contentHash: String(o.contentHash ?? o.content_hash ?? ''),
    file: o.file != null ? String(o.file) : undefined,
    span: o.span as TaskAnchor['span'],
  }
}

export function parseActor(raw: unknown): { kind: string; name: string } | null {
  if (!raw || typeof raw !== 'object') return null
  const o = raw as Record<string, unknown>
  return {
    kind: String(o.kind ?? 'human'),
    name: String(o.name ?? (o.kind === 'agent' ? 'agent' : 'user')),
  }
}

export function parseListItem(raw: unknown): ThreadListItem | null {
  if (!raw || typeof raw !== 'object') return null
  const o = raw as Record<string, unknown>
  const id = String(o.id ?? '')
  if (!id) return null
  return {
    id,
    title: String(o.title ?? id),
    status: String(o.status ?? 'todo'),
    updated_at: String(o.updated_at ?? ''),
    labels: Array.isArray(o.labels) ? o.labels.map(String) : [],
    created_by: parseActor(o.created_by) ?? undefined,
    assignee: parseActor(o.assignee),
    last_actor: parseActor(o.last_actor) ?? parseActor(o.lastActor),
    route: o.route != null ? String(o.route) : null,
    anchor: parseAnchor(o.anchor),
    artifact_count: typeof o.artifact_count === 'number' ? o.artifact_count : undefined,
    drift: parseDrift(o.drift),
  }
}

export function parseThread(raw: unknown): TaskThread | null {
  if (!raw || typeof raw !== 'object') return null
  const o = raw as Record<string, unknown>
  const id = String(o.id ?? '')
  if (!id) return null
  const events: TaskEvent[] = Array.isArray(o.events)
    ? o.events.map((e) => {
        const ev = e as Record<string, unknown>
        return {
          id: String(ev.id ?? ''),
          at: String(ev.at ?? ''),
          type: String(ev.type ?? ''),
          actor: (ev.actor as TaskEvent['actor']) ?? { kind: 'human', name: 'user' },
          body: ev.body != null ? String(ev.body) : null,
          meta: (ev.meta as Record<string, unknown>) ?? null,
        }
      })
    : []
  const artifacts: TaskArtifact[] = Array.isArray(o.artifacts)
    ? o.artifacts.map((a) => {
        const ar = a as Record<string, unknown>
        return {
          id: String(ar.id ?? ''),
          path: String(ar.path ?? ''),
          mime: String(ar.mime ?? ''),
          kind: String(ar.kind ?? 'file'),
          label: ar.label != null ? String(ar.label) : null,
          region: (ar.region as Record<string, unknown>) ?? null,
          created_at: String(ar.created_at ?? ''),
        }
      })
    : []
  return {
    id,
    title: String(o.title ?? id),
    status: String(o.status ?? 'todo'),
    labels: Array.isArray(o.labels) ? o.labels.map(String) : [],
    created_at: String(o.created_at ?? ''),
    updated_at: String(o.updated_at ?? ''),
    created_by: (o.created_by as TaskThread['created_by']) ?? {
      kind: 'human',
      name: 'user',
    },
    assignee: (o.assignee as TaskThread['assignee']) ?? null,
    route: o.route != null ? String(o.route) : null,
    anchor: parseAnchor(o.anchor),
    artifacts,
    events,
    drift: parseDrift(o.drift),
  }
}
