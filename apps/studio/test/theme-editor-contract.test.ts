/**
 * THEMES-1 studio editor contract — every control maps a schema field;
 * five facets covered; no orphan write paths.
 */
import { describe, expect, it } from 'vitest'
import {
  THEME_EDITOR_CONTROLS,
  THEME_EDITOR_CONTROL_IDS,
  contractSchemaCoverage,
  controlsForFacet,
  isContractWritePath,
} from '../src/design/theme-editor-contract'
import type { ThemeFacet } from '../src/compositions/theme-session/theme-nav'

const FACETS: ThemeFacet[] = ['overview', 'color', 'shape', 'assets', 'components']

describe('THEMES-1 theme editor contract', () => {
  it('covers all five studio theme facets', () => {
    for (const f of FACETS) {
      const controls = controlsForFacet(f)
      expect(controls.length, f).toBeGreaterThan(0)
    }
  })

  it('every control has a schema path and kind', () => {
    for (const c of THEME_EDITOR_CONTROLS) {
      expect(c.id).toBeTruthy()
      expect(c.schemaPath).toBeTruthy()
      expect(['identity', 'mode', 'scale', 'component', 'meta']).toContain(c.kind)
      expect(FACETS).toContain(c.facet)
    }
    // unique ids
    expect(new Set(THEME_EDITOR_CONTROL_IDS).size).toBe(THEME_EDITOR_CONTROL_IDS.length)
  })

  it('schema coverage includes identity, modes, scales — not per-tag component leaves', () => {
    const cov = contractSchemaCoverage()
    expect(cov.identity).toContain('$extensions.sail.themeId')
    expect(cov.mode).toContain('$extensions.sail.defaultMode')
    expect(cov.mode).toContain('$extensions.sail.supportedModes')
    expect(cov.scaleGroups).toContain('color')
    expect(cov.scaleGroups).toContain('radius')
    expect(cov.scaleGroups).toContain('typography')
    expect(cov.component).toBe(false)
  })

  it('contract write path accepts mode-scoped scale roles and refuses component leaves', () => {
    expect(isContractWritePath('modes.light.color.primary.default')).toBe(true)
    expect(isContractWritePath('modes.light.color.input')).toBe(true)
    expect(isContractWritePath('modes.light.color.card.default')).toBe(true)
    expect(isContractWritePath('modes.dark.component.button.bg')).toBe(false)
    expect(isContractWritePath('modes.light.radius.md')).toBe(true)
    expect(isContractWritePath('modes.light.typography.fontFamily.body')).toBe(true)
    expect(isContractWritePath('modes.light.shadow.sm')).toBe(true)
    expect(isContractWritePath('modes.light.icon.size')).toBe(true)
    expect(isContractWritePath('modes.light.interaction.strength')).toBe(true)
    expect(isContractWritePath('$extensions.sail.themeId')).toBe(true)
    expect(isContractWritePath('$extensions.sail.density')).toBe(true)
    expect(isContractWritePath('')).toBe(false)
    expect(isContractWritePath('not-a-path')).toBe(false)
  })

  it('contract write path refuses schema groups with no editor control', () => {
    // THEME_MODE_GROUP_KEYS includes these; editor contract product subset does not.
    expect(isContractWritePath('modes.light.motion.duration.fast')).toBe(false)
    expect(isContractWritePath('modes.dark.space.4')).toBe(false)
    expect(isContractWritePath('modes.light.scrollbar.thumb')).toBe(false)
    expect(isContractWritePath('modes.light.density.compact')).toBe(false)
    expect(isContractWritePath('modes.light.panel.min-w')).toBe(false)
    expect(isContractWritePath('modes.light.z.modal')).toBe(false)
    expect(isContractWritePath('modes.light.breakpoint.tablet')).toBe(false)
    expect(isContractWritePath('modes.light.control.h-md')).toBe(false)
  })

  it('components facet exposes live role controls (not per-tag leaves)', () => {
    const comps = controlsForFacet('components')
    expect(comps.some((c) => c.id === 'components.token')).toBe(true)
    expect(comps.some((c) => c.id === 'components.globalFallback')).toBe(true)
    expect(comps.some((c) => c.kind === 'component')).toBe(false)
    expect(comps.every((c) => !c.schemaPath.includes('component.{tag}'))).toBe(true)
  })
})
