/** Flatten JSON into tree rows for sail-json-view. */

export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [k: string]: JsonValue }

export interface Pad {
  i: number
}

export interface Row {
  k: string
  path: string
  kind: 'branch' | 'close' | 'str' | 'num' | 'bool' | 'null' | 'empty'
  level: number
  ariaLevel: string
  pads: Pad[]
  label: string
  text: string
  open: boolean
  openBracket: string
  closeBracket: string
  meta: string
}

export function buildJsonRows(
  parsed: JsonValue,
  overrides: Record<string, boolean>,
  expanded: boolean,
  depth: number,
): Row[] {
  const out: Row[] = []

  const pads = (level: number): Pad[] =>
    Array.from({ length: level }, (_, i) => ({ i }))

  const walk = (v: JsonValue, label: string, level: number, path: string): void => {
    const base = {
      path,
      level,
      ariaLevel: String(level + 1),
      pads: pads(level),
      label,
      meta: '',
      open: false,
      openBracket: '',
      closeBracket: '',
    }
    if (v === null) {
      out.push({ ...base, k: `${path}:null`, kind: 'null', text: 'null' })
      return
    }
    if (typeof v === 'string') {
      out.push({ ...base, k: `${path}:str:${v}`, kind: 'str', text: `"${v}"` })
      return
    }
    if (typeof v === 'number') {
      out.push({ ...base, k: `${path}:num:${v}`, kind: 'num', text: String(v) })
      return
    }
    if (typeof v === 'boolean') {
      out.push({ ...base, k: `${path}:bool:${v}`, kind: 'bool', text: String(v) })
      return
    }
    const isArr = Array.isArray(v)
    const entries: Array<[string, JsonValue]> = isArr
      ? (v as JsonValue[]).map((x, i) => [String(i), x])
      : Object.entries(v as Record<string, JsonValue>)
    const openBracket = isArr ? '[' : '{'
    const closeBracket = isArr ? ']' : '}'
    if (entries.length === 0) {
      out.push({
        ...base,
        k: `${path}:empty`,
        kind: 'empty',
        text: `${openBracket}${closeBracket}`,
      })
      return
    }
    const open = overrides[path] ?? (expanded || level < depth)
    out.push({
      ...base,
      k: `${path}:branch:${open}`,
      kind: 'branch',
      text: '',
      open,
      openBracket,
      closeBracket,
      meta: `${entries.length} ${isArr ? 'items' : 'keys'}`,
    })
    if (open) {
      for (const [k, val] of entries) {
        walk(val, isArr ? '' : k, level + 1, `${path}.${k}`)
      }
      out.push({ ...base, k: `${path}:close`, kind: 'close', text: closeBracket })
    }
  }

  walk(parsed, '', 0, '$')
  return out
}
