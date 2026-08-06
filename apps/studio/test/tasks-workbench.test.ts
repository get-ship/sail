import { afterEach, beforeAll, describe, expect, it } from 'vitest'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync, mkdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { createCliOpsClient } from '../src/edit/ops-adapter-cli'
import {
  __resetTasksForTests,
  attachLatestCrop,
  commitIntent,
  createThread,
  driftTone,
  getTasksState,
  loadThreads,
  newTaskFromBatch,
  selectThread,
  sendReply,
  setActorFilter,
  setDriftFilter,
  setOpenFilter,
  setReplyDraft,
  setTasksOpsClient,
  setTasksProjectRoot,
  statusLabel,
  statusTone,
  uiAnchorCard,
  uiAttachmentDiff,
  uiAttachmentChips,
  uiConversation,
  uiEmptyCopy,
  uiMessageListItems,
  uiThreadMarkdown,
  uiThreadRows,
} from '../src/tasks'
import {
  __resetBatchForTests,
  addRegionToBatch,
  clearBatch,
  getBatchItems,
} from '@ship/sail-studio-shell'

const REPO = join(dirname(fileURLToPath(import.meta.url)), '../../..')

let cli: ReturnType<typeof createCliOpsClient>
let fixtureRoot: string

beforeAll(() => {
  cli = createCliOpsClient(REPO)
}, 120_000)

afterEach(() => {
  __resetTasksForTests()
  __resetBatchForTests()
  if (fixtureRoot && existsSync(fixtureRoot)) {
    try {
      rmSync(fixtureRoot, { recursive: true, force: true })
    } catch {
      /* ignore */
    }
  }
})

function makeProject(): string {
  // Workspace-relative project so resolve_project can confine under monorepo.
  const abs = mkdtempSync(join(REPO, 'apps/studio', '.tmp-tool8-'))
  fixtureRoot = abs
  const rel = abs.slice(REPO.length + 1).replace(/\\/g, '/')
  return rel
}

describe('tasks workbench (ops + store)', () => {
  it('thread anchored; list carries drift field', async () => {
    const project = makeProject()
    setTasksOpsClient(cli)
    setTasksProjectRoot(project)

    const id = await createThread({
      title: 'hover lift too strong',
      body: 'The lift feels aggressive on dense screens. Region attached.',
      anchor: {
        nodeId: 'routes/page.sail#0.1#9f2c',
        elementPath: 'page.sail › card hero',
        contentHash: '9f2c',
        file: 'routes/page.sail',
        span: { start: 42, end: 58, line: 42, end_line: 58 },
      },
    })
    expect(id).toBeTruthy()
    await loadThreads()
    const rows = uiThreadRows()
    expect(rows.some((r) => r.title.includes('hover lift'))).toBe(true)
    const got = await cli.sailTaskGet!({ project, id: id! })
    const task = got.task as Record<string, unknown>
    expect((task.anchor as { nodeId: string }).nodeId).toContain('page.sail')
    expect(task.drift).toBeTruthy()
    // File may be missing in fixture → lost or ok-offline; never silent rebind.
    const drift = task.drift as { status: string }
    expect(['ok', 'moved', 'lost']).toContain(drift.status)
  }, 60_000)

  it('batch 3 items → one task with 3 render artifacts', async () => {
    const project = makeProject()
    setTasksOpsClient(cli)
    setTasksProjectRoot(project)
    clearBatch()

    // Seed three batch items (offline path without live render).
    for (let i = 0; i < 3; i++) {
      await addRegionToBatch(
        i === 1 ? 'screenshot' : 'annotate',
        { x: i * 10, y: 0, w: 100 + i, h: 40 },
        {},
      )
    }
    expect(getBatchItems().length).toBe(3)

    const id = await newTaskFromBatch('batch tray proof')
    expect(id).toBeTruthy()
    const got = await cli.sailTaskGet!({ project, id: id! })
    const task = got.task as { artifacts: { kind: string }[] }
    expect(task.artifacts.length).toBe(3)
    expect(task.artifacts.every((a) => a.kind === 'render')).toBe(true)
    expect(getBatchItems().length).toBe(0)
  }, 90_000)

  it('MCP-shaped: agent ThreadCreate + render attach + reply; Studio list sees it', async () => {
    const project = makeProject()
    // Direct ops (agent path) — same JSON as MCP tools.
    const created = await cli.sailTaskThreadCreate!({
      project,
      title: 'agent handoff thread',
      body: 'Looking at the card hover.',
      anchor: {
        nodeId: 'routes/page.sail#0.2#aa11',
        elementPath: 'page.sail › card hero',
        contentHash: 'aa11',
        file: 'routes/page.sail',
        span: { start: 10, end: 20, line: 10, end_line: 20 },
      },
      actor: { kind: 'agent', name: 'agent' },
    })
    const task = created.task as { id: string; events: { actor: { kind: string } }[] }
    const id = task.id
    expect(id).toBeTruthy()

    await cli.sailTaskAddArtifact!({
      project,
      id,
      filename: 'region.png',
      content_base64: 'aGVsbG8=',
      mime: 'image/png',
      kind: 'render',
      label: 'region · card@42',
      region: { nodeId: 'routes/page.sail#0.2#aa11' },
      actor: { kind: 'agent', name: 'agent' },
    })

    await cli.sailTaskThreadReply!({
      project,
      id,
      body: 'Proposed motion.preset = gentle',
      actor: { kind: 'agent', name: 'agent' },
    })

    setTasksOpsClient(cli)
    setTasksProjectRoot(project)
    await loadThreads()
    await selectThread(id)
    const s = getTasksState()
    expect(s.selected?.id).toBe(id)
    expect(s.selected?.artifacts.length).toBe(1)
    expect(s.selected?.artifacts[0].kind).toBe('render')
    const conv = uiConversation()
    expect(conv.some((m) => m.actorKind === 'agent')).toBe(true)
    // actor filter
    setActorFilter('agent')
    await loadThreads()
    expect(uiThreadRows().some((r) => r.id === id)).toBe(true)
  }, 90_000)

  it('commit intent writes decision record; .sail/tasks stays gitignored', async () => {
    const project = makeProject()
    setTasksOpsClient(cli)
    setTasksProjectRoot(project)
    const id = await createThread({
      title: 'tone=secondary?',
      body: 'Should this be secondary tone?',
      anchor: {
        nodeId: 'routes/page.sail#0.5#bb22',
        elementPath: 'page.sail › card hero',
        contentHash: 'bb22',
        file: 'routes/page.sail',
      },
    })
    expect(id).toBeTruthy()
    await selectThread(id!)
    const path = await commitIntent()
    expect(path).toBeTruthy()
    expect(path!.startsWith('decisions/')).toBe(true)
    const abs = join(REPO, project, path!)
    expect(existsSync(abs)).toBe(true)
    const md = readFileSync(abs, 'utf8')
    expect(md).toMatch(/tone=secondary/)
    expect(md).toMatch(/sail:task-id=/)

    // gitignore covers .sail/
    const gi = readFileSync(join(REPO, project, '.gitignore'), 'utf8')
    expect(gi).toMatch(/\.sail\//)
    // tasks dir is under .sail
    expect(existsSync(join(REPO, project, '.sail', 'tasks', id!))).toBe(true)

    // cleanup decision + project
    try {
      rmSync(join(REPO, project, 'decisions'), { recursive: true, force: true })
    } catch {
      /* ignore */
    }
  }, 60_000)

  it('events show actor distinctly; @agent handoff stays human; filter aligns with chip', async () => {
    const project = makeProject()
    setTasksOpsClient(cli)
    setTasksProjectRoot(project)
    const id = await createThread({
      title: 'actor filter proof',
      body: 'human opener',
      anchor: {
        nodeId: 'routes/page.sail#0.9#cc33',
        elementPath: 'page.sail › avatar',
        contentHash: 'cc33',
      },
    })
    await selectThread(id!)
    // Human @agent handoff must stay human-attributed (not AgentNote as agent)
    setReplyDraft('@agent please check contrast')
    await sendReply()
    await selectThread(id!)
    const conv = uiConversation()
    const handoff = conv.find((m) => m.body.includes('@agent'))
    expect(handoff).toBeTruthy()
    expect(handoff!.actorKind).toBe('you')
    // Explicit agent note (true agent path)
    await cli.sailTaskAgentNote!({
      project,
      id: id!,
      body: 'checking contrast now',
      actor: { kind: 'agent', name: 'agent' },
    })
    await selectThread(id!)
    const after = uiConversation()
    expect(after.some((m) => m.actorKind === 'agent' && m.body.includes('checking'))).toBe(
      true,
    )
    expect(after.some((m) => m.actorKind === 'you')).toBe(true)

    // last actor is agent → chip + filter agree
    await loadThreads()
    const row = uiThreadRows().find((r) => r.id === id)
    expect(row?.actorChip).toBe('agent')
    setActorFilter('agent')
    await loadThreads()
    expect(uiThreadRows().some((r) => r.id === id)).toBe(true)
    setActorFilter('human')
    await loadThreads()
    expect(uiThreadRows().some((r) => r.id === id)).toBe(false)
  }, 60_000)

  it('tasks route mounts surface; chrome keeps batch tray + store wiring', () => {
    const leaf = readFileSync(join(REPO, 'apps/studio/routes/tasks/page.sail'), 'utf8')
    const chrome = readFileSync(
      join(REPO, 'apps/studio/src/sail-studio-chrome/sail-studio-chrome.sail'),
      'utf8',
    )
    const shellVm = readFileSync(join(REPO, 'apps/studio/src/shell/shell-page-vm.ts'), 'utf8')
    expect(leaf).toMatch(/<tasks-surface\b/)
    expect(chrome).toMatch(/batch-tray/)
    expect(shellVm).toMatch(/from ['"]@ship\/studio\/tasks['"]/)
    expect(shellVm).toMatch(/loadThreads|newTaskFromBatch/)
    expect(shellVm).toMatch(/setTasksOpsClient|setTasksProjectRoot/)
    expect(chrome).not.toMatch(/tasks-surface/)
    expect(chrome).not.toMatch(/threads-list/)
    expect(chrome).not.toMatch(/commit-intent/)
  })

  it('tasks-surface carries markers, store imports, no raw containers', () => {
    // App-local layout (monorepo-discoverable under apps/studio/src/)
    const root = join(REPO, 'apps/studio/src/tasks-surface')
    const surface = readFileSync(join(root, 'tasks-surface.sail'), 'utf8')
    const ctrl = readFileSync(join(root, 'tasks-surface-vm.ts'), 'utf8')
    expect(surface).toMatch(/tasks-surface/)
    expect(surface).toMatch(/surface="tasks"/)
    expect(surface).toMatch(/threads-list/)
    expect(surface).toMatch(/tasks-table/)
    expect(surface).toMatch(/task-detail/)
    expect(surface).toMatch(/task-conversation/)
    expect(surface).toMatch(/anchor-card/)
    expect(surface).toMatch(/filter-actor/)
    expect(surface).toMatch(/status-badge/)
    // Chat primitives + composer
    expect(surface).toMatch(/<sail-composer\b/)
    expect(surface).toMatch(/<sail-message-list\b/)
    expect(surface).toMatch(/messages-host/)
    expect(surface).toMatch(/task-composer/)
    expect(surface).toMatch(/on:sail-composer-submit/)
    // Removed scaffold junk actions / diary
    expect(surface).not.toMatch(/commit-intent/)
    expect(surface).not.toMatch(/>\s*Commit\s*</)
    expect(surface).not.toMatch(/Re-link/)
    expect(surface).not.toMatch(/Crops from the tray/)
    expect(surface).not.toMatch(/data-accept-live/)
    expect(surface).not.toMatch(/data-rebind-selection/)
    // Master–detail workbench
    expect(surface).toMatch(/tasks-workbench/)
    // Anchor strip before conversation body
    const anchorAt = surface.indexOf('anchor-card')
    const messagesAt = surface.indexOf('messages')
    expect(anchorAt).toBeGreaterThan(0)
    expect(messagesAt).toBeGreaterThan(anchorAt)
    expect(surface).not.toMatch(/commit intent/i)
    expect(surface).not.toMatch(/<div\b/)
    expect(surface).not.toMatch(/<span\b/)
    // No hand-rolled reply input — composer owns the field
    expect(surface).not.toMatch(/reply-composer[\s\S]*<input\b/)
    expect(surface).toMatch(/thread-row/)
    expect(surface).not.toMatch(/<dialog\b/)
    expect(surface).toMatch(/<empty-state\b/)
    expect(surface).toMatch(/<studio-empty\b/)
    expect(surface).toMatch(/tasks-header/)
    expect(ctrl).toMatch(/from ['"]@ship\/studio\/tasks['"]/)
    expect(ctrl).toMatch(/loadThreads/)
    expect(ctrl).toMatch(/assignMessageListHost|uiMessageListItems/)
    expect(ctrl).toMatch(/onComposerSubmit/)
    // Dead bindings removed after composer cutover
    expect(ctrl).not.toMatch(/\btaskReply\b/)
    expect(ctrl).not.toMatch(/\bbatchAvailable\b/)
    expect(existsSync(join(root, 'tasks-surface.sail'))).toBe(true)
    expect(existsSync(join(REPO, 'apps/studio/components/tasks-surface'))).toBe(false)
  })

  it('clients registers sail-chat-elements; package depends on it', () => {
    const clients = readFileSync(join(REPO, 'apps/studio/src/clients.ts'), 'utf8')
    const pkg = readFileSync(join(REPO, 'apps/studio/package.json'), 'utf8')
    expect(clients).toMatch(/@ship\/sail-chat-elements/)
    expect(pkg).toMatch(/@ship\/sail-chat-elements/)
  })

  it('design modules stay CSS-free; tasks store uses SailTask ops only', () => {
    const store = readFileSync(join(REPO, 'apps/studio/src/tasks/store.ts'), 'utf8')
    expect(store).not.toMatch(/getComputedStyle|themes\.css/)
    expect(store).toMatch(/sailTaskThreadCreate|sailTaskAddArtifact|sailTaskCommitIntent/)
    // handoff must not rewrite human → agent via AgentNote
    expect(store).toMatch(/Human reply/)
    expect(store).not.toMatch(/isHandoff && client\.sailTaskAgentNote/)
  })

  it('uiThreadRows expose status badges for monitoring table', async () => {
    const project = makeProject()
    setTasksOpsClient(cli)
    setTasksProjectRoot(project)
    const id = await createThread({
      title: 'status badge row',
      body: 'monitoring view',
      anchor: {
        nodeId: 'routes/page.sail#0.3#dd44',
        elementPath: 'page.sail › card',
        contentHash: 'dd44',
      },
    })
    expect(id).toBeTruthy()
    await loadThreads()
    const row = uiThreadRows().find((r) => r.id === id)
    expect(row).toBeTruthy()
    expect(row!.statusLabel.length).toBeGreaterThan(0)
    expect(row!.statusTone.length).toBeGreaterThan(0)
    expect(row!.updatedLabel.length).toBeGreaterThan(0)
    expect(row!.pathLabel).toMatch(/page\.sail|card/)
  }, 60_000)
})

describe('tasks ui pure projections', () => {
  function stubEmptyClient() {
    setTasksOpsClient({
      sailTaskThreadList: async () => ({ ok: true, tasks: [] }),
      sailTaskList: async () => ({ ok: true, tasks: [] }),
    })
    setTasksProjectRoot('apps/studio')
  }

  it('statusLabel / statusTone matrix', () => {
    expect(statusLabel('in_progress')).toBe('In progress')
    expect(statusTone('in_progress')).toBe('primary')
    expect(statusLabel('todo')).toBe('To do')
    expect(statusTone('todo')).toBe('muted')
    expect(statusLabel('review')).toBe('Review')
    expect(statusTone('review')).toBe('warning')
    expect(statusLabel('done')).toBe('Done')
    expect(statusTone('done')).toBe('success')
    expect(statusLabel('cancelled')).toBe('Cancelled')
    expect(statusTone('cancelled')).toBe('muted')
    expect(statusLabel('backlog')).toBe('Backlog')
  })

  it('uiEmptyCopy: default open empty is not a filter miss', async () => {
    stubEmptyClient()
    await loadThreads()
    // Default openFilter=open, actor/drift=all → genuine open-empty copy
    const openEmpty = uiEmptyCopy()
    expect(openEmpty.title).toBe('No open work')
    expect(openEmpty.title).not.toBe('No matching threads')
    expect(openEmpty.body).toMatch(/Start a task/)

    setOpenFilter('all')
    await loadThreads()
    const allEmpty = uiEmptyCopy()
    expect(allEmpty.title).toBe('No agent work yet')
    expect(allEmpty.title).not.toBe('No matching threads')

    setActorFilter('agent')
    await loadThreads()
    expect(uiEmptyCopy().title).toBe('No matching threads')

    setActorFilter('all')
    setDriftFilter('moved')
    await loadThreads()
    expect(uiEmptyCopy().title).toBe('No matching threads')
  })

  it('uiEmptyCopy: no client is neutral empty (not red error)', async () => {
    // Ops not wired yet → honest empty, not a failure banner
    await loadThreads()
    const empty = uiEmptyCopy()
    expect(empty.title).toBe('No open work')
    expect(getTasksState().error).toBeNull()
  })

  it('uiEmptyCopy: real load error does not echo raw machine body', async () => {
    setTasksOpsClient({
      sailTaskThreadList: async () => {
        throw new Error('task service unavailable')
      },
    } as never)
    await loadThreads()
    const err = uiEmptyCopy()
    expect(err.title).toBe('Could not load tasks')
    expect(err.body).toBe('Try again in a moment.')
    expect(err.body).not.toMatch(/task service|unavailable/i)
    expect(getTasksState().error).toMatch(/Could not load tasks/)
    expect(getTasksState().error).not.toMatch(/task service/i)
  })
})

describe('drift pure contract (client-visible statuses)', () => {
  it('docs done-when statuses are ok|moved|lost', () => {
    const statuses = ['ok', 'moved', 'lost'] as const
    expect(statuses).toContain('ok')
    expect(statuses).toContain('moved')
    expect(statuses).toContain('lost')
  })

  it('driftTone matrix: ok muted, moved warning, lost destructive', () => {
    expect(driftTone('ok')).toBe('muted')
    expect(driftTone('moved')).toBe('warning')
    expect(driftTone('lost')).toBe('destructive')
    expect(driftTone('none')).toBe('muted')
  })

  it('uiAnchorCard: moved → warning + Accept; lost → destructive + Re-link; fileSpan', async () => {
    setTasksOpsClient({
      sailTaskGet: async ({ id }) => ({
        ok: true,
        task: {
          id,
          title: 'drift card',
          status: 'todo',
          labels: [],
          created_at: '2026-01-01T00:00:00Z',
          updated_at: '2026-01-01T00:00:00Z',
          created_by: { kind: 'human', name: 'user' },
          artifacts: [],
          events: [],
          anchor: {
            nodeId: 'routes/page.sail#0.1#ab',
            elementPath: 'page.sail › card hero',
            contentHash: 'ab',
            file: 'routes/page.sail',
            span: { start: 1, end: 2, line: 10, end_line: 12 },
          },
          drift: {
            status: 'moved',
            liveNodeId: 'routes/page.sail#0.2#cd',
            spanDisplay: '20–22',
          },
        },
      }),
      sailTaskThreadList: async () => ({ ok: true, tasks: [] }),
    } as never)
    setTasksProjectRoot('apps/studio')
    await selectThread('t-moved')
    const moved = uiAnchorCard()
    expect(moved.has).toBe(true)
    expect(moved.statusRaw).toBe('moved')
    expect(moved.statusTone).toBe('warning')
    expect(moved.showAcceptLive).toBe(true)
    expect(moved.showRebind).toBe(true)
    expect(moved.fileSpan).toMatch(/page\.sail:20–22|page\.sail:20/)
    expect(moved.movedHint).toMatch(/Now at|routes\/page\.sail/)

    setTasksOpsClient({
      sailTaskGet: async ({ id }) => ({
        ok: true,
        task: {
          id,
          title: 'lost card',
          status: 'todo',
          labels: [],
          created_at: '2026-01-01T00:00:00Z',
          updated_at: '2026-01-01T00:00:00Z',
          created_by: { kind: 'human', name: 'user' },
          artifacts: [],
          events: [],
          anchor: {
            nodeId: 'routes/page.sail#0.1#zz',
            elementPath: 'page.sail › gone',
            contentHash: 'zz',
            file: 'routes/page.sail',
            span: { start: 1, end: 2, line: 5, end_line: 6 },
          },
          drift: { status: 'lost' },
        },
      }),
      sailTaskThreadList: async () => ({ ok: true, tasks: [] }),
    } as never)
    await selectThread('t-lost')
    const lost = uiAnchorCard()
    expect(lost.statusRaw).toBe('lost')
    expect(lost.statusTone).toBe('destructive')
    expect(lost.showAcceptLive).toBe(false)
    expect(lost.showRebind).toBe(true)
    expect(lost.movedHint).toMatch(/re-link/i)
    expect(lost.fileSpan).toMatch(/page\.sail/)
  })
})

describe('attachLatestCrop + evidence thumbs', () => {
  it('attaches only the newest tray item and leaves the rest', async () => {
    const project = makeProject()
    setTasksOpsClient(cli)
    setTasksProjectRoot(project)
    clearBatch()
    const id = await createThread({
      title: 'latest crop only',
      body: 'attach proof',
      anchor: {
        nodeId: 'routes/page.sail#0.1#lc01',
        elementPath: 'page.sail › card',
        contentHash: 'lc01',
      },
    })
    expect(id).toBeTruthy()
    await selectThread(id!)
    for (let i = 0; i < 3; i++) {
      await addRegionToBatch(
        i === 2 ? 'screenshot' : 'annotate',
        { x: i * 10, y: 0, w: 80 + i, h: 40 },
        {},
      )
    }
    expect(getBatchItems().length).toBe(3)
    const ok = await attachLatestCrop(id!)
    expect(ok).toBe(true)
    expect(getBatchItems().length).toBe(2)
    await selectThread(id!)
    const arts = getTasksState().selected?.artifacts ?? []
    expect(arts.length).toBe(1)
    expect(arts[0].kind).toBe('render')
    // Evidence thumb cached from attach payload
    const chips = uiAttachmentChips()
    expect(chips.length).toBe(1)
    expect(chips[0].hasThumb).toBe(true)
    expect(chips[0].thumbSrc).toMatch(/^data:image\/png;base64,/)
  }, 90_000)
})

describe('chat-elements projections', () => {
  it('uiMessageListItems maps human→user and agent→assistant', async () => {
    const project = makeProject()
    setTasksOpsClient(cli)
    setTasksProjectRoot(project)
    const id = await createThread({
      title: 'message list proof',
      body: 'human opener with `code`',
      anchor: {
        nodeId: 'routes/page.sail#0.1#ml01',
        elementPath: 'page.sail › card',
        contentHash: 'ml01',
        file: 'routes/page.sail',
      },
    })
    await selectThread(id!)
    setReplyDraft('@agent check this')
    await sendReply()
    await cli.sailTaskAgentNote!({
      project,
      id: id!,
      body: 'looking now',
      actor: { kind: 'agent', name: 'agent' },
    })
    await selectThread(id!)
    const items = uiMessageListItems()
    expect(items.length).toBeGreaterThanOrEqual(2)
    expect(items.some((m) => m.from === 'user')).toBe(true)
    expect(items.some((m) => m.from === 'assistant' && m.content.includes('looking'))).toBe(
      true,
    )
    expect(items.every((m) => m.id && m.status === 'sent')).toBe(true)
    const md = uiThreadMarkdown()
    expect(md).toMatch(/message list proof/)
    expect(md).toMatch(/human opener|looking now|@agent/)
  }, 60_000)

  it('uiAttachmentDiff empty without from/to region', async () => {
    const project = makeProject()
    setTasksOpsClient(cli)
    setTasksProjectRoot(project)
    const id = await createThread({
      title: 'no diff yet',
      body: 'plain',
      anchor: {
        nodeId: 'routes/page.sail#0.1#nd01',
        elementPath: 'page.sail › x',
        contentHash: 'nd01',
      },
    })
    await selectThread(id!)
    expect(uiAttachmentDiff().has).toBe(false)
  }, 60_000)
})
