
import type { EditorSetting } from './types'

const DEFAULTS: Record<string, string> = {
  vscode: 'vscode://file/{path}:{line}:{col}',
  cursor: 'cursor://file/{path}:{line}:{col}',
  zed: 'zed://file/{path}:{line}:{col}',
}

export function defaultEditorSetting(scheme = 'vscode'): EditorSetting {
  const s = scheme.toLowerCase()
  return {
    scheme: s,
    template: DEFAULTS[s] ?? DEFAULTS.vscode!,
  }
}

/**
 * Parse sail.jsonc `editor` field:
 * - "vscode" | "cursor" | "zed"
 * - { "scheme": "vscode" }
 * - { "url": "vscode://file/{path}:{line}:{col}" }
 * - { "template": "..." }
 */
export function parseEditorSetting(raw: unknown): EditorSetting {
  if (raw == null || raw === '') return defaultEditorSetting('vscode')
  if (typeof raw === 'string') {
    const t = raw.trim()
    if (t.includes('{path}') || t.includes('://')) {
      const scheme = t.split('://')[0] || 'vscode'
      return { scheme, template: t }
    }
    return defaultEditorSetting(t)
  }
  if (typeof raw === 'object') {
    const o = raw as Record<string, unknown>
    if (typeof o.url === 'string') {
      const scheme =
        typeof o.scheme === 'string' ? o.scheme : o.url.split('://')[0] || 'vscode'
      return { scheme, template: o.url }
    }
    if (typeof o.template === 'string') {
      const scheme =
        typeof o.scheme === 'string'
          ? o.scheme
          : o.template.split('://')[0] || 'vscode'
      return { scheme, template: o.template }
    }
    if (typeof o.scheme === 'string') return defaultEditorSetting(o.scheme)
  }
  return defaultEditorSetting('vscode')
}

/**
 * Build deep-link URL for open-source at span.
 * Path is used as-is (caller resolves relative → absolute when known).
 */
export function editorDeepLink(
  setting: EditorSetting,
  filePath: string,
  line: number,
  col = 1,
): string {
  const path = filePath.replace(/\\/g, '/')
  return setting.template
    .replaceAll('{path}', path)
    .replaceAll('{line}', String(Math.max(1, line | 0)))
    .replaceAll('{col}', String(Math.max(1, col | 0)))
}

/** Assert URL shape for tests / validation. */
export function isValidEditorDeepLink(url: string): boolean {
  return (
    /^(vscode|cursor|zed):\/\/file\/.+/.test(url) ||
    /^[a-z][a-z0-9+.-]*:\/\/.+\d/.test(url)
  )
}
