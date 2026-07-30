/**
 * Minimal JSONC parser — strip // and /* *\/ comments + trailing commas.
 * Self-contained (no ship-substrate / deck-config import).
 */

export function stripJsonc(input: string): string {
  // Phase 1: strip comments, respecting strings.
  let stage = ''
  let i = 0
  let inString = false
  let escape = false
  while (i < input.length) {
    const c = input[i]!
    if (inString) {
      stage += c
      if (escape) {
        escape = false
      } else if (c === '\\') {
        escape = true
      } else if (c === '"') {
        inString = false
      }
      i++
      continue
    }
    if (c === '"') {
      inString = true
      stage += c
      i++
      continue
    }
    if (c === '/' && input[i + 1] === '/') {
      i += 2
      while (i < input.length && input[i] !== '\n') i++
      continue
    }
    if (c === '/' && input[i + 1] === '*') {
      i += 2
      while (i < input.length && !(input[i] === '*' && input[i + 1] === '/')) i++
      i += 2
      continue
    }
    stage += c
    i++
  }

  // Phase 2: remove trailing commas before ] or }.
  let out = ''
  inString = false
  escape = false
  i = 0
  while (i < stage.length) {
    const c = stage[i]!
    if (inString) {
      out += c
      if (escape) escape = false
      else if (c === '\\') escape = true
      else if (c === '"') inString = false
      i++
      continue
    }
    if (c === '"') {
      inString = true
      out += c
      i++
      continue
    }
    if (c === ',') {
      let j = i + 1
      while (j < stage.length && /\s/.test(stage[j]!)) j++
      if (stage[j] === ']' || stage[j] === '}') {
        i++
        continue
      }
    }
    out += c
    i++
  }
  return out
}

export function parseJsonc<T = unknown>(input: string): T {
  return JSON.parse(stripJsonc(input)) as T
}
