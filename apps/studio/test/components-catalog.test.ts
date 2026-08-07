/**
 * Components catalog mapping + agent brief + open-edit arg shape.
 */
import { describe, expect, it } from 'vitest'
import {
  extractTagName,
  filterRecords,
  isProjectComponentPath,
  mergeComponentRecords,
  mergeProjectPathAuthority,
  projectComponentId,
  recordsFromCatalog,
  recordsFromProjectPaths,
} from '../src/components/catalog'
import {
  __clearComponentsActivityForTests,
  readComponentsActivity,
  writeComponentsActivity,
} from '../src/compositions/components-session/components-activity'
import { buildAgentBrief, buildNewComponentBrief } from '../src/components/brief'
import type { ComponentRecord, ComponentsCatalog } from '../src/components/types'
import { COMPONENTS_CATALOG } from '../components.gen'
import {
  componentHref,
  componentIdFromLocation,
  componentsHomeHref,
  modeFromLocation,
} from '../src/compositions/components-session/components-nav'

const FIXTURE_CATALOG: ComponentsCatalog = {
  schemaVersion: '1.0.0',
  modules: [
    {
      kind: 'javascript-module',
      path: 'sail-widget.sail',
      package: '@ship/sail-primitives',
      declarations: [
        {
          kind: 'class',
          name: 'SailWidget',
          tagName: 'sail-widget',
          customElement: true,
          description: 'Demo widget',
          attributes: [
            {
              name: 'size',
              type: { text: "'sm' | 'md'" },
              default: 'md',
              description: 'Control size',
            },
            { name: 'label', type: { text: 'string' }, required: true },
          ],
          slots: ['default', 'icon'],
        },
        {
          kind: 'class',
          name: 'NotCustom',
          // no tagName — skipped
        },
      ],
    },
    {
      path: 'dup.sail',
      package: '@other/pack',
      declarations: [
        {
          tagName: 'sail-widget',
          customElement: true,
          name: 'Dup',
        },
      ],
    },
  ],
}

describe('components catalog map', () => {
  it('maps CEM modules to pack records with props/slots', () => {
    const records = recordsFromCatalog(FIXTURE_CATALOG)
    expect(records).toHaveLength(1)
    const r = records[0]!
    expect(r.id).toBe('sail-widget')
    expect(r.origin).toBe('pack')
    expect(r.package).toBe('@ship/sail-primitives')
    expect(r.path).toContain('sail-widget.sail')
    expect(r.canOpenInEdit).toBe(false)
    expect(r.props.map((p) => p.name)).toEqual(['size', 'label'])
    expect(r.props.find((p) => p.name === 'label')?.required).toBe(true)
    expect(r.slots).toEqual(['default', 'icon'])
    expect(r.description).toBe('Demo widget')
    expect(r.layer).toBe('L2')
  })

  it('maps studio COMPONENTS_CATALOG without dumping empty', () => {
    const records = recordsFromCatalog(COMPONENTS_CATALOG)
    expect(records.length).toBeGreaterThan(5)
    expect(records.every((r) => r.origin === 'pack')).toBe(true)
    expect(records.every((r) => !!r.id && r.id.includes('-') || r.id.length > 0)).toBe(true)
    // Sample known pack tag
    expect(records.some((r) => r.id === 'sail-calendar' || r.id.startsWith('sail-'))).toBe(
      true,
    )
  })

  it('project path heuristics + merge prefers project on id clash', () => {
    expect(isProjectComponentPath('apps/studio/src/themes-surface/themes-surface.sail')).toBe(
      true,
    )
    expect(isProjectComponentPath('apps/studio/routes/page.sail')).toBe(false)
    expect(isProjectComponentPath('apps/studio/routes/edit/page.sail')).toBe(false)
    expect(isProjectComponentPath('apps/studio/test/fixtures/sample.sail')).toBe(false)
    expect(projectComponentId('apps/studio/src/edit-surface/edit-surface.sail')).toBe(
      'edit-surface',
    )

    const project = recordsFromProjectPaths([
      'apps/studio/src/themes-surface/themes-surface.sail',
      'apps/studio/routes/themes/page.sail',
    ])
    expect(project).toHaveLength(1)
    expect(project[0]!.origin).toBe('project')
    expect(project[0]!.canOpenInEdit).toBe(true)

    const packs = recordsFromCatalog(FIXTURE_CATALOG)
    // Force id collision
    const clash: ComponentRecord = {
      ...packs[0]!,
      id: 'themes-surface',
      origin: 'pack',
    }
    const merged = mergeComponentRecords(project, [clash, ...packs])
    const hit = merged.find((r) => r.id === 'themes-surface')
    expect(hit?.origin).toBe('project')
  })

  it('filterRecords matches name pack path', () => {
    const records = recordsFromCatalog(FIXTURE_CATALOG)
    expect(filterRecords(records, 'widget')).toHaveLength(1)
    expect(filterRecords(records, 'primitives')).toHaveLength(1)
    expect(filterRecords(records, 'zzz-none')).toHaveLength(0)
  })
})

describe('agent brief', () => {
  it('includes id path props origin', () => {
    const record: ComponentRecord = {
      id: 'sail-widget',
      displayName: 'Sail Widget',
      origin: 'pack',
      package: '@ship/sail-primitives',
      path: '@ship/sail-primitives/sail-widget.sail',
      layer: 'L2',
      description: 'Demo',
      props: [
        { name: 'size', type: 'string', defaultValue: 'md', required: false, description: '' },
        { name: 'label', type: 'string', defaultValue: '', required: true, description: '' },
      ],
      slots: [],
      issueCount: 2,
      ssrTierSummary: 'unknown',
      hasPreview: false,
      threadCount: 0,
      canOpenInEdit: false,
    }
    const text = buildAgentBrief({
      record,
      composition: ['row', 'button'],
      issuesSummary: '2 warnings',
    })
    expect(text).toContain('Component: sail-widget (pack)')
    expect(text).toContain('Path: @ship/sail-primitives/sail-widget.sail')
    expect(text).toContain('Pack: @ship/sail-primitives')
    expect(text).toContain('Props: size, label!')
    expect(text).toContain('Issues: 2 (2 warnings)')
    expect(text).toContain('Composition: row, button')
    expect(text).toContain('Intent: Demo')
  })

  it('new component brief marks draft', () => {
    const text = buildNewComponentBrief({
      name: 'my-chip',
      intent: 'Dense status chip',
      layer: 'app',
    })
    expect(text).toContain('Component: my-chip (project)')
    expect(text).toContain('Intent: Dense status chip')
    expect(text).toContain('draft brief')
  })
})

describe('components nav', () => {
  it('href builders and mode/id parse', () => {
    expect(componentsHomeHref()).toMatch(/\/components$/)
    expect(componentsHomeHref('inspect')).toMatch(/\/components\/inspect$/)
    expect(componentHref('sail-widget')).toMatch(/\/components\/sail-widget$/)
    expect(componentHref('a/b')).toContain(encodeURIComponent('a/b'))
    expect(componentHref('x', 'inspect')).toMatch(/\/components\/inspect$/)
    expect(componentIdFromLocation('/components/sail-tree')).toBe('sail-tree')
    expect(componentIdFromLocation('/components/inspect')).toBe('')
    expect(componentIdFromLocation('/components')).toBe('')
    expect(modeFromLocation('', '/components/inspect')).toBe('inspect')
    expect(modeFromLocation('')).toBe('library')
  })
})

describe('openDocument open-in-edit args (contract)', () => {
  it('library open path only when canOpenInEdit', () => {
    const project = recordsFromProjectPaths([
      'apps/studio/src/components-surface/components-surface.sail',
    ])[0]!
    expect(project.canOpenInEdit).toBe(true)
    expect(project.path).toBe('apps/studio/src/components-surface/components-surface.sail')
    // Session openInEdit is integration; assert arg shape used by checks/session:
    const openArgs = {
      path: project.path,
      opts: { loadPreview: false as const },
    }
    expect(openArgs.path.endsWith('.sail')).toBe(true)
    expect(openArgs.opts.loadPreview).toBe(false)

    const pack = recordsFromCatalog(FIXTURE_CATALOG)[0]!
    expect(pack.canOpenInEdit).toBe(false)
  })
})

describe('catalog path authority (H1)', () => {
  it('union never shrinks after a large ListFiles-style set', () => {
    const listFiles = [
      'apps/studio/src/a/a.sail',
      'apps/studio/src/b/b.sail',
      'apps/studio/src/c/c.sail',
    ]
    let auth = mergeProjectPathAuthority([], listFiles, 'replace').paths
    expect(auth).toHaveLength(3)
    // Edit seed is a strict subset — must not drop paths
    const afterEdit = mergeProjectPathAuthority(auth, [listFiles[0]!], 'union')
    expect(afterEdit.changed).toBe(false)
    expect(afterEdit.paths).toHaveLength(3)
    expect(afterEdit.paths).toEqual(expect.arrayContaining(listFiles))
    // New path only unions
    const withNew = mergeProjectPathAuthority(auth, ['apps/studio/src/d/d.sail'], 'union')
    expect(withNew.changed).toBe(true)
    expect(withNew.paths).toHaveLength(4)
    // replace may shrink (gen-owned reload only)
    const shrunk = mergeProjectPathAuthority(withNew.paths, [listFiles[0]!], 'replace')
    expect(shrunk.paths).toEqual([listFiles[0]])
  })

  it('extractTagName + activity bag preserve selection keys', () => {
    expect(extractTagName('sail-tree', '')).toBe('sail-tree')
    expect(extractTagName(undefined, '<column>')).toBe('column')
    expect(extractTagName(undefined, '<sail-calendar> events')).toBe('sail-calendar')
    __clearComponentsActivityForTests()
    writeComponentsActivity({ selectedId: 'sail-widget', filterQuery: 'wid' })
    expect(readComponentsActivity()).toEqual({
      selectedId: 'sail-widget',
      filterQuery: 'wid',
    })
    __clearComponentsActivityForTests()
  })
})
