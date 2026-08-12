import { getBatchItems } from '../vendor/studio-shell'
import { getTasksState } from './store'
import { artifactThumbSrc } from './thumbs'

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
        const last = out[out.length - 1]
        if (last) {
          last.artifacts.push({
            id: e.id,
            label,
            kind,
            key: e.id,
          })
        }
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

/** Items for `<sail-message-list items>` — conversation turns. */
export type MessageListItem = {
  id: string
  content: string
  from: 'user' | 'assistant' | 'system'
  contentType: string
  timestamp: string
  status: 'sent' | 'sending' | 'error'
}

/**
 * Project selected-thread events into chat-elements message-list items.
 * Agent → assistant, human → user. Artifact-only turns become system notes.
 */
export function uiMessageListItems(): MessageListItem[] {
  const conv = uiConversation()
  return conv.map((m) => {
    let content = m.body
    if (m.hasArtifacts) {
      const labels = m.artifacts.map((a) => a.label).join(', ')
      content = content
        ? `${content}\n\n_Attachments: ${labels}_`
        : `_Attachments: ${labels}_`
    }
    const from: MessageListItem['from'] =
      m.actorKind === 'agent' ? 'assistant' : m.actorKind === 'you' ? 'user' : 'system'
    return {
      id: m.id,
      content: content || '…',
      from,
      contentType: 'text',
      timestamp: m.time,
      status: 'sent',
    }
  })
}

/**
 * Markdown for selected-thread bodies (create + every comment/agent_note).
 * `sail-message-list` owns the bubble slot as plain text (API has no rich body);
 * this card is the reuse-map `sail-markdown` path for the same prose.
 */
export function uiThreadMarkdown(): string {
  const t = getTasksState().selected
  if (!t) return ''
  const lines: string[] = []
  lines.push(`## ${t.title}`)
  for (const e of t.events) {
    if (!e.body?.trim()) continue
    if (e.type === 'created') {
      lines.push(String(e.body))
      continue
    }
    if (e.type === 'comment' || e.type === 'agent_note') {
      const who = e.actor?.kind === 'agent' ? 'Agent' : 'You'
      lines.push(`### ${who}`)
      lines.push(String(e.body))
    }
  }
  if (t.anchor) {
    const span =
      t.drift?.spanDisplay ??
      (t.anchor.span
        ? `${t.anchor.span.line ?? t.anchor.span.start}–${t.anchor.span.end_line ?? t.anchor.span.end}`
        : null)
    lines.push(
      `\n\`${t.anchor.elementPath || t.anchor.nodeId}\`${span ? ` · \`${span}\`` : ''}`,
    )
  }
  return lines.filter(Boolean).join('\n\n')
}

export function uiAnchorCard(): {
  has: boolean
  span: string
  status: string
  statusRaw: string
  /** Badge tone: ok quiet, moved warning, lost error. */
  statusTone: string
  path: string
  fileSpan: string
  /** Show "Accept new location" when moved with a live match. */
  showAcceptLive: boolean
  /** Show "Re-link to selection" when lost or moved. */
  showRebind: boolean
  liveNodeId: string
  movedHint: string
} {
  const selected = getTasksState().selected
  const a = selected?.anchor
  const d = selected?.drift
  if (!a) {
    return {
      has: false,
      span: '—',
      status: 'Not linked',
      statusRaw: 'none',
      statusTone: 'muted',
      path: '—',
      fileSpan: '—',
      showAcceptLive: false,
      showRebind: false,
      liveNodeId: '',
      movedHint: '',
    }
  }
  const span =
    d?.spanDisplay ??
    (a.span
      ? `${a.span.line ?? a.span.start}–${a.span.end_line ?? a.span.end}`
      : '—')
  const status = d?.status ?? 'ok'
  const file = a.file ? a.file.split('/').pop() : ''
  const fileSpan = file ? `${file}:${span}` : String(span)
  let movedHint = ''
  if (status === 'moved' && d?.liveNodeId) {
    movedHint = `Now at ${d.liveNodeId}`
  } else if (status === 'moved') {
    movedHint = 'Location changed — accept or re-link'
  } else if (status === 'lost') {
    movedHint = 'Element gone — re-link to selection'
  }
  return {
    has: true,
    span,
    status: driftLabel(status),
    statusRaw: status,
    statusTone: driftTone(status),
    path: a.elementPath || a.nodeId,
    fileSpan,
    showAcceptLive: status === 'moved' && Boolean(d?.liveNodeId),
    showRebind: status === 'lost' || status === 'moved',
    liveNodeId: d?.liveNodeId ?? '',
    movedHint,
  }
}

export function driftTone(status: string): string {
  switch (status) {
    case 'ok':
      return 'muted'
    case 'moved':
      return 'warning'
    case 'lost':
      return 'destructive'
    default:
      return 'muted'
  }
}

export type AttachChipUi = {
  id: string
  label: string
  key: string
  kind: string
  isRender: boolean
  isDiff: boolean
  fromText: string
  toText: string
  /** data: URL for evidence thumb when crop bytes were cached at attach. */
  thumbSrc: string
  hasThumb: boolean
}

export function uiAttachmentChips(): AttachChipUi[] {
  const arts = getTasksState().selected?.artifacts ?? []
  return arts.map((a) => {
    const region = (a.region ?? {}) as Record<string, unknown>
    const fromText =
      region.from != null
        ? String(region.from)
        : region.fromText != null
          ? String(region.fromText)
          : ''
    const toText =
      region.to != null
        ? String(region.to)
        : region.toText != null
          ? String(region.toText)
          : ''
    const isDiff = Boolean(fromText || toText) || a.kind === 'diff'
    const isRender = a.kind === 'render' || (a.mime || '').startsWith('image/')
    const thumbSrc = isRender ? artifactThumbSrc(a.id, a.path) : ''
    return {
      id: a.id,
      label: a.label || a.path.split('/').pop() || a.id,
      key: a.id,
      kind: String(a.kind || 'file'),
      isRender,
      isDiff,
      fromText,
      toText,
      thumbSrc,
      hasThumb: Boolean(thumbSrc),
    }
  })
}

/** First diff-shaped attachment for `<sail-diff from to>`. */
export function uiAttachmentDiff(): { has: boolean; from: string; to: string } {
  const chip = uiAttachmentChips().find((c) => c.isDiff && (c.fromText || c.toText))
  if (!chip) return { has: false, from: '', to: '' }
  return { has: true, from: chip.fromText, to: chip.toText }
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
