
export type SailTokenType =
  | 'tag'
  | 'tagName'
  | 'attrName'
  | 'attrValue'
  | 'sx'
  | 'expr'
  | 'directive'
  | 'comment'
  | 'text'
  | 'punctuation'
  | 'string'

export type SailToken = {
  type: SailTokenType
  text: string
  from: number
  to: number
}

/**
 * Tokenize a .sail source slice into a flat token list (deterministic).
 * Covers: HTML-like tags, attributes, block style({…}), {expr}, control-flow tags,
 * comments, and string literals.
 */
export function tokenizeSail(source: string): SailToken[] {
  const tokens: SailToken[] = []
  const n = source.length
  let i = 0

  const push = (type: SailTokenType, from: number, to: number) => {
    if (to <= from) return
    tokens.push({ type, text: source.slice(from, to), from, to })
  }

  const isIdentStart = (c: string) => /[A-Za-z_:]/.test(c)
  const isIdent = (c: string) => /[A-Za-z0-9_.:-]/.test(c)

  while (i < n) {
    // Line comment <!-- … -->
    if (source.startsWith('<!--', i)) {
      const end = source.indexOf('-->', i + 4)
      const to = end < 0 ? n : end + 3
      push('comment', i, to)
      i = to
      continue
    }

    // Control / open / close tags
    if (source[i] === '<') {
      const tagStart = i
      i++
      let isClose = false
      if (source[i] === '/') {
        isClose = true
        i++
      }
      push('punctuation', tagStart, i)

      // tag name
      if (i < n && isIdentStart(source[i]!)) {
        const nameFrom = i
        while (i < n && isIdent(source[i]!)) i++
        const name = source.slice(nameFrom, i)
        const dir = isDirectiveTag(name)
        push(dir ? 'directive' : 'tagName', nameFrom, i)
      }

      // attributes until > or />
      while (i < n) {
        while (i < n && /\s/.test(source[i]!)) i++
        if (i >= n) break
        if (source[i] === '>') {
          push('punctuation', i, i + 1)
          i++
          break
        }
        if (source.startsWith('/>', i)) {
          push('punctuation', i, i + 2)
          i += 2
          break
        }
        // attr name (may include prefix on: / bind: / sx)
        if (isIdentStart(source[i]!) || source[i] === '@') {
          const aFrom = i
          while (i < n && (isIdent(source[i]!) || source[i] === '|')) i++
          const attr = source.slice(aFrom, i)
          const isSx = attr === 'sx' || attr.endsWith(':sx')
          push(isSx ? 'sx' : 'attrName', aFrom, i)
          while (i < n && /\s/.test(source[i]!)) i++
          if (source[i] === '=') {
            push('punctuation', i, i + 1)
            i++
            while (i < n && /\s/.test(source[i]!)) i++
            // value
            if (source[i] === '"' || source[i] === "'") {
              const q = source[i]!
              const vFrom = i
              i++
              while (i < n && source[i] !== q) {
                if (source[i] === '\\') i++
                i++
              }
              if (i < n) i++
              push(isSx ? 'sx' : 'attrValue', vFrom, i)
            } else if (source[i] === '{') {
              const vFrom = i
              i = skipBrace(source, i)
              push(isSx ? 'sx' : 'expr', vFrom, i)
            } else if (i < n && !/[>\s/]/.test(source[i]!)) {
              const vFrom = i
              while (i < n && !/[\s>]/.test(source[i]!) && !source.startsWith('/>', i)) i++
              push(isSx ? 'sx' : 'attrValue', vFrom, i)
            }
          }
          continue
        }
        // stray char inside tag
        push('punctuation', i, i + 1)
        i++
        if (isClose) {
          /* keep scanning */
        }
      }
      continue
    }

    // Brace expression / directive in text
    if (source[i] === '{') {
      const from = i
      i = skipBrace(source, i)
      const body = source.slice(from + 1, i - 1)
      const type: SailTokenType =
        body.startsWith('#') || body.startsWith('/') || body.startsWith(':')
          ? 'directive'
          : 'expr'
      push(type, from, i)
      continue
    }

    // Plain text run
    const tFrom = i
    while (i < n && source[i] !== '<' && source[i] !== '{') i++
    push('text', tFrom, i)
  }

  return tokens
}

function isDirectiveTag(name: string): boolean {
  return /^(if|else-if|else|for|match|when|case|default|scope|slot|controller)$/i.test(
    name,
  )
}

/** Advance past `{…}` honoring nested braces and quotes. */
export function skipBrace(source: string, openIdx: number): number {
  let depth = 1
  let i = openIdx + 1
  let quote: string | null = null
  while (i < source.length) {
    const ch = source[i]!
    if (quote) {
      if (ch === '\\') {
        i += 2
        continue
      }
      if (ch === quote) quote = null
      i++
      continue
    }
    if (ch === '"' || ch === "'") {
      quote = ch
      i++
      continue
    }
    if (ch === '{') {
      depth++
      i++
      continue
    }
    if (ch === '}') {
      depth--
      i++
      if (depth === 0) return i
      continue
    }
    i++
  }
  return i
}

/** Compact token stream for snapshot tests: `type:text` lines. */
export function tokenSnapshot(source: string): string[] {
  return tokenizeSail(source).map((t) => {
    const text = t.text.replace(/\s+/g, ' ').trim()
    if (!text) return `${t.type}:␠`
    return `${t.type}:${text.length > 40 ? text.slice(0, 37) + '…' : text}`
  })
}

/**
 * Highlight classes used by a CM decoration overlay (optional host).
 * Maps token types → CSS class names.
 */
export const SAIL_TOKEN_CLASS: Record<SailTokenType, string> = {
  tag: 'cm-sail-tag',
  tagName: 'cm-sail-tagName',
  attrName: 'cm-sail-attrName',
  attrValue: 'cm-sail-attrValue',
  sx: 'cm-sail-sx',
  expr: 'cm-sail-expr',
  directive: 'cm-sail-directive',
  comment: 'cm-sail-comment',
  text: 'cm-sail-text',
  punctuation: 'cm-sail-punct',
  string: 'cm-sail-string',
}

/**
 * Minimal StreamLanguage-like face for CM6 hosts.
 * Returns a tokenize function compatible with @codemirror/language StreamLanguage.define.
 */
export function sailStreamTokenizer() {
  return {
    name: 'sail',
    startState: () => ({ inTag: false }),
    token(stream: {
      sol: () => boolean
      eol: () => boolean
      peek: () => string | undefined
      next: () => string | undefined
      eat: (m: string | RegExp) => string | undefined
      eatWhile: (m: string | RegExp) => boolean
      match: (m: string | RegExp, consume?: boolean) => boolean | RegExpMatchArray | null
      current: () => string
      skipToEnd: () => void
      pos: number
    }): string | null {
      if (stream.match(/<!--/, true)) {
        while (!stream.eol() && !stream.match(/-->/, true)) stream.next()
        stream.match(/-->/, true)
        return 'comment'
      }
      if (stream.match(/<\/?/, true)) {
        stream.eatWhile(/[A-Za-z0-9_.:-]/)
        return 'tagName'
      }
      if (stream.match(/\{/, true)) {
        let depth = 1
        while (!stream.eol() && depth > 0) {
          const c = stream.next()
          if (c === '{') depth++
          else if (c === '}') depth--
        }
        return 'expr'
      }
      if (stream.match(/sx\s*=/, true)) return 'sx'
      if (stream.match(/[A-Za-z_:][\w.:|-]*/, true)) return 'attrName'
      if (stream.match(/"[^"]*"|'[^']*'/, true)) return 'string'
      stream.next()
      return null
    },
  }
}
