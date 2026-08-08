/** Pure file-list helpers for sail-file-upload. */

export function fmtSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1048576).toFixed(1)} MB`
}

/** True when `file` matches a comma-separated accept string (ext, type, or type/*). */
export function matchesAccept(file: File, accept: string): boolean {
  const tokens = accept
    .split(',')
    .map((t: string) => t.trim().toLowerCase())
    .filter(Boolean)
  if (tokens.length === 0) return true
  const ext = '.' + (file.name.split('.').pop() || '').toLowerCase()
  const type = file.type.toLowerCase()
  return tokens.some((t: string) => {
    if (t.startsWith('.')) return ext === t
    if (t.endsWith('/*')) return type.startsWith(t.slice(0, -1))
    return type === t
  })
}
