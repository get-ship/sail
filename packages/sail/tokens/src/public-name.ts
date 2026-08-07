/**
 * Generator-owned public short names for design tokens.
 *
 * Twin of `sail_compiler::tokens_generate::public_name`. Design identity is
 * the DTCG path; CSS/Style public names (`primary`, `chip`, …) are derived
 * here — not stamped as `$extensions.*.cssName` in the model.
 *
 * Keep in lockstep with `crates/sail/compiler/src/tokens_generate.rs`.
 */

/** `brandAccent` → `brand-accent`; `menuMinW` → `menu-min-w`. */
function camelToKebab(s: string): string {
  let out = ''
  for (let i = 0; i < s.length; i++) {
    const c = s[i]!
    const code = c.charCodeAt(0)
    const isUpper = code >= 65 && code <= 90
    if (isUpper) {
      const prev = i > 0 ? s[i - 1]! : ''
      const next = i + 1 < s.length ? s[i + 1]! : ''
      const prevLowerOrDigit =
        i > 0 &&
        ((prev.charCodeAt(0) >= 97 && prev.charCodeAt(0) <= 122) ||
          (prev.charCodeAt(0) >= 48 && prev.charCodeAt(0) <= 57))
      const nextLower = next.charCodeAt(0) >= 97 && next.charCodeAt(0) <= 122
      const prevUpper = i > 0 && prev.charCodeAt(0) >= 65 && prev.charCodeAt(0) <= 90
      if (prevLowerOrDigit || (prevUpper && nextLower)) {
        out += '-'
      }
      out += c.toLowerCase()
    } else {
      out += c
    }
  }
  return out
}

/** Packs where `component.{pack}.track` is the primary surface (like `.bg`). */
const TRACK_AS_PRIMARY = new Set(['meter', 'progress', 'switch'])

/**
 * Map DTCG path → public short name used in tokens.css / Style / registry.
 * Rules reproduce the historical cssName map 100%.
 */
export function publicName(path: string): string {
  if (path === 'component.error-boundary.icon-fg') return 'error-boundary-icon'
  if (path === 'radius.default') return 'radius'

  const parts = path.split('.')
  if (parts.length === 0 || (parts.length === 1 && parts[0] === '')) return ''

  const root = parts[0]!

  if (root === 'component' && parts.length >= 2) {
    const pack = parts[1]!
    const rest = parts.slice(2)
    if (rest.length === 0) return pack
    if (rest.length === 1 && rest[0] === 'bg') return pack
    if (rest.length === 1 && rest[0] === 'fg') return `${pack}-foreground`
    if (rest.length === 1 && rest[0] === 'track' && TRACK_AS_PRIMARY.has(pack)) {
      return pack
    }
    const mapped = rest.map((p) => {
      if (p === 'fg') return 'foreground'
      if (p.endsWith('-fg')) return `${p.slice(0, -3)}-foreground`
      return camelToKebab(p)
    })
    return `${pack}-${mapped.join('-')}`
  }

  if (root === 'color') {
    const rest = parts.slice(1)
    if (rest[0] === 'brandAccent') {
      if (rest.length === 2 && rest[1] === 'default') return 'accent'
      return `accent-${rest.slice(1).map(camelToKebab).join('-')}`
    }
    if (rest[0] === 'mix' && rest.length === 2) {
      const leaf = rest[1]!
      if (leaf.endsWith('Mix')) {
        return `mix-${camelToKebab(leaf.slice(0, -3))}`
      }
    }
    const segs = rest.map(camelToKebab)
    if (segs.length >= 2 && segs[segs.length - 1] === 'default') {
      segs.pop()
    }
    return segs.join('-')
  }

  if (root === 'typography' || root === 'motion' || root === 'interaction') {
    return parts.slice(1).map(camelToKebab).join('-')
  }

  if (root === 'control' && parts.length >= 2 && parts[1] === 'press') {
    return parts.slice(1).map(camelToKebab).join('-')
  }

  if (root === 'icon' && parts.length === 2) {
    return `icon-size-${parts[1]}`
  }

  return parts.map(camelToKebab).join('-')
}
