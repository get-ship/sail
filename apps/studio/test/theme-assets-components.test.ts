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
  it('schema-missing component path write is a hard error', async () => {
    const { setTokenValue } = await import('../src/design')
    __setWorkingDocForTests({
      $extensions: { ship: { themeId: 'comp-toy' } },
      modes: { light: { component: {} } },
    })
    await setTokenValue('component.button.not-a-real-slot', '#22c55e')
    const err = getDesignState().lastWriteError
    expect(err).toBeTruthy()
    expect(String(err)).toMatch(/Unknown component surface path|not in the host surface schema/i)
    expect(getDesignState().dirty).toBe(false)
  })

  it('setTokenValue can write a component-scoped color hex to oklch', async () => {
    const { setTokenValue, getPathValue } = await import('../src/design')
    __setWorkingDocForTests({
      $extensions: { ship: { themeId: 'comp-toy' } },
      modes: {
        light: {
          component: {
            button: {
              bg: { $value: 'oklch(0.5 0.1 40)', $type: 'color' },
            },
          },
        },
      },
    })
    await setTokenValue('component.button.bg', '#22c55e')
    const v = getPathValue(getDesignState().themeDoc, 'component.button.bg', 'light')
    expect(v).toMatch(/^oklch\(/)
    expect(getDesignState().dirty).toBe(true)
  })
})

describe('composite component knobs (SPIN-3)', () => {
  it('dialog.border / card.border members label as one face', async () => {
    const { compositeForPath, componentKnobLabel } = await import('../src/design')
    const color = compositeForPath('component.dialog.border.color')
    expect(color?.name).toBe('dialog.border')
    expect(color?.member).toBe('color')
    const width = compositeForPath('component.card.border.width')
    expect(width?.name).toBe('card.border')
    expect(width?.member).toBe('width')
    expect(componentKnobLabel('component.dialog.border.color', 'border')).toBe(
      'dialog.border · color',
    )
    expect(componentKnobLabel('component.card.border.style', 'border')).toBe(
      'card.border · style',
    )
    expect(compositeForPath('component.dialog.bg')).toBeNull()
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
    expect(compositeForPath('fill.surface.color')?.name).toBe('fill.surface')

    const groups = listTokenGroups()
    for (const id of ['size', 'type', 'elevation', 'fill', 'ink']) {
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
      modes: { light: { component: { button: {} } } },
    })
    await setTokenValue('component.button.hover-bg', '#22c55e')
    const err = getDesignState().lastWriteError
    expect(err).toBeTruthy()
    expect(String(err)).toMatch(/Parked component state path|not an editable rest surface/i)
    expect(getDesignState().dirty).toBe(false)
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
