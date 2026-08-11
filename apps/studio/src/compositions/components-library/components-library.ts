/**
 * Components library domain — catalog load, filter, groups, selection detail VMs.
 * Single path authority: gen-owned loadCatalog replaces; edit ticks only union.
 */
import { mountAmbient } from '../../vendor/runtime'
import { COMPONENTS_CATALOG } from '../../../components.gen'
import {
import { signal } from '../../vendor/runtime'
  filterRecords,
  mergeComponentRecords,
  mergeProjectPathAuthority,
  recordById,
  recordsFromCatalog,
  recordsFromProjectPaths,
} from '../../components/catalog'
import type {
  ComponentRecord,
  ComponentRowUi,
  KvRowUi,
  PropRowUi,
} from '../../components/types'
import type { ComponentsSession } from '../components-session/components-session'
import { getOpsClient, getEditState, subscribeEdit } from '../../edit/index.ts'
import { resolveProjectRoot, projectPaths } from '../shell/project'

export type ComponentsLibrary = ReturnType<typeof createComponentsLibrary>

function mapRow(r: ComponentRecord, sel: string): ComponentRowUi {
  const meta =
    r.origin === 'project'
      ? r.path.split('/').slice(-2).join('/') || r.path
      : r.package && r.package !== '—'
        ? r.package.replace(/^@ship\//, '')
        : r.path
  return {
    id: r.id,
    title: r.displayName || r.id,
    meta,
    path: r.path,
    origin: r.origin,
    package: r.package,
    selected: sel === r.id,
    originLabel: r.origin === 'project' ? 'project' : 'pack',
    originTone: r.origin === 'project' ? 'primary' : 'muted',
    issueLabel: r.issueCount > 0 ? String(r.issueCount) : '',
    hasIssues: r.issueCount > 0,
    key: r.id,
  }
}

export function createComponentsLibrary(session: ComponentsSession) {
  const allRecords = signal<ComponentRecord[]>([])
  const projectRows = signal<ComponentRowUi[]>([])
  const packRows = signal<ComponentRowUi[]>([])
  const hasProjectRows = signal(false)
  const hasPackRows = signal(false)
  const projectCountLabel = signal('0')
  const packCountLabel = signal('0')
  const isLoading = signal(false)
  const loadingLabel = signal('')
  const hasSelected = signal(false)
  const selectedRecord = signal<ComponentRecord | null>(null)

  const detailTitle = signal('Select a component')
  const detailPath = signal('')
  const detailOriginLabel = signal('')
  const detailOriginTone = signal('muted')
  const detailPackage = signal('')
  const detailLayer = signal('')
  const detailIntent = signal('No intent documented.')
  const hasIntent = signal(false)
  const canOpenInEdit = signal(false)
  const propRows = signal<PropRowUi[]>([])
  const hasProps = signal(false)
  const propsIncomplete = signal(false)
  const kvRows = signal<KvRowUi[]>([])
  const compositionLabel = signal('—')
  const slotsLabel = signal('—')
  const previewTitle = signal('Preview')
  const previewHint = signal('Select a component to preview.')
  const emptyProject = signal(false)
  const emptyAll = signal(false)
  const schemaNote = signal('')

  /** Catalog load generation — stale ListFiles settles must not overwrite. */
  let loadGen = 0
  /**
   * Authoritative project paths. Only gen-owned `loadCatalog` may replace.
   * Edit explorer ticks may only union (never shrink after ListFiles).
   */
  let authProjectPaths: string[] = []

  const recordsSnapshot = () => allRecords()

  const publishDetail = (rec: ComponentRecord | null) => {
    selectedRecord.set(rec)
    hasSelected.set(!!rec)
    if (!rec) {
      detailTitle.set(session.showNewForm() ? 'New component' : 'Select a component')
      detailPath.set('')
      detailOriginLabel.set('')
      detailOriginTone.set('muted')
      detailPackage.set('')
      detailLayer.set('')
      detailIntent.set(
        session.showNewForm()
          ? 'Describe intent, then copy a brief or start a thread.'
          : 'Choose a component from This project or Installed packs.',
      )
      hasIntent.set(false)
      canOpenInEdit.set(false)
      propRows.set([])
      hasProps.set(false)
      propsIncomplete.set(false)
      kvRows.set([])
      compositionLabel.set('—')
      slotsLabel.set('—')
      previewTitle.set('Preview')
      previewHint.set('Select a component to preview.')
      schemaNote.set('')
      return
    }
    detailTitle.set(rec.displayName || rec.id)
    detailPath.set(rec.path || '—')
    detailOriginLabel.set(rec.origin === 'project' ? 'This project' : 'Installed pack')
    detailOriginTone.set(rec.origin === 'project' ? 'primary' : 'muted')
    detailPackage.set(rec.package || '—')
    detailLayer.set(rec.layer)
    const intent = rec.description?.trim()
    hasIntent.set(!!intent)
    detailIntent.set(intent || 'No intent documented.')
    canOpenInEdit.set(!!rec.canOpenInEdit && !!rec.path && !rec.path.includes('@ship/'))
    const props: PropRowUi[] = rec.props.map((p, i) => ({
      name: p.name,
      type: p.type,
      defaultValue: p.defaultValue || '—',
      requiredLabel: p.required ? 'required' : 'optional',
      description: p.description || '',
      key: `${p.name}-${i}`,
    }))
    propRows.set(props)
    hasProps.set(props.length > 0)
    propsIncomplete.set(rec.origin === 'project' && props.length === 0)
    schemaNote.set(
      rec.origin === 'project' && props.length === 0
        ? 'Schema incomplete — project components have no CEM props yet.'
        : props.length === 0
          ? 'No attributes declared in catalog.'
          : '',
    )
    kvRows.set(
      [
        { label: 'Id', value: rec.id, key: 'id' },
        { label: 'Origin', value: rec.origin, key: 'origin' },
        { label: 'Package', value: rec.package || '—', key: 'package' },
        { label: 'Path', value: rec.path || '—', key: 'path' },
        { label: 'Layer', value: rec.layer, key: 'layer' },
        {
          label: 'SSR',
          value: rec.ssrTierSummary || 'unknown',
          key: 'ssr',
        },
      ].filter(Boolean) as KvRowUi[],
    )
    slotsLabel.set(rec.slots.length ? rec.slots.join(', ') : '—')
    compositionLabel.set(rec.slots.length ? `slots: ${rec.slots.join(', ')}` : '—')
    previewTitle.set(rec.displayName || rec.id)
    previewHint.set(
      rec.origin === 'pack'
        ? 'Pack components show catalog detail here. Open in docs or Edit when a path is available.'
        : 'Preview uses the app theme. Some project components mount live; others show catalog detail only.',
    )
  }

  const syncRows = () => {
    const q = session.filterQuery()
    const sel = session.selectedId()
    const filtered = filterRecords(allRecords(), q)
    const proj = filtered.filter((r) => r.origin === 'project').map((r) => mapRow(r, sel))
    const packs = filtered.filter((r) => r.origin === 'pack').map((r) => mapRow(r, sel))
    projectRows.set(proj)
    packRows.set(packs)
    hasProjectRows.set(proj.length > 0)
    hasPackRows.set(packs.length > 0)
    projectCountLabel.set(String(proj.length))
    packCountLabel.set(String(packs.length))
    emptyProject.set(proj.length === 0)
    emptyAll.set(filtered.length === 0 && !isLoading())
    const rec = recordById(allRecords(), sel)
    publishDetail(rec)
  }

  const ensureDefaultSelection = () => {
    const records = allRecords()
    if (!records.length) {
      if (session.selectedId()) session.setSelectedId('', { navigate: false })
      syncRows()
      return
    }
    const cur = session.selectedId()
    if (cur && recordById(records, cur)) {
      syncRows()
      return
    }
    // Prefer first project component
    const first = records.find((r) => r.origin === 'project') ?? records[0]
    if (first) {
      session.setSelectedId(first.id, { navigate: false })
    }
    syncRows()
  }

  const rebuildFromAuth = () => {
    const packs = recordsFromCatalog(COMPONENTS_CATALOG)
    const project = recordsFromProjectPaths(authProjectPaths)
    allRecords.set(mergeComponentRecords(project, packs))
    ensureDefaultSelection()
  }

  /** Gen-owned replace of project path authority + rebuild. */
  const applyCatalog = (projectPathsList: string[]) => {
    const { paths } = mergeProjectPathAuthority(authProjectPaths, projectPathsList, 'replace')
    authProjectPaths = paths
    rebuildFromAuth()
  }

  /**
   * Edit explorer tick — union only. Never replace with a strict subset of
   * the last ListFiles authority (H1).
   */
  const unionFromEditPaths = (paths: string[]) => {
    const { paths: next, changed } = mergeProjectPathAuthority(
      authProjectPaths,
      paths,
      'union',
    )
    if (!changed) return false
    authProjectPaths = next
    rebuildFromAuth()
    return true
  }

  const loadCatalog = async () => {
    const gen = ++loadGen
    isLoading.set(true)
    loadingLabel.set('Loading components…')
    const root = resolveProjectRoot()
    let files = projectPaths(root)
    const editFiles = getEditState().files
    if (editFiles?.length) {
      files = editFiles.map((f) => f.path)
    }
    try {
      try {
        const ops = getOpsClient()
        if (ops?.sailAuthoringListFiles) {
          const listed = await ops.sailAuthoringListFiles({ project: root })
          if (gen !== loadGen) return
          if (listed.ok && listed.files?.length) files = listed.files
        }
      } catch {
        /* keep seed */
      }
      if (gen !== loadGen) return
      applyCatalog(files)
    } finally {
      // Clear loading only for the owning generation (L1).
      if (gen === loadGen) {
        isLoading.set(false)
        loadingLabel.set('')
      }
    }
  }

  const select = (id: string) => {
    if (!id) return
    session.showNewForm.set(false)
    session.setSelectedId(id)
    syncRows()
  }

  const onListClick = (id: string) => {
    if (!id) return
    select(id)
  }

  const onOpenInEdit = () => {
    const rec = selectedRecord()
    if (!rec?.canOpenInEdit || !rec.path) {
      session.setNote('No project path to open in Edit.', 'warning')
      return
    }
    void session.openInEdit(rec.path)
  }

  const onStartThread = () => {
    void session.startThread(selectedRecord())
  }

  const onCopyBrief = () => {
    void session.copyAgentBrief(selectedRecord())
  }

  const getRecord = (id: string) => recordById(allRecords(), id)

  syncRows()

  mountAmbient(() => {
    const unsub = session.onSync(() => syncRows())
    const unsubEdit = subscribeEdit(() => {
      // Union-only: edit.files may be a partial seed — never shrink ListFiles catalog.
      if (isLoading()) return
      const files = getEditState().files
      if (!files?.length) return
      unionFromEditPaths(files.map((f) => f.path))
    })
    void loadCatalog()
    return () => {
      unsub()
      unsubEdit()
      loadGen += 1 // invalidate in-flight
    }
  })

  return {
    allRecords,
    projectRows,
    packRows,
    hasProjectRows,
    hasPackRows,
    projectCountLabel,
    packCountLabel,
    isLoading,
    loadingLabel,
    hasSelected,
    selectedRecord,
    detailTitle,
    detailPath,
    detailOriginLabel,
    detailOriginTone,
    detailPackage,
    detailLayer,
    detailIntent,
    hasIntent,
    canOpenInEdit,
    propRows,
    hasProps,
    propsIncomplete,
    kvRows,
    compositionLabel,
    slotsLabel,
    previewTitle,
    previewHint,
    emptyProject,
    emptyAll,
    schemaNote,
    loadCatalog,
    select,
    onListClick,
    onOpenInEdit,
    onStartThread,
    onCopyBrief,
    getRecord,
    recordsSnapshot,
    /** @internal tests */
    __applyCatalog: applyCatalog,
    __unionFromEditPaths: unionFromEditPaths,
    __authProjectPaths: () => authProjectPaths.slice(),
    __loadGen: () => loadGen,
    __bumpLoadGen: () => {
      loadGen += 1
    },
  }
}
