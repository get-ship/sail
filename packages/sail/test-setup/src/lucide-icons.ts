// Offline icon primer for vitest. Reads `lucide-static` (or a JSON map) so
// jsdom tests paint `<icon name>` without a daemon or CDN.
//
// Named icons never fetch at runtime in tests — we prime the same cache the
// action uses. Role aliases match the compiler core table (`git` → `git-branch`).

import { existsSync, readdirSync, readFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const MODULE_DIR = dirname(fileURLToPath(import.meta.url))

/** Role aliases kept in lockstep with `sail_compiler::icons` core rows. */
const ICON_ALIASES = [
  ['git', 'git-branch'],
  ['refresh', 'refresh-cw'],
  ['stop', 'stop-circle'],
  ['warning', 'triangle-alert'],
  ['close', 'x'],
  ['trash', 'trash-2'],
  ['add', 'plus'],
  ['edit', 'pencil'],
  ['success', 'check-circle'],
  ['error', 'x-circle'],
  ['file-code', 'file-code-2'],
] as const

function wrap(inner: string): string {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" `
    + `stroke="currentColor" stroke-width="2" stroke-linecap="round" `
    + `stroke-linejoin="round">${inner}</svg>`
}

function extractInner(svg: string): string {
  const stripped = svg.replace(/<!--[\s\S]*?-->/g, '').trim()
  const openEnd = stripped.indexOf('>')
  const close = stripped.lastIndexOf('</svg>')
  if (openEnd < 0 || close <= openEnd) return stripped
  return stripped.slice(openEnd + 1, close).trim().replace(/\s+/g, ' ')
}

function locateLucideStaticIcons(): string {
  const fromEnv = process.env.LUCIDE_ICONS_JSON ?? process.env.LUCIDE_STATIC_ICONS
  if (fromEnv && existsSync(fromEnv)) return fromEnv
  const candidates = [
    resolve(MODULE_DIR, '../../../../node_modules/lucide-static/icons'),
    resolve(MODULE_DIR, '../../../../../node_modules/lucide-static/icons'),
    resolve(process.cwd(), 'node_modules/lucide-static/icons'),
  ]
  for (const c of candidates) {
    if (existsSync(c)) return c
  }
  throw new Error(
    `lucide-static/icons not found (looked under node_modules). ` +
    `Set LUCIDE_STATIC_ICONS or LUCIDE_ICONS_JSON to a glyphs dir or JSON map.`,
  )
}

/** Path to the offline pack (dir of SVGs, or a JSON map). */
export function defaultLucideIconsPath(): string {
  return locateLucideStaticIcons()
}

/** Read a JSON map (`name → inner fragment`) or a directory of `.svg` files. */
export function loadLucideIconMap(jsonPath: string = defaultLucideIconsPath()): Map<string, string> {
  const out = new Map<string, string>()
  if (jsonPath.endsWith('.json')) {
    const icons: Record<string, string> = JSON.parse(readFileSync(jsonPath, 'utf8'))
    for (const [name, inner] of Object.entries(icons)) {
      out.set(name, wrap(inner))
    }
  } else {
    for (const file of readdirSync(jsonPath)) {
      if (!file.endsWith('.svg')) continue
      const stem = file.slice(0, -4)
      const raw = readFileSync(join(jsonPath, file), 'utf8')
      out.set(stem, wrap(extractInner(raw)))
    }
  }
  for (const [alias, real] of ICON_ALIASES) {
    const svg = out.get(real)
    if (svg) out.set(alias, svg)
  }
  return out
}

/** Convenience: read the pack + push every entry through `primer`. */
export function primeLucideIcons(
  primer: (name: string, svg: string) => void,
  jsonPath: string = defaultLucideIconsPath(),
): void {
  for (const [name, svg] of loadLucideIconMap(jsonPath)) primer(name, svg)
}
