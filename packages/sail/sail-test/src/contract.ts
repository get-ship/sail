// Lean test-map consumption (SPEC-TEST-2). Same shape the compiler emits.
// Query via role / name / state — never data-* or class atoms.

import { getByRole, type ByRoleOptions } from './queries'

export type TestEntry = {
  role?: string
  name?: string
  states?: string[]
}

export type TestMap = {
  version: number
  file: string
  entries: TestEntry[]
}

const MAP_KEYS = new Set(['version', 'file', 'entries'])
const ENTRY_KEYS = new Set(['role', 'name', 'states'])

export function isLeanTestMap(map: unknown): map is TestMap {
  if (!map || typeof map !== 'object') return false
  const rec = map as Record<string, unknown>
  if (Object.keys(rec).some((k) => !MAP_KEYS.has(k))) return false
  if (rec.version !== 1 || typeof rec.file !== 'string' || !Array.isArray(rec.entries)) {
    return false
  }
  return rec.entries.every((entry) => {
    if (!entry || typeof entry !== 'object') return false
    const e = entry as Record<string, unknown>
    if (Object.keys(e).some((k) => !ENTRY_KEYS.has(k))) return false
    return Boolean(e.role || e.name)
  })
}

/** Role + options both DOM `getByRole` and Playwright `page.getByRole` accept. */
export function contractQuery(entry: TestEntry): { role: string; options: ByRoleOptions } {
  if (!entry.role) {
    throw new Error('contractQuery: entry has no role')
  }
  const options: ByRoleOptions = {}
  if (entry.name) options.name = entry.name
  if (entry.states?.includes('pressed')) options.pressed = true
  if (entry.states?.includes('expanded')) options.expanded = true
  if (entry.states?.includes('selected')) options.selected = true
  if (entry.states?.includes('checked')) options.checked = true
  if (entry.states?.includes('hidden')) options.hidden = true
  return { role: entry.role, options }
}

/** Locate a published contract entry in a mounted tree. */
export function queryContract(container: ParentNode, entry: TestEntry): HTMLElement {
  const { role, options } = contractQuery(entry)
  return getByRole(container, role, options)
}

type RoleHost<T> = {
  getByRole(
    role: string,
    options?: {
      name?: string | RegExp
      pressed?: boolean
      expanded?: boolean
      selected?: boolean
      checked?: boolean
    },
  ): T
}

/** Playwright `page` / `locator` — same contract, no data-* / class. */
export function getByContract<T>(host: RoleHost<T>, entry: TestEntry): T {
  const { role, options } = contractQuery(entry)
  return host.getByRole(role, {
    name: options.name,
    pressed: options.pressed === true ? true : undefined,
    expanded: options.expanded,
    selected: options.selected,
    checked: options.checked === true ? true : undefined,
  })
}
