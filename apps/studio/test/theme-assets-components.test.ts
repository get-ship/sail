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
