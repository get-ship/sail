// theme-select — catalog filter tabs (no paint on tab), theme row applies,
// mode-only cycle, never data-mode=system on the document.
// P-1: product grid uses effective emit set, not always full THEMES.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  themeSelect,
  setEffectiveThemeIds,
  effectiveProductThemes,
  parseEffectiveThemeIdsContent,
  readEffectiveThemeIds,
} from '../src/actions/theme-select'
import {
  THEMES,
  THEME_MODE_STORAGE_KEY,
  applyTheme,
  persistMode,
  readModePreference,
} from '@ship/tokens/themes'

function optionValues(grid: ParentNode): string[] {
  return Array.from(grid.querySelectorAll('[role="option"][value]')).map(
    (o) => o.getAttribute('value') ?? '',
  )
}

function stampMeta(name: string, content: string): HTMLMetaElement {
  const meta = document.createElement('meta')
  meta.setAttribute('name', name)
  meta.setAttribute('content', content)
  document.head.appendChild(meta)
  return meta
}

function harness(opts?: { modeOnly?: boolean; extraOption?: string }) {
  const root = document.createElement('div')
  if (opts?.modeOnly) root.setAttribute('mode-only', '')

  const trigger = document.createElement('button')
  trigger.type = 'button'
  trigger.setAttribute('data-picker-trigger', 'full')
  trigger.setAttribute('popovertarget', 'sail-theme-panel')
  trigger.setAttribute('aria-label', 'Choose theme')
  root.appendChild(trigger)

  const modeTrigger = document.createElement('button')
  modeTrigger.type = 'button'
  modeTrigger.setAttribute('data-picker-trigger', 'mode')
  modeTrigger.setAttribute('data-mode-only-trigger', '1')
  modeTrigger.setAttribute('aria-label', 'Color mode')
  modeTrigger.hidden = true
  for (const kind of ['light', 'dark', 'system'] as const) {
    const icon = document.createElement('span')
    icon.setAttribute('data-mode-icon', kind)
    if (kind !== 'light') icon.hidden = true
    icon.textContent = kind[0]!
    modeTrigger.appendChild(icon)
  }
  root.appendChild(modeTrigger)

  const panel = document.createElement('div')
  panel.id = 'sail-theme-panel'
  panel.setAttribute('popover', '')
  panel.setAttribute('role', 'listbox')
  // jsdom: pretend popover API
  panel.hidePopover = () => {
    panel.toggleAttribute('open', false)
  }
  root.appendChild(panel)

  const header = document.createElement('div')
  header.setAttribute('data-picker-header', '')
  const sel = document.createElement('span')
  sel.setAttribute('data-selected-label', '')
  sel.textContent = 'Theme'
  header.appendChild(sel)
  const catalog = document.createElement('span')
  catalog.setAttribute('data-catalog-label', '')
  catalog.textContent = 'Light themes'

  const modeBar = document.createElement('div')
  modeBar.setAttribute('data-mode-bar', '')
  modeBar.setAttribute('data-mode-segment', '')
  modeBar.setAttribute('role', 'tablist')
  for (const m of ['light', 'dark', 'system'] as const) {
    const b = document.createElement('button')
    b.type = 'button'
    b.setAttribute('data-mode', m)
    b.setAttribute('role', 'tab')
    b.textContent = m
    modeBar.appendChild(b)
  }
  header.appendChild(modeBar)

  const lock = document.createElement('div')
  lock.setAttribute('data-mode-lock-banner', '')
  lock.hidden = true
  const lockText = document.createElement('span')
  lockText.setAttribute('data-mode-lock-text', '')
  lock.appendChild(lockText)
  header.appendChild(lock)
  panel.appendChild(header)
  panel.appendChild(catalog)

  const grid = document.createElement('div')
  grid.setAttribute('data-theme-grid', '')
  // Skeleton row — ensureCatalog fills from THEMES.
  const mkOption = (id: string, label: string) => {
    const btn = document.createElement('button')
    btn.type = 'button'
    btn.setAttribute('role', 'option')
    btn.setAttribute('value', id)
    const row = document.createElement('div')
    const preview = document.createElement('div')
    preview.setAttribute('theme', id)
    preview.setAttribute('data-mode', 'light')
    const title = document.createElement('span')
    title.setAttribute('data-swatch-title', '')
    title.textContent = label
    preview.appendChild(title)
    const outer = document.createElement('span')
    outer.setAttribute('data-option-label', '')
    outer.textContent = label
    const check = document.createElement('span')
    check.setAttribute('data-selected-check', '')
    check.hidden = true
    row.append(preview, outer, check)
    btn.appendChild(row)
    return btn
  }
  grid.appendChild(mkOption('ship', 'SHIP'))
  if (opts?.extraOption) {
    grid.appendChild(mkOption(opts.extraOption, opts.extraOption))
  }
  panel.appendChild(grid)

  document.body.appendChild(root)
  document.documentElement.setAttribute('theme', 'ship')
  document.documentElement.dataset.mode = 'light'
  return {
    root,
    trigger,
    modeTrigger,
    panel,
    grid,
    modeBar,
  }
}

describe('themeSelect — catalog filter + theme apply', () => {
  let destroy: (() => void) | undefined

  beforeEach(() => {
    setEffectiveThemeIds(null)
    vi.unstubAllGlobals()
    document.documentElement.removeAttribute('theme')
    document.documentElement.removeAttribute('data-mode')
    document.documentElement.classList.remove('dark')
    delete document.documentElement.dataset.studioPreview
    document.head
      .querySelectorAll(
        'meta[name="sail-effective-themes"], meta[name="sail-project"], meta[name="sail-project-themes"]',
      )
      .forEach((m) => m.remove())
    try {
      localStorage.clear()
    } catch {
      /* */
    }
  })

  afterEach(() => {
    destroy?.()
    destroy = undefined
    setEffectiveThemeIds(null)
    vi.unstubAllGlobals()
    document.body.replaceChildren()
    document.documentElement.removeAttribute('theme')
    document.documentElement.removeAttribute('data-mode')
    delete document.documentElement.dataset.studioPreview
    document.head
      .querySelectorAll(
        'meta[name="sail-effective-themes"], meta[name="sail-project"], meta[name="sail-project-themes"]',
      )
      .forEach((m) => m.remove())
  })

  it('builds options from THEMES when no effective stamp (full catalog) and drops unknowns', () => {
    const { root, grid } = harness({ extraOption: 'not-a-real-theme' })
    const action = themeSelect(root)
    destroy = action?.destroy

    const values = optionValues(grid)
    expect(values.sort()).toEqual([...THEMES.map((t) => t.id)].sort())
    expect(values).not.toContain('not-a-real-theme')
    expect(values.length).toBe(THEMES.length)
  })

  it('builds product grid from allowlist (effective emit set), not full THEMES', () => {
    setEffectiveThemeIds(['neutral', 'paper'])
    const { root, grid } = harness({ extraOption: 'synthwave' })
    const action = themeSelect(root)
    destroy = action?.destroy

    const values = optionValues(grid)
    expect(values.sort()).toEqual(['neutral', 'paper'].sort())
    expect(values).not.toContain('synthwave')
    expect(values).not.toContain('ship')
    expect(values).not.toContain('glass')
    expect(values.length).toBe(2)
  })

  it('empty effective allowlist yields empty product grid', () => {
    setEffectiveThemeIds([])
    const { root, grid } = harness()
    const action = themeSelect(root)
    destroy = action?.destroy

    // Skeleton row removed when not in effective set.
    expect(optionValues(grid)).toEqual([])
  })

  it('preserves option template so empty→allowlist rebuild works without remount', () => {
    setEffectiveThemeIds([])
    const { root, grid } = harness()
    const action = themeSelect(root)
    destroy = action?.destroy
    expect(optionValues(grid)).toEqual([])

    setEffectiveThemeIds(['neutral', 'paper'])
    document.dispatchEvent(new CustomEvent('ship:theme:change'))
    expect(optionValues(grid).sort()).toEqual(['neutral', 'paper'].sort())
  })

  it('reads sail-effective-themes meta as product grid source', () => {
    stampMeta('sail-effective-themes', 'neutral,paper')

    const { root, grid } = harness()
    const action = themeSelect(root)
    destroy = action?.destroy

    expect(optionValues(grid).sort()).toEqual(['neutral', 'paper'].sort())
  })

  it('meta takes precedence over session ops ids', () => {
    stampMeta('sail-effective-themes', 'paper')
    // Session would suggest a broader set if consulted first.
    expect(readEffectiveThemeIds(['neutral', 'paper', 'ship'])).toEqual(['paper'])
    expect(effectiveProductThemes(['neutral', 'paper', 'ship']).map((t) => t.id)).toEqual([
      'paper',
    ])
  })

  it('parseEffectiveThemeIdsContent rejects path-like and unsafe tokens', () => {
    expect(parseEffectiveThemeIdsContent('all')).toBeNull()
    expect(parseEffectiveThemeIdsContent('')).toEqual([])
    expect(parseEffectiveThemeIdsContent('neutral,paper')).toEqual(['neutral', 'paper'])
    expect(parseEffectiveThemeIdsContent('../evil,ship,Has Space')).toEqual(['ship'])
    expect(parseEffectiveThemeIdsContent('ship/../x')).toEqual([])
  })

  it('effectiveProductThemes filters THEMES by allowlist', () => {
    setEffectiveThemeIds(['neutral', 'paper'])
    const ids = effectiveProductThemes().map((t) => t.id)
    expect(ids.sort()).toEqual(['neutral', 'paper'].sort())
    expect(readEffectiveThemeIds()).toEqual(['neutral', 'paper'])
  })

  it('pending session is fail-closed empty; meta still wins over pending', () => {
    expect(readEffectiveThemeIds('pending')).toEqual([])
    expect(effectiveProductThemes('pending')).toEqual([])
    stampMeta('sail-effective-themes', 'neutral')
    expect(readEffectiveThemeIds('pending')).toEqual(['neutral'])
  })

  it('ops EffectiveIds updates grid after async success and drops unsafe ids', async () => {
    stampMeta('sail-project', 'apps/thin')
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input)
      expect(url).toContain('/sail.ops.v1.SailTheme/EffectiveIds')
      return {
        ok: true,
        json: async () => ({
          ok: true,
          all: ['neutral', 'paper', '../evil', 'Has Space'],
        }),
      } as Response
    })
    vi.stubGlobal('fetch', fetchMock)

    const { root, grid } = harness()
    const action = themeSelect(root)
    destroy = action?.destroy

    // Fail-closed while pending.
    expect(optionValues(grid)).toEqual([])

    await vi.waitFor(() => {
      expect(optionValues(grid).sort()).toEqual(['neutral', 'paper'].sort())
    })
    expect(fetchMock).toHaveBeenCalled()
  })

  it('ops failure after pending fail-opens to full THEMES', async () => {
    stampMeta('sail-project', 'apps/thin')
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({ ok: false, json: async () => ({}) }) as Response),
    )

    const { root, grid } = harness()
    const action = themeSelect(root)
    destroy = action?.destroy
    expect(optionValues(grid)).toEqual([])

    await vi.waitFor(() => {
      expect(optionValues(grid).length).toBe(THEMES.length)
    })
  })

  it('ops empty all:[] empties the product grid', async () => {
    stampMeta('sail-project', 'apps/thin')
    vi.stubGlobal(
      'fetch',
      vi.fn(
        async () =>
          ({
            ok: true,
            json: async () => ({ ok: true, all: [] }),
          }) as Response,
      ),
    )

    const { root, grid } = harness()
    const action = themeSelect(root)
    destroy = action?.destroy

    await vi.waitFor(() => {
      // Settled empty (not pending flash alone — stay empty after resolve).
      expect(optionValues(grid)).toEqual([])
      expect(fetch).toHaveBeenCalled()
    })
  })

  it('skips ops fetch when sail-effective-themes meta is already stamped', async () => {
    stampMeta('sail-project', 'apps/thin')
    stampMeta('sail-effective-themes', 'paper')
    const fetchMock = vi.fn(async () => {
      throw new Error('ops should not be called when meta is present')
    })
    vi.stubGlobal('fetch', fetchMock)

    const { root, grid } = harness()
    const action = themeSelect(root)
    destroy = action?.destroy

    expect(optionValues(grid)).toEqual(['paper'])
    // Give microtasks a tick; fetch must never run.
    await Promise.resolve()
    await Promise.resolve()
    expect(fetchMock).not.toHaveBeenCalled()
  })

  it('ops settle under host override still resolves session (clearing override works)', async () => {
    stampMeta('sail-project', 'apps/thin')
    let releaseFetch!: () => void
    const gate = new Promise<void>((r) => {
      releaseFetch = r
    })
    const fetchMock = vi.fn(async () => {
      await gate
      return {
        ok: true,
        json: async () => ({ ok: true, all: ['neutral', 'paper'] }),
      } as Response
    })
    vi.stubGlobal('fetch', fetchMock)

    const { root, grid } = harness()
    const action = themeSelect(root)
    destroy = action?.destroy
    // Pending fail-closed.
    expect(optionValues(grid)).toEqual([])
    expect(fetchMock).toHaveBeenCalled()

    // Host override mid-flight — grid follows override, not pending/session.
    setEffectiveThemeIds(['ship'])
    document.dispatchEvent(new CustomEvent('ship:theme:change'))
    expect(optionValues(grid)).toEqual(['ship'])

    // Ops settles while override is active — must still store session allowlist.
    releaseFetch()
    await expect(fetchMock.mock.results[0]!.value).resolves.toBeTruthy()
    // tryFetch: await json() + outer then-handler
    await Promise.resolve()
    await Promise.resolve()
    await Promise.resolve()
    expect(optionValues(grid)).toEqual(['ship'])

    // Clear override — session should already be settled (not stuck pending).
    setEffectiveThemeIds(null)
    document.dispatchEvent(new CustomEvent('ship:theme:change'))
    expect(optionValues(grid).sort()).toEqual(['neutral', 'paper'].sort())
  })

  it('shows System filter tab for dual-mode themes (ship/sail)', () => {
    const { root } = harness()
    const action = themeSelect(root)
    destroy = action?.destroy

    const system = root.querySelector<HTMLElement>('button[data-mode="system"]')
    expect(system?.hidden).toBe(false)
    const light = root.querySelector<HTMLElement>('button[data-mode="light"]')
    const dark = root.querySelector<HTMLElement>('button[data-mode="dark"]')
    expect(light?.hidden).toBe(false)
    expect(dark?.hidden).toBe(false)
  })

  it('filter tabs do NOT change document mode, theme, or mode localStorage', () => {
    const { root } = harness()
    applyTheme('ship', document.documentElement, 'light')
    persistMode('light')
    const action = themeSelect(root)
    destroy = action?.destroy

    const before = {
      theme: document.documentElement.getAttribute('theme'),
      mode: document.documentElement.dataset.mode,
      stored: localStorage.getItem(THEME_MODE_STORAGE_KEY),
      pref: readModePreference(),
    }

    root.querySelector<HTMLButtonElement>('button[data-mode="dark"]')!.click()
    expect(document.documentElement.getAttribute('theme')).toBe(before.theme)
    expect(document.documentElement.dataset.mode).toBe(before.mode)
    expect(localStorage.getItem(THEME_MODE_STORAGE_KEY)).toBe(before.stored)
    expect(readModePreference()).toBe(before.pref)
    expect(root.getAttribute('data-catalog-filter')).toBe('dark')

    root.querySelector<HTMLButtonElement>('button[data-mode="system"]')!.click()
    expect(document.documentElement.getAttribute('theme')).toBe(before.theme)
    expect(document.documentElement.dataset.mode).toBe(before.mode)
    expect(localStorage.getItem(THEME_MODE_STORAGE_KEY)).toBe(before.stored)
    expect(document.documentElement.dataset.mode).not.toBe('system')
    expect(root.getAttribute('data-catalog-filter')).toBe('system')

    root.querySelector<HTMLButtonElement>('button[data-mode="light"]')!.click()
    expect(document.documentElement.dataset.mode).toBe(before.mode)
    expect(localStorage.getItem(THEME_MODE_STORAGE_KEY)).toBe(before.stored)
    expect(root.getAttribute('data-catalog-filter')).toBe('light')
  })

  it('dark tab hides light-only themes; shows dark-only and duals with data-mode=dark', () => {
    const { root } = harness()
    const action = themeSelect(root)
    destroy = action?.destroy

    root.querySelector<HTMLButtonElement>('button[data-mode="dark"]')!.click()

    // light-only (glass) hidden; dark-only (synthwave) visible
    expect(root.querySelector<HTMLElement>('[role="option"][value="glass"]')?.hidden).toBe(true)
    expect(root.querySelector<HTMLElement>('[role="option"][value="synthwave"]')?.hidden).toBe(
      false,
    )
    // duals visible with dark preview
    expect(root.querySelector<HTMLElement>('[role="option"][value="ship"]')?.hidden).toBe(false)
    expect(
      root.querySelector('[theme="ship"]')?.getAttribute('data-mode'),
    ).toBe('dark')
    expect(
      root.querySelector('[theme="synthwave"]')?.getAttribute('data-mode'),
    ).toBe('dark')
    expect(root.querySelector('[data-catalog-label]')?.textContent).toMatch(/Dark/i)
    expect(
      root.querySelector('button[data-mode="dark"]')?.getAttribute('aria-selected'),
    ).toBe('true')
  })

  it('light tab hides dark-only; duals get data-mode=light', () => {
    const { root } = harness()
    const action = themeSelect(root)
    destroy = action?.destroy

    // Switch to dark first, then back to light (filter only)
    root.querySelector<HTMLButtonElement>('button[data-mode="dark"]')!.click()
    root.querySelector<HTMLButtonElement>('button[data-mode="light"]')!.click()

    expect(root.querySelector<HTMLElement>('[role="option"][value="synthwave"]')?.hidden).toBe(
      true,
    )
    expect(root.querySelector<HTMLElement>('[role="option"][value="glass"]')?.hidden).toBe(false)
    expect(root.querySelector<HTMLElement>('[role="option"][value="ship"]')?.hidden).toBe(false)
    expect(
      root.querySelector('[theme="ship"]')?.getAttribute('data-mode'),
    ).toBe('light')
    expect(
      root.querySelector('[theme="glass"]')?.getAttribute('data-mode'),
    ).toBe('light')
    expect(root.querySelector('[data-catalog-label]')?.textContent).toMatch(/Light/i)
  })

  it('system tab shows only dual-mode themes; single-mode hidden', () => {
    const { root } = harness()
    const action = themeSelect(root)
    destroy = action?.destroy

    root.querySelector<HTMLButtonElement>('button[data-mode="system"]')!.click()

    expect(root.querySelector<HTMLElement>('[role="option"][value="glass"]')?.hidden).toBe(true)
    expect(root.querySelector<HTMLElement>('[role="option"][value="synthwave"]')?.hidden).toBe(
      true,
    )
    expect(root.querySelector<HTMLElement>('[role="option"][value="ship"]')?.hidden).toBe(false)
    expect(root.querySelector('[data-catalog-label]')?.textContent).toMatch(/Dual/i)
    // Document paint unchanged by tab
    expect(document.documentElement.dataset.mode).not.toBe('system')
  })

  it('clicking a theme under dark filter applies that theme with dark paint', () => {
    const { root } = harness()
    applyTheme('ship', document.documentElement, 'light')
    persistMode('light')
    const action = themeSelect(root)
    destroy = action?.destroy

    root.querySelector<HTMLButtonElement>('button[data-mode="dark"]')!.click()
    // Tab must not have painted yet
    expect(document.documentElement.dataset.mode).toBe('light')

    // neutral is dual-mode (visible under dark); paper is light-only
    const neutral = root.querySelector<HTMLButtonElement>('[role="option"][value="neutral"]')!
    expect(neutral.hidden).toBe(false)
    neutral.click()

    expect(document.documentElement.getAttribute('theme')).toBe('neutral')
    expect(document.documentElement.dataset.mode).toBe('dark')
    expect(readModePreference()).toBe('dark')
    expect(localStorage.getItem(THEME_MODE_STORAGE_KEY)).toBe('dark')
  })

  it('selecting a theme does not close the popover', () => {
    const { root, panel } = harness()
    let closed = false
    panel.hidePopover = () => {
      closed = true
    }
    const action = themeSelect(root)
    destroy = action?.destroy
    root.querySelector<HTMLButtonElement>('[role="option"][value="neutral"]')!.click()
    expect(document.documentElement.getAttribute('theme')).toBe('neutral')
    expect(closed).toBe(false)
  })

  it('selected card exposes check mark via data-selected-check', () => {
    const { root, grid } = harness()
    for (const o of grid.querySelectorAll('[role="option"]')) {
      if (!o.querySelector('[data-selected-check]')) {
        const mark = document.createElement('span')
        mark.setAttribute('data-selected-check', '')
        mark.hidden = true
        o.appendChild(mark)
      }
    }
    const action = themeSelect(root)
    destroy = action?.destroy
    const paper = root.querySelector<HTMLButtonElement>('[role="option"][value="paper"]')!
    paper.click()
    expect(paper.getAttribute('aria-selected')).toBe('true')
    expect(paper.querySelector<HTMLElement>('[data-selected-check]')?.hidden).toBe(false)
    const ship = root.querySelector<HTMLButtonElement>('[role="option"][value="ship"]')!
    expect(ship.querySelector<HTMLElement>('[data-selected-check]')?.hidden).toBe(true)
  })

  it('selected header tracks active theme label', () => {
    const { root } = harness()
    const action = themeSelect(root)
    destroy = action?.destroy
    root.querySelector<HTMLButtonElement>('[role="option"][value="paper"]')!.click()
    expect(root.querySelector('[data-selected-label]')?.textContent).toMatch(/Paper/i)
  })

  it('selecting a theme under light filter keeps light paint', () => {
    const { root } = harness()
    const action = themeSelect(root)
    destroy = action?.destroy

    applyTheme('ship', document.documentElement, 'light')
    persistMode('light')
    root.querySelector<HTMLButtonElement>('button[data-mode="light"]')!.click()
    const neutral = root.querySelector<HTMLButtonElement>('[role="option"][value="neutral"]')!
    neutral.click()
    expect(document.documentElement.getAttribute('theme')).toBe('neutral')
    expect(document.documentElement.dataset.mode).toBe('light')
    expect(neutral.getAttribute('aria-selected')).toBe('true')
    expect(
      root.querySelector('[role="option"][value="ship"]')?.getAttribute('aria-selected'),
    ).toBe('false')
  })

  it('selecting dual theme under system filter persists system preference', () => {
    const { root } = harness()
    applyTheme('ship', document.documentElement, 'light')
    persistMode('light')
    const action = themeSelect(root)
    destroy = action?.destroy

    root.querySelector<HTMLButtonElement>('button[data-mode="system"]')!.click()
    expect(localStorage.getItem(THEME_MODE_STORAGE_KEY)).toBe('light') // filter only

    // dual-mode theme under system filter
    root.querySelector<HTMLButtonElement>('[role="option"][value="neutral"]')!.click()
    expect(document.documentElement.getAttribute('theme')).toBe('neutral')
    expect(readModePreference()).toBe('system')
    expect(document.documentElement.dataset.mode).not.toBe('system')
    expect(['light', 'dark']).toContain(document.documentElement.dataset.mode)
  })

  it('modeOnly hides theme grid and cycles mode on trigger', () => {
    const { root, grid, modeTrigger } = harness({ modeOnly: true })
    document.documentElement.setAttribute('theme', 'ship')
    document.documentElement.dataset.mode = 'light'
    persistMode('light')
    const action = themeSelect(root)
    destroy = action?.destroy
    expect(grid.hidden).toBe(true)
    expect(modeTrigger.hidden).toBe(false)
    expect(root.getAttribute('data-mode-only')).toBe('1')

    // Cycle light → dark
    modeTrigger.click()
    expect(document.documentElement.dataset.mode).toBe('dark')
    expect(readModePreference()).toBe('dark')
    expect(modeTrigger.getAttribute('aria-label')).toMatch(/Dark/i)

    // Cycle dark → system
    modeTrigger.click()
    expect(readModePreference()).toBe('system')
    expect(document.documentElement.dataset.mode).not.toBe('system')
    expect(modeTrigger.getAttribute('aria-label')).toMatch(/System/i)

    // Cycle system → light
    modeTrigger.click()
    expect(readModePreference()).toBe('light')
    expect(document.documentElement.dataset.mode).toBe('light')
  })

  it('unique popover id links trigger for anchor', () => {
    const a = harness()
    const b = harness()
    const da = themeSelect(a.root)
    const db = themeSelect(b.root)
    destroy = () => {
      da?.destroy?.()
      db?.destroy?.()
    }
    const idA = a.panel.id
    const idB = b.panel.id
    expect(idA).toMatch(/^sail-theme-panel-/)
    expect(idB).toMatch(/^sail-theme-panel-/)
    expect(idA).not.toBe(idB)
    expect(a.trigger.getAttribute('popovertarget')).toBe(idA)
    expect(b.trigger.getAttribute('popovertarget')).toBe(idB)
  })

  it('heals legacy data-mode=system on the document', () => {
    document.documentElement.setAttribute('theme', 'ship')
    document.documentElement.dataset.mode = 'system'
    const { root } = harness()
    const action = themeSelect(root)
    destroy = action?.destroy
    expect(document.documentElement.dataset.mode).not.toBe('system')
    expect(['light', 'dark']).toContain(document.documentElement.dataset.mode)
  })

  it('no-ops document paint while studioPreview is active (U2.1c)', () => {
    const { root } = harness()
    const action = themeSelect(root)
    destroy = action?.destroy
    document.documentElement.setAttribute('theme', 'ship')
    document.documentElement.dataset.mode = 'light'
    document.documentElement.dataset.studioPreview = '1'

    // Tabs still filter locally but never paint (already true)
    root.querySelector<HTMLButtonElement>('button[data-mode="dark"]')!.click()
    expect(document.documentElement.dataset.mode).toBe('light')
    expect(root.getAttribute('data-catalog-filter')).toBe('dark')

    root.querySelector<HTMLButtonElement>('[role="option"][value="paper"]')!.click()
    expect(document.documentElement.getAttribute('theme')).toBe('ship')
  })

  it('initial catalog filter follows live paint (not storage preference alone)', () => {
    // Storage says light, document is dark (config boot / external apply).
    applyTheme('ship', document.documentElement, 'dark')
    persistMode('light')
    expect(readModePreference()).toBe('light')

    const { root } = harness()
    document.documentElement.setAttribute('theme', 'ship')
    document.documentElement.dataset.mode = 'dark'

    const action = themeSelect(root)
    destroy = action?.destroy

    expect(root.getAttribute('data-catalog-filter')).toBe('dark')
    expect(document.documentElement.getAttribute('theme')).toBe('ship')
    expect(document.documentElement.dataset.mode).toBe('dark')
    expect(
      root.querySelector('button[data-mode="dark"]')?.getAttribute('aria-selected'),
    ).toBe('true')
    // Dark-capable catalog row (8bit) is visible under dark filter.
    const eight = root.querySelector<HTMLElement>('[role="option"][value="8bit"]')
    expect(eight).toBeTruthy()
    expect(eight!.hidden).toBe(false)
  })

  it('sync reconciles catalog filter to live paint after external theme change', () => {
    applyTheme('ship', document.documentElement, 'light')
    persistMode('light')
    const { root } = harness()
    document.documentElement.setAttribute('theme', 'ship')
    document.documentElement.dataset.mode = 'light'
    const action = themeSelect(root)
    destroy = action?.destroy
    expect(root.getAttribute('data-catalog-filter')).toBe('light')

    // External writer flips paint without going through the filter tabs.
    applyTheme('ship', document.documentElement, 'dark')
    document.dispatchEvent(new CustomEvent('ship:theme:change'))

    expect(root.getAttribute('data-catalog-filter')).toBe('dark')
    expect(
      root.querySelector('button[data-mode="dark"]')?.getAttribute('aria-selected'),
    ).toBe('true')
  })
})
