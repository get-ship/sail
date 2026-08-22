/**
 * Assets + Components facet behavior (store ops / catalog), not template scans.
 */
import { afterEach, describe, expect, it } from 'vitest'
import {
  __resetDesignForTests,
  __setWorkingDocForTests,
  addFontFamily,
  getDesignState,
  searchFonts,
  setDesignOpsClient,
  setTypeRole,
  type DesignOpsClient,
} from '../src/design'

afterEach(() => {
  __resetDesignForTests()
  setDesignOpsClient(null)
})

function mockClient(partial: Partial<DesignOpsClient>): DesignOpsClient {
  return partial as DesignOpsClient
}

describe('font discoverer behavior', () => {
  it('searchFonts returns families from SailTheme/FontSearch (not a local guess list)', async () => {
    setDesignOpsClient(
      mockClient({
        sailThemeFontSearch: async ({ query }) => ({
          ok: true,
          families:
            query.toLowerCase().includes('pop')
              ? ['Poppins', 'Poppins ExtraBold']
              : ['Roboto', 'Inter', 'Open Sans'],
          hits: [
            { family: 'Poppins', category: 'Sans Serif', popularity: 7, mono: false },
          ],
          count: 1,
          catalogSize: 1942,
          source: 'bundled-google-fonts',
        }),
      }),
    )
    const r = await searchFonts('poppins')
    expect(r.families).toContain('Poppins')
    expect(r.catalogSize).toBe(1942)
    expect(r.hits[0]?.family).toBe('Poppins')
    expect(r.source).toMatch(/google|bundled/i)
  })

  it('searchFonts with no client returns empty (honest failure, not fake hits)', async () => {
    setDesignOpsClient(null)
    const r = await searchFonts('inter')
    expect(r.families).toEqual([])
    expect(r.hits).toEqual([])
    expect(r.count).toBe(0)
  })

  it('addFontFamily installs via ops then sets body role; setTypeRole writes typography', async () => {
    const added: string[] = []
    setDesignOpsClient(
      mockClient({
        sailThemeFontAdd: async ({ family }) => {
          added.push(family)
          return { ok: true, family }
        },
      }),
    )
    __setWorkingDocForTests({
      $extensions: { ship: { themeId: 'font-toy' } },
      modes: { light: {} },
    })
    const r = await addFontFamily('Poppins', 'body')
    expect(r.ok).toBe(true)
    expect(added).toEqual(['Poppins'])
    expect(getDesignState().typeRoles.body).toBe('Poppins')
    expect(getDesignState().diffRows.some((row) => row.path.includes('typography'))).toBe(true)
  })

  it('setTypeRole display/mono updates type roles without FontAdd', async () => {
    __setWorkingDocForTests({ $extensions: { ship: { themeId: 'ty' } } })
    await setTypeRole('display', 'Fraunces')
    await setTypeRole('mono', 'JetBrains Mono')
    const s = getDesignState()
    expect(s.typeRoles.display).toBe('Fraunces')
    expect(s.typeRoles.mono).toBe('JetBrains Mono')
    expect(s.diffRows.some((r) => r.path.includes('typography'))).toBe(true)
  })
})

describe('component token write behavior', () => {
  it('component.* path write is a hard error', async () => {
    const { setTokenValue } = await import('../src/design')
    __setWorkingDocForTests({
      $extensions: { ship: { themeId: 'comp-toy' } },
      modes: { light: { color: {} } },
    })
    await setTokenValue('component.button.not-a-real-slot', '#22c55e')
    const err = getDesignState().lastWriteError
    expect(err).toBeTruthy()
    expect(String(err)).toMatch(/Unknown theme path|write a color, type, or radius role/i)
    expect(getDesignState().dirty).toBe(false)
  })

  it('setTokenValue writes a live color role hex to oklch', async () => {
    const { setTokenValue, getPathValue } = await import('../src/design')
    __setWorkingDocForTests({
      $extensions: { ship: { themeId: 'comp-toy' } },
      modes: {
        light: {
          color: {
            card: {
              default: { $value: 'oklch(0.5 0.1 40)', $type: 'color' },
            },
          },
        },
      },
    })
    await setTokenValue('color.card.default', '#22c55e')
    const v = getPathValue(getDesignState().themeDoc, 'color.card.default', 'light')
    expect(v).toMatch(/^oklch\(/)
    expect(getDesignState().dirty).toBe(true)
    expect(getDesignState().lastWriteError).toBeNull()
  })

  it('setTokenValue refuses a retired component.button.bg path', async () => {
    const { setTokenValue, getPathValue } = await import('../src/design')
    __setWorkingDocForTests({
      $extensions: { ship: { themeId: 'comp-toy' } },
      modes: {
        light: {
          color: {
            primary: { default: { $value: 'oklch(0.5 0.1 40)', $type: 'color' } },
          },
        },
      },
    })
    await setTokenValue('component.button.bg', '#22c55e')
    expect(getDesignState().lastWriteError).toMatch(/Unknown theme path|write a color/i)
    expect(getPathValue(getDesignState().themeDoc, 'component.button.bg', 'light')).toBeNull()
    expect(getDesignState().dirty).toBe(false)
  })
})

describe('composite component knobs (SPIN-3)', () => {
  it('bg.surface / bg.app members label as one face', async () => {
    const { compositeForPath, componentKnobLabel } = await import('../src/design')
    const color = compositeForPath('bg.surface.color')
    expect(color?.name).toBe('bg.surface')
    expect(color?.member).toBe('color')
    const app = compositeForPath('bg.app.color')
    expect(app?.name).toBe('bg.app')
    expect(app?.member).toBe('color')
    expect(componentKnobLabel('bg.surface.color', 'color')).toBe('bg.surface · color')
    expect(componentKnobLabel('bg.app.color', 'color')).toBe('bg.app · color')
    expect(compositeForPath('color.card.default')).toBeNull()
  })

  it('size / type / elevation composites fold as faces, not pad-on-size', async () => {
    const { compositeForPath, componentKnobLabel, listTokenGroups, getKnob } = await import(
      '../src/design'
    )
    expect(compositeForPath('size.control-md.h')?.name).toBe('size.control-md')
    expect(compositeForPath('size.control-md.h')?.member).toBe('h')
    expect(componentKnobLabel('size.control-md.h', 'h')).toBe('size.control-md · h')
    expect(compositeForPath('size.control-md.pad')).toBeNull()
    expect(compositeForPath('size.control-sm')?.member).toBe('')

    expect(compositeForPath('type.heading.family')?.name).toBe('type.heading')
    expect(componentKnobLabel('type.heading.size', 'size')).toBe('type.heading · size')
    expect(compositeForPath('elevation.sm.shadow')?.name).toBe('elevation.sm')
    expect(componentKnobLabel('elevation.sm.shadow', 'shadow')).toBe('elevation.sm · shadow')
    expect(compositeForPath('bg.surface.color')?.name).toBe('bg.surface')

    const groups = listTokenGroups()
    for (const id of ['size', 'type', 'elevation', 'bg', 'fg']) {
      expect(groups.some((g) => g.id === id)).toBe(true)
    }
    const sizeGroup = groups.find((g) => g.id === 'size')
    expect(sizeGroup?.knobs.some((k) => k.path.includes('pad'))).toBe(false)
    expect(getKnob('size.control-md.h')?.label).toBe('size.control-md · h')
  })
})

describe('component state leaf parking (SPEC-D1 rest-only)', () => {
  it('parked hover storage leaf write is a hard error (not a silent look no-op)', async () => {
    const { setTokenValue } = await import('../src/design')
    __setWorkingDocForTests({
      $extensions: { ship: { themeId: 'comp-toy' } },
      modes: { light: { color: {} } },
    })
    await setTokenValue('component.button.hover-bg', '#22c55e')
    const err = getDesignState().lastWriteError
    expect(err).toBeTruthy()
    expect(String(err)).toMatch(
      /Unknown theme path|Parked component state path|write a color|not an editable rest surface/i,
    )
    expect(getDesignState().dirty).toBe(false)
  })

  it('Themes Components face roles never write component.* paths', async () => {
    const { FACE_ROLES } = await import(
      '../src/compositions/theme-components-surface/theme-components-edit'
    )
    const paths = Object.values(FACE_ROLES).flatMap((rows) => rows.map((r) => r.path))
    expect(paths.length).toBeGreaterThan(0)
    expect(paths.every((p) => !p.startsWith('component.'))).toBe(true)
    expect(paths).toContain('color.input')
    expect(paths).toContain('color.surface')
    expect(paths).toContain('color.popover.default')
    expect(paths).toContain('color.primary.default')
    expect(paths).toContain('color.foreground')
    expect(paths).toContain('typography.label.weight')
    expect(FACE_ROLES.checkbox?.some((r) => r.path === 'color.input')).toBe(true)
    expect(FACE_ROLES.field?.some((r) => r.path === 'color.input')).toBe(true)
    expect(FACE_ROLES.button?.some((r) => r.path === 'color.primary.default')).toBe(true)
    expect(FACE_ROLES.label?.some((r) => r.path === 'typography.label.weight')).toBe(true)
  })

  it('isParkedComponentStateLeaf covers hover leaves but not hover-card pack rest', async () => {
    const { isParkedComponentStateLeaf } = await import('../src/design')
    expect(isParkedComponentStateLeaf('component.button.hover-bg')).toBe(true)
    expect(isParkedComponentStateLeaf('component.field.border-hover')).toBe(true)
    expect(isParkedComponentStateLeaf('component.button.bg')).toBe(false)
    expect(isParkedComponentStateLeaf('component.hover-card.bg')).toBe(false)
    expect(isParkedComponentStateLeaf('component.hover-card.fg')).toBe(false)
  })
})
