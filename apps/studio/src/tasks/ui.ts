import { getBatchItems } from '@ship/sail-studio-shell'
import { getTasksState } from './store'

/** Chip label matching actor filter: last conversational actor. */
function actorChipFromKind(kind: string | undefined | null): string {
  return kind === 'agent' ? 'agent' : 'you'
}

export function statusLabel(status: string): string {
  switch (status) {
    case 'in_progress':
      return 'In progress'
    case 'todo':
      return 'To do'
    case 'backlog':
      return 'Backlog'
    case 'review':
      return 'Review'
    case 'done':
      return 'Done'
    case 'cancelled':
      return 'Cancelled'
    default:
      return status || 'To do'
  }
}

export function statusTone(status: string): string {
  switch (status) {
    case 'in_progress':
      return 'primary'
    case 'review':
      return 'warning'
    case 'done':
      return 'success'
    case 'cancelled':
      return 'muted'
    default:
      return 'muted'
  }
}

export function formatTime(iso: string): string {
  // Keep short local-ish clock from ISO without locale deps.
  const m = iso.match(/T(\d{2}):(\d{2})/)
  if (!m) return iso.slice(11, 16) || iso || '—'
  const h = Number(m[1])
  const min = m[2]
  const ampm = h >= 12 ? 'pm' : 'am'
  const h12 = h % 12 || 12
  return `${h12}:${min}${ampm}`
}

export type ThreadRowUi = {
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

export function uiThreadRows(): ThreadRowUi[] {
  const state = getTasksState()
  return state.threads.map((t) => {
    // Prefer ops last_actor (same rule as ThreadList actor filter).
    const kind = t.last_actor?.kind ?? t.created_by?.kind ?? 'human'
    const lastActor = actorChipFromKind(kind)
    const drift = t.drift?.status ?? 'none'
    const span =
      t.drift?.spanDisplay ??
      (t.anchor?.span
        ? `${t.anchor.span.line ?? t.anchor.span.start}–${t.anchor.span.end_line ?? t.anchor.span.end}`
        : '')
    const pathLabel = t.anchor
      ? `${t.anchor.elementPath}${span ? ` · :${span}` : ''}`
      : (t.route ?? '—')
    const status = t.status || 'todo'
    return {
      id: t.id,
      title: t.title,
      pathLabel,
      status,
      statusLabel: statusLabel(status),
      statusTone: statusTone(status),
      actorChip: lastActor,
      driftChip: driftLabel(drift),
      drift,
      showDrift: drift !== 'none' && drift !== 'ok',
      updatedLabel: formatTime(t.updated_at),
      selected: t.id === state.selectedId,
      key: t.id,
    }
  })
}

export type EventUi = {
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

export function uiConversation(): EventUi[] {
  const t = getTasksState().selected
  if (!t) return []
  const out: EventUi[] = []
  for (const e of t.events) {
    if (!['comment', 'agent_note', 'created', 'artifact_added'].includes(e.type)) {
      continue
    }
    if (e.type === 'created' && !e.body) continue
    if (e.type === 'artifact_added') {
      const label = e.meta?.label
        ? String(e.meta.label)
        : e.meta?.path
          ? String(e.meta.path)
          : 'attachment'
      const kind = e.meta?.kind ? String(e.meta.kind) : 'file'
      if (out.length) {
        out[out.length - 1].artifacts.push({
          id: e.id,
          label,
          kind,
          key: e.id,
        })
        continue
      }
    }
    const actorKind = e.actor?.kind === 'agent' ? 'agent' : 'you'
    out.push({
      id: e.id,
      actorLabel: actorKind,
      actorKind,
      time: formatTime(e.at),
      body: e.body ?? '',
      hasBody: Boolean(e.body),
      hasArtifacts: false,
      artifacts: [],
      key: e.id,
    })
  }
  for (const m of out) m.hasArtifacts = m.artifacts.length > 0
  return out
}

export function uiAnchorCard(): {
  has: boolean
  span: string
  status: string
  path: string
  /** Show "Accept new location" when moved with a live match. */
  showAcceptLive: boolean
  /** Show "Re-link to selection" when lost or moved. */
  showRebind: boolean
  liveNodeId: string
} {
  const selected = getTasksState().selected
  const a = selected?.anchor
  const d = selected?.drift
  if (!a) {
    return {
      has: false,
      span: '—',
      status: 'none',
      path: '—',
      showAcceptLive: false,
      showRebind: false,
      liveNodeId: '',
    }
  }
  const span =
    d?.spanDisplay ??
    (a.span
      ? `${a.span.line ?? a.span.start}–${a.span.end_line ?? a.span.end}`
      : '—')
  const status = d?.status ?? 'ok'
  return {
    has: true,
    span,
    status: driftLabel(status),
    path: a.elementPath,
    showAcceptLive: status === 'moved' && Boolean(d?.liveNodeId),
    showRebind: status === 'lost' || status === 'moved',
    liveNodeId: d?.liveNodeId ?? '',
  }
}

export function uiAttachmentChips(): { id: string; label: string; key: string }[] {
  const arts = getTasksState().selected?.artifacts ?? []
  return arts.map((a) => ({
    id: a.id,
    label: a.label || a.path.split('/').pop() || a.id,
    key: a.id,
  }))
}

/** Selected thread artifact count (not batch tray). */
export function uiAttachmentCount(): number {
  return getTasksState().selected?.artifacts?.length ?? 0
}

export function uiHeader(): {
  title: string
  drift: string
  status: string
  statusLabel: string
  statusTone: string
  hasSelected: boolean
  threadCountLabel: string
} {
  const state = getTasksState()
  const status = state.selected?.status ?? 'todo'
  const n = state.threads.length
  return {
    title: state.selected?.title ?? 'Tasks',
    drift: driftLabel(state.selected?.drift?.status ?? 'none'),
    status,
    statusLabel: statusLabel(status),
    statusTone: statusTone(status),
    hasSelected: Boolean(state.selected),
    threadCountLabel: n === 1 ? '1 thread' : `${n} threads`,
  }
}

export function uiBatchSummary(): { count: number; label: string } {
  const n = getBatchItems().length
  return {
    count: n,
    label: n === 1 ? 'New task (1 item)' : `New task (${n} items)`,
  }
}

/** Human label for linked-UI location status (avoids raw "drift" jargon). */
export function driftLabel(status: string): string {
  switch (status) {
    case 'ok':
      return 'In place'
    case 'moved':
      return 'Moved'
    case 'lost':
      return 'Lost'
    case 'none':
      return 'Not linked'
    default:
      return status || 'Not linked'
  }
}

/**
 * Empty-list copy for the monitoring table.
 * Default open filter is not a "filter miss" — that would hide genuine empty.
 */
export function uiEmptyCopy(): { title: string; body: string } {
  const state = getTasksState()
  if (state.error) {
    return {
      title: 'Could not load tasks',
      body: 'Try again in a moment.',
    }
  }
  // Narrowing beyond default open/all is a real filter miss.
  if (state.actorFilter !== 'all' || state.driftFilter !== 'all') {
    return {
      title: 'No matching threads',
      body: 'Try a different filter, or start a new task.',
    }
  }
  if (state.openFilter === 'open') {
    return {
      title: 'No open work',
      body: 'Start a task when you want help on a part of the app.',
    }
  }
  return {
    title: 'No agent work yet',
    body: 'Start a task when you want help on a part of the app.',
  }
}
