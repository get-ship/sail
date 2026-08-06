/**
 * STUDIO-2 regressions — R1–R6 gates + bar token checks.
 * Split from shell.test.ts to hold the hard ≤400 LOC file cap.
 */
import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

const REPO = join(dirname(fileURLToPath(import.meta.url)), '../../..')
const SHELL_VM = join(REPO, 'apps/studio/src/shell/shell-page-vm.ts')

const SURFACE_VMS = [
  'apps/studio/src/shell/shell-page-vm.ts',
  'apps/studio/src/edit-surface/edit-surface-vm.ts',
  'apps/studio/src/edit-explorer/edit-explorer-vm.ts',
  'apps/studio/src/edit-element-tree/edit-element-tree-vm.ts',
  'apps/studio/src/checks-surface/checks-surface-vm.ts',
  'apps/studio/src/themes-surface/themes-surface-vm.ts',
  'apps/studio/src/tasks-surface/tasks-surface-vm.ts',
  'apps/studio/src/theme-session/theme-session.ts',
] as const

describe('STUDIO-2 repairs + bar gates', () => {
  it('R1: ambient mountAmbient (not route mount) in every surface VM', () => {
    // Bare `mount` from @ship/sail-runtime is the host route mount and reads
    // payload.params; ambient lifecycle must use mountAmbient.
    for (const rel of SURFACE_VMS) {
      const src = readFileSync(join(REPO, rel), 'utf8')
      expect(src, rel).toMatch(/mountAmbient\s*\(/)
      expect(src, rel).toMatch(
        /import \{[^}]*mountAmbient[^}]*\} from ['"]@ship\/sail-runtime['"]/,
      )
      const masked = src.replace(/mountAmbient/g, '__AMBIENT__')
      expect(masked, rel).not.toMatch(/\bmount\s*\(/)
      expect(masked, rel).not.toMatch(
        /import \{[^}]*\bmount\b[^}]*\} from ['"]@ship\/sail-runtime['"]/,
      )
    }
  })

  it('R1: host route mount throws params TypeError on ambient-style call', async () => {
    // Spec wants zero pageerrors on load. Full browser harness is optional;
    // this unit-locks the exact symptom: mis-calling route mount as lifecycle
    // reads payload.params of undefined → "reading 'params'".
    // Host route mount lives on /internal (tier B root only exports mountAmbient).
    const { mount: routeMount } = await import('@ship/sail-runtime/internal')
    await expect(
      // @ts-expect-error intentional wrong ambient-style call
      routeMount(() => {}),
    ).rejects.toThrow(/params/)
  })

  it('R4/R5/B1: workbench theme + neutral empty borders', () => {
    expect(existsSync(join(REPO, 'apps/studio/themes/studio.json'))).toBe(true)
    const theme = readFileSync(join(REPO, 'apps/studio/themes/studio.json'), 'utf8')
    expect(theme).toMatch(/"themeId":\s*"studio"/)
    expect(theme).toMatch(/Fraunces/)
    // Warm shadows (not cool hue 250)
    expect(theme).not.toMatch(/oklch\([^)]*\s250\s/)
    const shellVm = readFileSync(SHELL_VM, 'utf8')
    expect(shellVm).toMatch(/theme',\s*'studio'|setThemeMode\('studio'/)
    const cfg = readFileSync(join(REPO, 'apps/studio/sail.config.jsonc'), 'utf8')
    expect(cfg).toMatch(/"default"\s*:\s*"studio"/)
    for (const rel of [
      'apps/studio/src/tasks-surface/tasks-surface.sail',
      'apps/studio/src/checks-surface/checks-surface.sail',
      'apps/studio/src/edit-explorer/edit-explorer.sail',
      'apps/studio/src/edit-surface/edit-surface.sail',
    ]) {
      const src = readFileSync(join(REPO, rel), 'utf8')
      expect(src, rel).not.toMatch(/empty-state[^>]*tone=["']warning["']/)
      // Neutral empty borders: border / borderT / placeholder variants all count
      expect(src, rel).toMatch(
        /border(?:T|B|L|R)?\(border-subtle\)|variant="placeholder"|variant="inline"/,
      )
    }
  })

  it('B2: theme domain specimen + type roles keep Fraunces display pairing', () => {
    const themeIo = readFileSync(join(REPO, 'apps/studio/src/design/theme-io.ts'), 'utf8')
    expect(themeIo).toMatch(/display:\s*['"]Fraunces['"]/)
    const specimen = readFileSync(join(REPO, 'apps/studio/src/design/specimen.ts'), 'utf8')
    expect(specimen).toMatch(/\?\? ['"]Fraunces['"]/)
    // Themes facets own product UI (Design surface removed)
    const colorSpec = readFileSync(
      join(REPO, 'apps/studio/src/theme-color-specimen/theme-color-specimen.sail'),
      'utf8',
    )
    expect(colorSpec).toMatch(/color-specimen|preview-specimen/)
    expect(colorSpec).not.toMatch(/theme-preview=/)
  })

  it('B3: explorer sail-tree labels stay short (no size meta — stacked-text fix)', () => {
    const vm = readFileSync(join(REPO, 'apps/studio/src/edit-explorer/edit-explorer-vm.ts'), 'utf8')
    const explorer = readFileSync(join(REPO, 'apps/studio/src/edit/explorer.ts'), 'utf8')
    expect(vm).toMatch(/fileSailTreeNodes|createTable/)
    expect(explorer).toMatch(/fileSailTreeNodes/)
    expect(explorer).not.toMatch(/sourceLen/)
    expect(explorer).not.toMatch(/KB/)
  })

  it('B4: Themes no permanent Used by chrome + no Design product surface', () => {
    const chrome = readFileSync(
      join(REPO, 'apps/studio/src/theme-facet-chrome/theme-facet-chrome.sail'),
      'utf8',
    )
    expect(chrome).not.toMatch(/used-by/)
    expect(chrome).toMatch(/theme-facet-chrome/)
    expect(existsSync(join(REPO, 'apps/studio/src/design-surface'))).toBe(false)
    expect(existsSync(join(REPO, 'apps/studio/src/design-inspector'))).toBe(false)
  })

  it(
    'R6: sail check clean on chrome + surfaces (batched)',
    () => {
      const sailBin = join(REPO, 'target/debug/sail')
      expect(existsSync(sailBin), 'target/debug/sail must be built for studio tests').toBe(true)
      const r = spawnSync(
        'sh',
        [
          '-c',
          `"$1" check "$2" && "$1" check "$3"`,
          'sh',
          sailBin,
          join(REPO, 'apps/studio/routes'),
          join(REPO, 'apps/studio/src'),
        ],
        {
          encoding: 'utf8',
          cwd: REPO,
          timeout: 300_000,
        },
      )
      const out = `${r.stdout}\n${r.stderr}`
      expect(r.status, out).toBe(0)
      expect(out).not.toMatch(/raw <(?:div|span)/)
      expect(out).not.toMatch(/sail-no-raw-container/)
    },
    360_000,
  )
})
